use std::error::Error;

use ethers::prelude::*;
use futures::future;

const BALANCE_THRESHOLD: u8 = 0;

// listening to block creation
pub async fn monitoring_blocks(ws_provider: &Provider<Ws>) -> Option<()> {
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
        let contract_creation_transactions: Vec<Transaction> = transactions
            .into_iter()
            .filter(|t| t.is_some())
            .map(|t| t.unwrap())
            .filter(|t| t.to.is_none())
            .collect();

        println!(
            "Number of smart contract creation {}",
            contract_creation_transactions.len()
        );

        // get the contract address
        let tx_receipts = future::try_join_all(
            contract_creation_transactions
                .iter()
                .map(|tx| ws_provider.get_transaction_receipt(tx.hash)),
        )
        .await
        .unwrap();

        let contract_address: Vec<H160> = tx_receipts
            .into_iter()
            .filter(|tx_receipts| tx_receipts.is_some())
            .map(|tx_receipts| tx_receipts.unwrap().contract_address.unwrap())
            .collect();

        println!("Smart contracts: {:?}", contract_address);
        let balances = future::try_join_all(
            contract_address
                .iter()
                .map(|addr| get_balance_address(&ws_provider, *addr, block.hash.unwrap())),
        )
        .await
        .unwrap();

        println!("Balances: {:?}", balances);

        let filtered_balances: Vec<(H160, U256)> = balances
            .into_iter()
            .filter(|(_, balance)| balance > &U256::from(BALANCE_THRESHOLD))
            .collect();

        println!("Balances with balance > 0 eth: {:?}", filtered_balances);
    }

    Some(())
}

// get balance of contract
pub async fn get_balance_address(
    ws_provider: &Provider<Ws>,
    from: H160,
    block: H256,
) -> Result<(H160, U256), Box<dyn Error>> {
    let block_id = BlockId::Hash(block);
    let balance = ws_provider.get_balance(from, Some(block_id)).await.unwrap();

    Ok((from, balance))
}

// getting a list of all tx from a specific block
pub async fn get_all_tx_from_block(
    ws_provider: &Provider<Ws>,
    block_hash: H256,
) -> Result<Vec<TxHash>, Box<dyn Error>> {
    let block = ws_provider.get_block(block_hash).await?.unwrap();
    Ok(block.transactions)
}

#[tokio::main]
async fn main() {
    let ws_provider =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await
            .unwrap();

    // allow going backwards
    // allow chose from where to start
    monitoring_blocks(&ws_provider).await;
}
