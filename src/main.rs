use bitcoin::consensus::encode::deserialize_hex;
use pathfinder::blockchain::{BitcoinRpcClient, BlockchainError};
use serde_json::{Value, json, to_string_pretty};

#[tokio::main]
async fn main() {
    let rpc_client = BitcoinRpcClient::new(
        "http://127.0.0.1:8332".to_string(),
        "testnode".to_string(),
        "asdjio3u2o23d32dpiadas".to_string(),
    );

    let coinbase_txid = vec![
        json!("ea30577074d9a93457b2c0b87bb33ad2942dca1845ef88c5538512b27388f260"),
        json!(1),
    ];
    // let txid = vec![
    //     json!("44aa39aa88d6da9747efe4efbbcfbc0cfed390bc031a28e6679bde3b3223e9b4"),
    //     json!(1),
    // ];
    let rpc_result = rpc_client
        .rpc_call("getrawtransaction", coinbase_txid)
        .await
        .unwrap();

    // print!("result {:?}", rpc_result);

    let transaction: bitcoin::Transaction = match rpc_result.get("hex") {
        Some(hex) => deserialize_hex(&hex.as_str().unwrap()).unwrap(),
        None => panic!(),
    };
    //
}
