use std::{error::Error, str::FromStr};

use ethers::prelude::*;
use futures::future;

// listening to block creation
pub async fn monitoring_blocks(ws_provider: Provider<Ws>) -> Option<()> {
    let mut block_stream = ws_provider.subscribe_blocks().await.unwrap();

    println!("---------- MONITORING NEW BLOCKS ----------");
    while let Some(block) = block_stream.next().await {
        println!("---- Checking block hash: {:?} ----", block.hash.unwrap());
        let transactions_fetched = get_all_tx_from_block(&ws_provider, block.hash.unwrap())
            .await
            .unwrap();

        let transactions = future::try_join_all(
            transactions_fetched
                .iter()
                .map(|tx_hash| ws_provider.get_transaction(*tx_hash)),
        )
        .await
        .unwrap();

        // filter only creation tx (to is null)
        let finalzer: Vec<Transaction> = transactions
            .into_iter()
            .filter(|t| t.is_some())
            .map(|t| t.unwrap())
            .filter(|t| t.to.is_none())
            .collect();

        println!("Number of smart contract creation {}", finalzer.len());
        println!("Txs: {:?}", finalzer);
    }

    Some(())
}

// getting a list of all tx from a specific block
pub async fn get_all_tx_from_block(
    ws_provider: &Provider<Ws>,
    block_hash: H256,
) -> Result<Vec<TxHash>, Box<dyn Error>> {
    let block = ws_provider.get_block(block_hash).await?.unwrap();
    Ok(block.transactions)
}

// get balance of contract

#[tokio::main]
async fn main() {
    let zero_address = H160::from_str("0x0000000000000000000000000000000000000000");
    println!("{:?}", zero_address);
    let ws_provider =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await
            .unwrap();
    // allow going backwards
    // allow chose from where to start
    monitoring_blocks(ws_provider).await;
}
