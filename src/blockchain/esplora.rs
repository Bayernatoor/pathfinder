use crate::blockchain::{BlockchainDataSource, BlockchainError, Result};
use async_trait::async_trait;
use bitcoin::{Address, OutPoint, Transaction, Txid};
use serde::Deserialize;

/// Esplora HTTP client used to retrieve blockchain data.
///
/// This client connects to whichever Esplora-compatible API endpoint you want (currently uses
/// mempool.space) to fetch transaction data and spend information.
///
/// # Rate Limiting
/// Includes a helper to avoid overwhelming public API (100ms delay).
/// This is important since UTXO tracing can result in hundreds of sequential requests.
///
/// Ideally you should run your own esplora instance.
pub struct EsploraClient {
    base_url: String,
    client: reqwest::Client,
}

impl EsploraClient {
    /// Creates a new Esplora client
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Esplora instance (e.g. "https://mempool.space/api")
    ///
    /// Automatically trims trailing slashes to ensure proper URL construction.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Helper that applies a small delay to prevent rate limiting
    ///
    /// 100ms which limits us to 10 req/sec, ideally preventing rate limits
    ///
    /// NOTE:For testing only - Should run own esplora indexer for better reliably
    async fn throttle(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// Response from Esplora's outspend endpoint.
///
/// Indicates whether a specific output (OutPoint) has been spent,
/// and if so, which transaction spent it.
#[derive(Deserialize, Debug)]
struct OutspendResponse {
    spent: bool,
    /// Txid of the spending transaction, will only be present if spent == true
    #[serde(default)]
    txid: Option<Txid>,
    /// Input index of the spending transaction, will only be present if spent == true
    #[serde(default)]
    _vin: Option<u32>,
}

#[async_trait]
impl BlockchainDataSource for EsploraClient {
    /// Fetches a transaction by its txid.
    ///
    /// Uses the `/tx/{txid}/hex` endpoint for efficient raw bytes retrieval,
    /// then deserializes into a `bitcoin::Transaction`.
    ///
    /// # Errors
    /// - `NetworkFailure` - HTTP request failed
    /// - `NotFound` - Transaction not found (404)
    /// - `DataInconsistency` - Invalid hex or deserialization failure
    async fn get_transaction(&self, txid: Txid) -> Result<Transaction> {
        let url = format!("{}/tx/{}/hex", self.base_url, txid);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;

        // Immediately check for 404 as it would mean transaction id does not exist
        if response.status() == 404 {
            return Err(BlockchainError::NotFound(format!(
                "Transaction {} not found",
                txid
            )));
        }

        // handle any other 4**/5** errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read body".to_string());
            return Err(BlockchainError::NetworkFailure(format!(
                "HTTP {} for {}: {}",
                status, url, body
            )));
        }

        let hex = response
            .text()
            .await
            .map_err(|e| BlockchainError::NetworkFailure(e.to_string()))?;

        bitcoin::consensus::encode::deserialize_hex(&hex)
            .map_err(|e| BlockchainError::DataInconsistency(format!("Invalid hex: {}", e)))
    }

    /// Finds the transaction that spends a specific OutPoint.
    ///
    /// Uses the `/tx/{txid}/outspend/{vout}` endpoint to check if an output
    /// has been spent. If spent, fetches the full spending transaction.
    ///
    /// # Arguments
    /// * `outpoint` - The UTXO to check (txid + output index)
    ///
    /// # Returns
    /// - `Ok(Some(tx))` - The transaction that spent this outpoint
    /// - `Ok(None)` - The outpoint is still unspent
    /// - `Err(NotFound)` - The original transaction doesn't exist
    /// - `Err(DataInconsistency)` - API returned invalid data
    async fn get_spending_transaction(&self, outpoint: OutPoint) -> Result<Option<Transaction>> {
        // protect against mempool.space rate limiting
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

        // Immediately check for 404 as it would mean transaction id does not exist
        if response.status() == 404 {
            return Err(BlockchainError::NotFound(format!(
                "Transaction {} not found",
                outpoint.txid
            )));
        }

        // handle any other 4**/5** errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read body".to_string());

            return Err(BlockchainError::NetworkFailure(format!(
                "HTTP {} for {}: {}",
                status, url, body
            )));
        }

        // Deserialize the response into our OutspendResponse Struct
        let outspend: OutspendResponse = response
            .json()
            .await
            .map_err(|e| BlockchainError::DataInconsistency(e.to_string()))?;

        // if output is not spent return None Immediately
        if !outspend.spent {
            return Ok(None);
        }

        // If spent, fetch the full transaction data using the txid that was found
        match outspend.txid {
            Some(txid) => self.get_transaction(txid).await.map(Some),
            None => Err(BlockchainError::DataInconsistency(
                "Outspend marked spent but no txid returned".to_string(),
            )),
        }
    }

    async fn get_address_transactions(&self, _address: Address) -> Result<Vec<Transaction>> {
        todo!()
    }

    async fn get_transactions_batch(&self, _txids: &[Txid]) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }

    async fn get_spending_transactions_batch(
        &self,
        _outpoints: &[OutPoint],
    ) -> Result<Vec<Option<Transaction>>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uses real network and could fail for many reasons. Will improve in the future.
    #[tokio::test]
    #[ignore] // Hits real API, don't want this running in CI yet.
    async fn test_esplora_outspend() {
        let client = EsploraClient::new("https://mempool.space/api".to_string());

        let outpoint = OutPoint::new(
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
