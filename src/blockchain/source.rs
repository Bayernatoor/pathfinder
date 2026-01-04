use crate::blockchain::Result;
use crate::models::{OutPoint, Transaction};
use async_trait::async_trait;

#[async_trait]
pub trait BlockchainDataSource {
    async fn get_transaction(&self, txid: &str) -> Result<Transaction>;
    async fn get_spending_transaction(&self, outpoint: &OutPoint) -> Result<Option<Transaction>>;
    async fn get_address_transactions(&self, address: &str) -> Result<Vec<Transaction>>;
    async fn get_transactions_batch(&self, txids: &[&str]) -> Result<Vec<Option<Transaction>>>;
    async fn get_spending_transactions_batch(
        &self,
        outpoints: &[&OutPoint],
    ) -> Result<Vec<Option<Transaction>>>;
}
