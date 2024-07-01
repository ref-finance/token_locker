use near_sdk::{
    assert_one_yocto, borsh::BorshSerialize, collections::{UnorderedMap, UnorderedSet}, env, is_promise_success,
    json_types::U128, log, near, require, serde_json::{self, json}, AccountId, BorshStorageKey,
    Gas, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

mod account;
mod event;
mod storage;
mod token_receiver;
mod legacy;
mod upgrade;
mod utils;
mod view;
pub use account::*;
pub use legacy::*;
pub use event::*;
pub use storage::*;
pub use token_receiver::*;
pub use utils::*;
pub use view::*;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Accounts,
    WhiteList,
}

#[near(serializers = [borsh])]
pub struct ContractData {
    owner_id: AccountId,
    accounts: UnorderedMap<AccountId, VAccount>,
    token_white_list: UnorderedSet<AccountId>,
    burn_account_id: Option<AccountId>,
}

#[near(serializers = [borsh])]
pub enum VersionedContractData {
    V1000(ContractDataV1000),
    V1001(ContractData),
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    data: VersionedContractData,
}

#[near]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            data: VersionedContractData::V1001(ContractData {
                owner_id,
                accounts: UnorderedMap::new(StorageKey::Accounts),
                token_white_list: UnorderedSet::new(StorageKey::WhiteList),
                burn_account_id: None
            }),
        }
    }

    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.data_mut().owner_id = owner_id;
    }

    #[payable]
    pub fn set_burn_account_id(&mut self, burn_account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.data_mut().burn_account_id = Some(burn_account_id);
    }

    #[payable]
    pub fn extend_token_white_list(&mut self, token_ids: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for token_id in token_ids {
            self.data_mut().token_white_list.insert(&token_id);
        }
    }

    #[payable]
    pub fn remove_token_white_list(&mut self, token_ids: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for token_id in token_ids {
            let is_success = self.data_mut().token_white_list.remove(&token_id);
            assert!(is_success, "Invalid token id");
        }
    }
}

impl Contract {
    #[allow(unreachable_patterns)]
    fn data(&self) -> &ContractData {
        match &self.data {
            VersionedContractData::V1001(data) => data,
            _ => unimplemented!(),
        }
    }

    #[allow(unreachable_patterns)]
    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VersionedContractData::V1001(data) => data,
            _ => unimplemented!(),
        }
    }

    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.data().owner_id,
            "NOT ALLOWED"
        );
    }

    pub fn assert_white_list_token(&self, token_id: &AccountId) {
        require!(
            self.data().token_white_list.contains(token_id),
            "NOT WHITE LIST TOKEN"
        );
    }
}
