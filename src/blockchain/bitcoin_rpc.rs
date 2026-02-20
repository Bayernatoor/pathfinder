use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};
use async_trait::async_trait;
use bitcoin::consensus::encode::deserialize_hex;
use serde_json::{Value, json};

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
        // print!("response {:?}", response);

        // convert response to serde_json value
        let json_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;
        // print!("json_response from rpc call {:?}", json_response);

        if let Some(rpc_error) = json_response.get("error").and_then(|e| e.as_object())
            && !rpc_error.is_empty() {
                let code = rpc_error.get("code").and_then(|c| c.as_i64()).unwrap_or(0);

                let message = rpc_error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown RPC Error");

                // Map JSON RPC errors to BlockchainError
                // Codes are specific to transaction related errors
                match code {
                    -5 | -20 => return Err(BlockchainError::NotFound(message.to_string())),
                    -8 | -22 => return Err(BlockchainError::InvalidInput(message.to_string())),
                    -32603 => return Err(BlockchainError::Other(message.to_string())),
                    _ => {
                        return Err(BlockchainError::Other(format!(
                            "RPC error {code}: {message}"
                        )));
                    }
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
    async fn get_transaction(&self, txid: bitcoin::Txid) -> Result<bitcoin::Transaction> {
        let rpc_result: Value = self
            .rpc_call("getrawtransaction", vec![json!(txid), json!(1)])
            .await?;

        // Extract hex string
        let hex_str = rpc_result
            .get("hex")
            .and_then(|h| h.as_str())
            .ok_or_else(|| {
                BlockchainError::DataInconsistency(
                    "RPC response is missing 'hex' field or type is invalid".to_string(),
                )
            })?;

        // Deserialize hex value into a bitcoin::Transaction
        let transaction = deserialize_hex(hex_str).map_err(|e| {
            BlockchainError::DataInconsistency(format!(
                "Failed to deserialize Hex {:?}, for Txid {:?}: {:?}",
                hex_str, txid, e
            ))
        })?;

        Ok(transaction)
    }
    async fn get_spending_transaction(
        &self,
        _outpoint: bitcoin::OutPoint,
    ) -> Result<Option<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_address_transactions(
        &self,
        _address: bitcoin::Address,
    ) -> Result<Vec<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_transactions_batch(
        &self,
        _txids: &[bitcoin::Txid],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
    async fn get_spending_transactions_batch(
        &self,
        _outpoints: &[bitcoin::OutPoint],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
}
