use ethers::types::U256;

use crate::contract::Contract;

pub const BALANCE_THRESHOLD: u8 = 1; // in eth

pub fn filter_contracts_on_balance(
    addresses_to_balance: &mut Vec<Contract>,
    balance_threshold: U256,
) {
    addresses_to_balance.retain(|contract| contract.balance >= balance_threshold);
}
