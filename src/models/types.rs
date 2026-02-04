use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutPoint {
    pub txid: String,
    pub index: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub txid: String,
    pub vin: Vec<TxInput>,
    pub vout: Vec<TxOutput>,
    pub blockhash: String,
    pub timestamp: u64,
    pub amount: f64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxInput {
    pub prevout: OutPoint,
    pub scriptsig: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TxOutput {
    pub index: u32,
    pub value: f64,
    pub scriptpubkey: String,
    pub address: Option<String>,
}
