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

        let transactions = get_transactions(&ws_provider, transactions_fetched).await;

        // filter only creation tx (to is null)
        let contract_creation_transactions: Vec<Transaction> =
            filter_contract_creation(transactions);

        println!(
            "Number of smart contract creation {}",
            contract_creation_transactions.len()
        );

        // get the contract address
        let contract_address =
            get_contract_addresses(ws_provider, contract_creation_transactions).await;

        println!("Smart contracts: {:?}", contract_address);
        let balances = get_balances(&ws_provider, contract_address, block.hash.unwrap()).await;

        println!("Balances: {:?}", balances);

        let filtered_balances: Vec<(H160, U256)> =
            filter_address_on_balance(balances, BALANCE_THRESHOLD);

        println!("Balances with balance >= 0 eth: {:?}", filtered_balances);
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

pub async fn get_transactions(
    ws_provider: &Provider<Ws>,
    transactions_hashes: Vec<H256>,
) -> Vec<Option<Transaction>> {
    let transactions = future::try_join_all(
        transactions_hashes
            .iter()
            .map(|tx_hash| ws_provider.get_transaction(*tx_hash)),
    )
    .await
    .unwrap();

    transactions
}

pub fn filter_contract_creation(transactions: Vec<Option<Transaction>>) -> Vec<Transaction> {
    transactions
        .into_iter()
        .filter(|t| t.is_some())
        .map(|t| t.unwrap())
        .filter(|t| t.to.is_none())
        .collect()
}

pub async fn get_contract_addresses(
    ws_provider: &Provider<Ws>,
    transactions: Vec<Transaction>,
) -> Vec<H160> {
    let tx_receipts = future::try_join_all(
        transactions
            .iter()
            .map(|tx| ws_provider.get_transaction_receipt(tx.hash)),
    )
    .await
    .unwrap();

    tx_receipts
        .into_iter()
        .filter(|tx_receipts| tx_receipts.is_some())
        .map(|tx_receipts| tx_receipts.unwrap().contract_address.unwrap())
        .collect()
}

// getting a list of all tx from a specific block
pub async fn get_all_tx_from_block(
    ws_provider: &Provider<Ws>,
    block_hash: H256,
) -> Result<Vec<TxHash>, Box<dyn Error>> {
    let block = ws_provider.get_block(block_hash).await?.unwrap();
    Ok(block.transactions)
}

pub async fn get_balances(
    ws_provider: &Provider<Ws>,
    addresses: Vec<H160>,
    block: H256,
) -> Vec<(H160, U256)> {
    future::try_join_all(
        addresses
            .iter()
            .map(|addr| get_balance_address(&ws_provider, *addr, block)),
    )
    .await
    .unwrap()
}

pub fn filter_address_on_balance(
    addresses_to_balance: Vec<(H160, U256)>,
    balance_threshold: u8,
) -> Vec<(H160, U256)> {
    addresses_to_balance
        .into_iter()
        .filter(|(_, balance)| balance >= &U256::from(balance_threshold))
        .collect()
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
