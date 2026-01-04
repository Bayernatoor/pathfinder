use crate::blockchain::Result;
use crate::models::{OutPoint, Transaction};

pub trait BlockchainDataSource {
    fn get_transaction(txid: &str) -> Result<Transaction>;
    fn get_spending_transaction(outpoint: &OutPoint) -> Result<Option<Transaction>>;
    fn get_address_transactions(address: &str) -> Result<Vec<Transaction>>;
    fn get_transactions_batch(txids: &[&str]) -> Result<Vec<Option<Transaction>>>;
    fn get_spending_transactions_batch(outpoints: &[&OutPoint])
    -> Result<Vec<Option<Transaction>>>;
}
