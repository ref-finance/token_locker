use crate::*;
use near_sdk::ext_contract;

pub const GAS_FOR_TOKEN_TRANSFER: Gas = Gas::from_tgas(20);
pub const GAS_FOR_AFTER_TOKEN_TRANSFER: Gas = Gas::from_tgas(10);

pub const MFT_TAG: &str = "@";
pub const MAX_LOCK_NUM: usize = 64;

pub fn nano_to_sec(nano: u64) -> u32 {
    (nano / 10u64.pow(9)) as u32
}

pub fn generate_mft_token_id(token_id: String) -> String {
    format!("{}{}{}", env::predecessor_account_id(), MFT_TAG, token_id)
}

pub fn parse_token_id(token_id: &String) -> (AccountId, Option<String>) {
    let v: Vec<&str> = token_id.split(MFT_TAG).collect();
    if v.len() == 1 {
        let contract_id: AccountId = v[0].parse().unwrap();
        (contract_id, None)
    } else if v.len() == 2 {
        let contract_id: AccountId = v[0].parse().unwrap();
        (contract_id, Some(v[1].to_string()))
    } else {
        env::panic_str("INVALID TOKEN ID");
    }
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_multi_fungible_token)]
pub trait MultiFungibleToken {
    fn mft_transfer(
        &mut self,
        token_id: String,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_token_transfer(
        &mut self,
        account_id: AccountId,
        token_id: String,
        amount: U128,
    ) -> bool;
}
