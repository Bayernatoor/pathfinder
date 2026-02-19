use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};
use async_trait::async_trait;
use bitcoin::{Block, OutPoint, Transaction, Txid};
use serde::Deserialize;

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

    // helper to prevent rate limiting on mempool.space
    async fn throttle(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// Response structure for /tx/:txid/outspend/:vout
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
            .get(&url)
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
        // protext againt mempool.space rate limiting
        self.throttle().await;

        let url = format!(
            "{}/tx/{}/outspend/{}",
            self.base_url, outpoint.txid, outpoint.vout
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;

        // Immediately check for 404, would mean transaction id does not exist
        if response.status() == 404 {
            return Err(BlockchainError::NotFound(format!(
                "Transaction {} not found",
                outpoint.txid
            )));
        }

        // Deserialize the response into our OutspendResponse Struct
        let outspend: OutspendResponse = response
            .json()
            .await
            .map_err(|e| BlockchainError::DataInconsistency(e.to_string()))?;

        if !outspend.spent {
            return Ok(None);
        }

        match outspend.txid {
            // Fetch the full transaction data using the txid we just found
            Some(txid) => self.get_transaction(txid).await.map(Some),
            None => Err(BlockchainError::DataInconsistency(
                "Outspend marked spent but no txid returned".to_string(),
            )),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_esplora_outspend() {
        let client = EsploraClient::new("https://mempool.space/api".to_string());

        let outpoint = bitcoin::OutPoint::new(
            "a260cc34b85217c01f0f8a14d0213c9536952592f3022a0f414e7485a4b016ec"
                .parse()
                .unwrap(),
            27,
        );

        let result = client.get_spending_transaction(outpoint).await;
        assert!(result.is_ok());

        let tx = result.unwrap();
        assert!(tx.is_some());
        assert_eq!(
            tx.unwrap().compute_txid().to_string(),
            "59c738932e6d039e76c7eb9da8d6e36def35ee9ed092fb678f6e469d1a160312"
        );
    }
}
