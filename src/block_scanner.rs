use std::error::Error;

use crate::contract;
use crate::helper;
use crate::vulnerability::{Detector, Vulnerability};
use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{Block, Transaction, TxHash, H256, U64},
};
use futures::{future, StreamExt};
use strum::IntoEnumIterator;

// listening to block creation
pub async fn monitoring_blocks(ws_provider: &Provider<Ws>, block_number: U64) -> Option<()> {
    let mut block_stream = ws_provider.subscribe_blocks().await.unwrap();

    println!("---------- MONITORING NEW BLOCKS ----------");
    while let Some(block) = block_stream.next().await {
        analyze_block(&ws_provider, block, block_number).await;
    }

    Some(())
}

pub async fn analyze_block(ws_provider: &Provider<Ws>, block: Block<H256>, latest_block: U64) {
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
    let mut contracts = contract::get_contracts(ws_provider, contract_creation_transactions).await;

    println!("Smart contracts: {:?}", &contracts);
    contract::get_balances(&ws_provider, &mut contracts, latest_block).await;

    println!("Balances: {:?}", &contracts);

    helper::filter_contracts_on_balance(
        &mut contracts,
        ethers::utils::parse_ether(helper::BALANCE_THRESHOLD).unwrap(),
    );
    println!("filtered: {:?}", &contracts);
    contract::get_verified_code(&mut contracts).await;

    println!(
        "Balances with balance >= {}eth eth: {:?}",
        helper::BALANCE_THRESHOLD,
        &contracts
    );

    // checking for vulnerability
    let mut reports = vec![];
    contracts.iter().for_each(|contract| {
        for vul in Vulnerability::iter() {
            reports.push(vul.detect(contract));
        }
    });

    println!("Vulnerability analysis done. Reports: {:?}", &reports);
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

// getting a list of all tx from a specific block
async fn get_all_tx_from_block(
    ws_provider: &Provider<Ws>,
    block_hash: H256,
) -> Result<Vec<TxHash>, Box<dyn Error>> {
    let block = ws_provider.get_block(block_hash).await?.unwrap();
    Ok(block.transactions)
}
