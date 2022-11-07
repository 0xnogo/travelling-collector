use std::error::Error;

use ethers::prelude::*;
use futures::future;
use std::env;

const BALANCE_THRESHOLD: u8 = 1; // in eth

// get balance of contract
async fn get_balance_address(
    ws_provider: &Provider<Ws>,
    from: H160,
    block_number: U64,
) -> Result<(H160, U256), Box<dyn Error>> {
    let block_id = BlockId::Number(BlockNumber::Number(block_number));
    let balance = ws_provider.get_balance(from, Some(block_id)).await.unwrap();

    Ok((from, balance))
}

async fn get_transactions(
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

fn filter_contract_creation(transactions: Vec<Option<Transaction>>) -> Vec<Transaction> {
    transactions
        .into_iter()
        .filter(|t| t.is_some())
        .map(|t| t.unwrap())
        .filter(|t| t.to.is_none())
        .collect()
}

async fn get_contract_addresses(
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
async fn get_all_tx_from_block(
    ws_provider: &Provider<Ws>,
    block_hash: H256,
) -> Result<Vec<TxHash>, Box<dyn Error>> {
    let block = ws_provider.get_block(block_hash).await?.unwrap();
    Ok(block.transactions)
}

async fn get_balances(
    ws_provider: &Provider<Ws>,
    addresses: Vec<H160>,
    block_number: U64,
) -> Vec<(H160, U256)> {
    future::try_join_all(
        addresses
            .iter()
            .map(|addr| get_balance_address(&ws_provider, *addr, block_number)),
    )
    .await
    .unwrap()
}

fn filter_address_on_balance(
    addresses_to_balance: Vec<(H160, U256)>,
    balance_threshold: U256,
) -> Vec<(H160, U256)> {
    addresses_to_balance
        .into_iter()
        .filter(|(_, balance)| balance >= &balance_threshold)
        .collect()
}

async fn analyze_block(ws_provider: &Provider<Ws>, block: Block<H256>, latest_block: U64) {
    println!("---- Checking block hash: {:?} ----", block.hash.unwrap());
    let transactions_fetched = get_all_tx_from_block(&ws_provider, block.hash.unwrap())
        .await
        .unwrap();

    let transactions = get_transactions(&ws_provider, transactions_fetched).await;

    // filter only creation tx (to is null)
    let contract_creation_transactions: Vec<Transaction> = filter_contract_creation(transactions);

    println!(
        "Number of smart contract creation {}",
        contract_creation_transactions.len()
    );

    // get the contract address
    let contract_address =
        get_contract_addresses(ws_provider, contract_creation_transactions).await;

    println!("Smart contracts: {:?}", contract_address);
    let balances = get_balances(&ws_provider, contract_address, latest_block).await;

    println!("Balances: {:?}", balances);

    let filtered_balances: Vec<(H160, U256)> = filter_address_on_balance(
        balances,
        ethers::utils::parse_ether(BALANCE_THRESHOLD).unwrap(),
    );

    println!("Balances with balance >= 1 eth: {:?}", filtered_balances);
}

#[allow(dead_code)]
// listening to block creation
async fn monitoring_blocks(ws_provider: &Provider<Ws>, block_number: U64) -> Option<()> {
    let mut block_stream = ws_provider.subscribe_blocks().await.unwrap();

    println!("---------- MONITORING NEW BLOCKS ----------");
    while let Some(block) = block_stream.next().await {
        analyze_block(&ws_provider, block, block_number).await;
    }

    Some(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input_block: u64 = args[1].parse().unwrap(); // 15900000
    let backwards: bool = match &args[2][..] {
        "true" => true,
        "false" => false,
        _ => true,
    }; // true

    let ws_provider = Provider::<Ws>::connect(env::var("RPC_WS_ENDPOINT").unwrap())
        .await
        .unwrap();

    let latest_block_number = ws_provider.get_block_number().await.unwrap();

    if backwards {
        loop {
            let block = ws_provider.get_block(input_block).await.unwrap().unwrap();
            println!("Perform logic on {} block", input_block);
            analyze_block(&ws_provider, block, latest_block_number).await;
            if input_block == 0 {
                break;
            }
            input_block -= 1;
        }
    }

    // monitoring_blocks(&ws_provider).await;
}
