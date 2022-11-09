use std::error::Error;

use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{BlockId, BlockNumber, Transaction, H160, H256, U256, U64},
};
use futures::future;

use crate::source_code;

#[derive(Debug)]
pub struct Contract {
    pub address: H160,
    pub balance: U256,
    pub block_hash: H256,
    pub bytecode: Option<String>,
    pub verified_code: Option<Vec<(String, Vec<String>)>>,
}

#[allow(dead_code)]
impl Contract {
    pub fn new(
        address: H160,
        balance: U256,
        block_hash: H256,
        bytecode: String,
        verified_code: Vec<(String, Vec<String>)>,
    ) -> Self {
        Self {
            address,
            balance,
            block_hash,
            bytecode: Some(bytecode),
            verified_code: Some(verified_code),
        }
    }

    pub fn new_without_code(address: H160, balance: U256, block_hash: H256) -> Self {
        Self {
            address,
            balance,
            block_hash,
            bytecode: None,
            verified_code: None,
        }
    }

    pub fn set_balance(&mut self, balance: U256) {
        self.balance = balance;
    }

    pub fn set_verified_code(&mut self, code: Vec<(String, Vec<String>)>) {
        if code.len() == 0 {
            self.verified_code = None;
        }

        self.verified_code = Some(code);
    }
}

async fn get_balance_address(
    ws_provider: &Provider<Ws>,
    from: H160,
    block_number: U64,
) -> Result<(H160, U256), Box<dyn Error>> {
    let block_id = BlockId::Number(BlockNumber::Number(block_number));
    let balance = ws_provider.get_balance(from, Some(block_id)).await.unwrap();

    Ok((from, balance))
}

pub async fn get_balances(
    ws_provider: &Provider<Ws>,
    contracts: &mut Vec<Contract>,
    block_number: U64,
) {
    future::join_all(contracts.iter_mut().map(|contract| async {
        let balance = get_balance_address(&ws_provider, contract.address, block_number)
            .await
            .unwrap();
        contract.set_balance(balance.1);
    }))
    .await;
}

pub async fn get_contracts(
    ws_provider: &Provider<Ws>,
    transactions: Vec<Transaction>,
) -> Vec<Contract> {
    let tx_receipts = future::try_join_all(
        transactions
            .iter()
            .map(|tx| ws_provider.get_transaction_receipt(tx.hash)),
    )
    .await
    .unwrap();

    tx_receipts
        .into_iter()
        .filter(|tx_receipt| tx_receipt.is_some())
        .map(|tx_receipt| {
            Contract::new_without_code(
                tx_receipt.as_ref().unwrap().contract_address.unwrap(),
                U256::from(0),
                tx_receipt.as_ref().unwrap().block_hash.unwrap(),
            )
        })
        .collect()
}

pub async fn get_verified_code(contracts: &mut Vec<Contract>) {
    let smart_contracts = future::join_all(
        contracts
            .iter()
            .map(|c| source_code::get_source_code(&c.address)),
    )
    .await;

    let mut iterator = smart_contracts.iter();

    contracts.iter_mut().for_each(|c| match iterator.next() {
        Some(sc) => c.set_verified_code(sc.to_owned()),
        _ => (),
    });
}
