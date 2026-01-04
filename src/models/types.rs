#[derive(Debug, Clone)]
pub struct OutPoint {
    txid: String,
    index: u32,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    txid: String,
    vin: Vec<TxInput>,
    vout: Vec<TxOutput>,
    blockheight: u32,
    timestamp: u64,
    amount: u64,
}

#[derive(Debug, Clone)]
pub struct TxInput {
    previous_outpoint: OutPoint,
    scriptsig: String,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    index: u32,
    value: u64,
    scriptpubkey: String,
    address: Option<String>,
}
