use crate::*;
use near_sdk::ext_contract;

pub const GAS_FOR_TOKEN_TRANSFER: Gas = Gas::from_tgas(20);
pub const GAS_FOR_AFTER_TOKEN_TRANSFER: Gas = Gas::from_tgas(10);
pub const GAS_FOR_AFTER_TOKEN_BURN: Gas = Gas::from_tgas(10);

pub const MFT_TAG: &str = "@";
pub const MAX_LOCK_NUM: usize = 64;

pub fn nano_to_sec(nano: u64) -> u32 {
    (nano / 10u64.pow(9)) as u32
}

pub fn generate_mft_token_id(token_id: String) -> String {
    format!("{}{}{}", env::predecessor_account_id(), MFT_TAG, token_id)
}

pub fn parse_token_id(token_id: &String) -> (AccountId, Option<String>) {
    if let Some((contract_id, mft_token_id)) = token_id.split_once(MFT_TAG) {
        (contract_id.parse().unwrap(), Some(mft_token_id.to_string()))
    } else {
        (token_id.parse().unwrap(), None)
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
