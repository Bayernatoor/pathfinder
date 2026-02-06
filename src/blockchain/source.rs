use crate::blockchain::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BlockchainDataSource {
    async fn get_transaction(&self, txid: bitcoin::Txid) -> Result<bitcoin::Transaction>;
    async fn get_spending_transaction(
        &self,
        outpoint: bitcoin::OutPoint,
    ) -> Result<Option<bitcoin::Transaction>>;
    async fn get_address_transactions(
        &self,
        address: bitcoin::Address,
    ) -> Result<Vec<bitcoin::Transaction>>;
    async fn get_transactions_batch(
        &self,
        txids: &[bitcoin::Txid],
    ) -> Result<Vec<Option<bitcoin::Transaction>>>;
    async fn get_spending_transactions_batch(
        &self,
        outpoints: &[bitcoin::OutPoint],
    ) -> Result<Vec<Option<bitcoin::Transaction>>>;
}
