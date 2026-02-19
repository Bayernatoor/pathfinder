use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};
use async_trait::async_trait;
use bitcoin::{OutPoint, Transaction, Txid, consensus::encode::deserialize_hex};
use serde::Deserialize;
use serde_json::{Value, json};

pub struct EsploraClient {
    base_url: String,
    client: reqwest::Client,
}

impl EsploraClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn throttle(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

// Response structure for /tx/:txid/outspend/:vout
#[derive(Deserialize, Debug)]
struct OutspendResponse {
    spent: bool,
    #[serde(default)]
    txid: Option<Txid>,
    #[serde(default)]
    vin: Option<u32>,
}

#[async_trait]
impl BlockchainDataSource for EsploraClient {
    async fn get_transaction(&self, txid: bitcoin::Txid) -> Result<bitcoin::Transaction> {
        let url = format!("{}/tx/{}/hex", self.base_url, txid);

        let hex = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?
            .text()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;

        bitcoin::consensus::encode::deserialize_hex(&hex)
            .map_err(|e| BlockchainError::DataInconsistency(format!("Invalid hex: {}:", e)))
    }
    async fn get_spending_transaction(
        &self,
        outpoint: bitcoin::OutPoint,
    ) -> Result<Option<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_address_transactions(
        &self,
        address: bitcoin::Address,
    ) -> Result<Vec<bitcoin::Transaction>> {
        todo!()
    }
    async fn get_transactions_batch(
        &self,
        txids: &[bitcoin::Txid],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
    async fn get_spending_transactions_batch(
        &self,
        outpoints: &[bitcoin::OutPoint],
    ) -> Result<Vec<Option<bitcoin::Transaction>>> {
        todo!()
    }
}
