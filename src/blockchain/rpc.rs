use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};
use crate::models::{OutPoint, Transaction, TxInput, TxOutput};
use async_trait::async_trait;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct BitcoinRpcClient {
    url: String,
    username: String,
    password: String,
    client: reqwest::Client,
}

impl BitcoinRpcClient {
    pub fn new(url: String, username: String, password: String) -> Self {
        BitcoinRpcClient {
            url,
            username,
            password,
            client: reqwest::Client::new(),
        }
    }

    pub async fn rpc_call(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // request body needs id otherwise no response
        let id = 1;
        // JSON-RPC 2.0 request body
        let rpc_request_body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params" : params,
            "id": id,
        });

        // Post request to RPC server
        let response = self
            .client
            .post(&self.url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&rpc_request_body)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;

        // convert response to serde_json value
        let json_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;
        // print!("json_response from rpc call {:?}", json_response);

        if let Some(rpc_error) = json_response.get("error").and_then(|e| e.as_object()) {
            if !rpc_error.is_empty() {
                return Err(BlockchainError::Other(
                    rpc_error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown Error")
                        .to_string(),
                ));
            }
        }

        // Extract and return the result field
        let result = json_response.get("result").cloned().ok_or_else(|| {
            BlockchainError::DataInconsistency("No result found in response".to_string())
        })?;

        Ok(result)
    }
}

#[async_trait]
impl BlockchainDataSource for BitcoinRpcClient {
    async fn get_transaction(&self, txid: &str) -> Result<Transaction> {
        let rpc_client = BitcoinRpcClient::new(
            "http://127.0.0.1:8332".to_string(),
            "testnode".to_string(),
            "asdjio3u2o23d32dpiadas".to_string(),
        );

        let rpc_result = rpc_client
            .rpc_call("getrawtransaction", vec![json!(txid), json!(1)])
            .await
            .unwrap();

        // Extract some basic fields
        let txid = rpc_result
            .get("txid")
            .expect("txid should always be present")
            .to_string();
        let blocktime = rpc_result
            .get("blocktime")
            .unwrap()
            .as_u64()
            .expect("Blocktime should always be present");
        let blockhash = rpc_result
            .get("blockhash")
            .expect("Blockhash should always be present")
            .to_string();

        // Generate my TxInputs
        // This checks to see if the Transaction is a Coinbase TX and handles it appropriately
        let mut tx_vins: Vec<TxInput> = vec![];

        if let Some(vin) = rpc_result.get("vin").and_then(|vin| vin.as_array()) {
            if let Some(coinbase) = vin[0].get("coinbase") {
                print!("Is a Coinbase tx\n");
                let tx_input = TxInput {
                    prevout: OutPoint {
                        txid: coinbase.to_string(),
                        index: 0,
                    },
                    scriptsig: "None".to_string(),
                };
                tx_vins.push(tx_input);
            } else {
                for input in vin {
                    print!("Not a Coinbase tx\n");
                    let txid = input.get("txid").unwrap().to_string();
                    let index = input.get("vout").unwrap().as_u64().unwrap() as u32;
                    let tx_input = TxInput {
                        prevout: OutPoint { txid, index },
                        scriptsig: input.get("scriptSig").unwrap().to_string(),
                    };
                    tx_vins.push(tx_input);
                }
            }
        }

        // Generate my TxOutputs
        let mut tx_vouts: Vec<TxOutput> = vec![];

        if let Some(vout) = rpc_result.get("vout").and_then(|vout| vout.as_array()) {
            for output in vout {
                let index = output.get("n").unwrap().as_u64().unwrap() as u32;
                let value = output.get("value").unwrap().as_f64().unwrap();
                let mut scriptpubkey = String::new();
                let mut address = Some(String::new());
                if let Some(script_pub_key_obj) =
                    output.get("scriptPubKey").and_then(|v| v.as_object())
                {
                    scriptpubkey = script_pub_key_obj
                        .get("hex")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();
                    address = script_pub_key_obj
                        .get("address")
                        .and_then(|v| Some(v.to_string()))
                }

                let tx_output = TxOutput {
                    index,
                    value,
                    scriptpubkey,
                    address,
                };

                tx_vouts.push(tx_output);
            }
        }

        // Get the total transaction amount
        let mut amount: f64 = 0.0;
        if let Some(vout) = rpc_result["vout"].as_array() {
            for output in vout {
                let value = output.get("value").and_then(|v| v.as_f64()).unwrap();
                amount += value;
            }
        }

        // Build my Transaction
        let transaction = Transaction {
            txid,
            vin: tx_vins,
            vout: tx_vouts,
            blockhash,
            timestamp: blocktime,
            amount,
        };
        Ok(transaction)
    }

    async fn get_spending_transaction(&self, outpoint: &OutPoint) -> Result<Option<Transaction>> {
        todo!()
    }
    async fn get_address_transactions(&self, address: &str) -> Result<Vec<Transaction>> {
        todo!()
    }
    async fn get_transactions_batch(&self, txids: &[&str]) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }
    async fn get_spending_transactions_batch(
        &self,
        outpoints: &[&OutPoint],
    ) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }
}
