use near_sdk::{
    assert_one_yocto, borsh::BorshSerialize, collections::UnorderedMap, env, is_promise_success,
    json_types::U128, log, near, require, serde_json::{self, json}, AccountId, BorshStorageKey,
    Gas, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

mod account;
mod event;
mod storage;
mod token_receiver;
mod upgrade;
mod utils;
mod view;
pub use account::*;
pub use event::*;
pub use storage::*;
pub use token_receiver::*;
pub use utils::*;
pub use view::*;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Accounts,
}

#[near(serializers = [borsh])]
pub struct ContractData {
    owner_id: AccountId,
    accounts: UnorderedMap<AccountId, VAccount>,
}

#[near(serializers = [borsh])]
pub enum VersionedContractData {
    V1000(ContractData),
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
            data: VersionedContractData::V1000(ContractData {
                owner_id,
                accounts: UnorderedMap::new(StorageKey::Accounts),
            }),
        }
    }

    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.data_mut().owner_id = owner_id;
    }

}

impl Contract {
    #[allow(unreachable_patterns)]
    fn data(&self) -> &ContractData {
        match &self.data {
            VersionedContractData::V1000(data) => data,
            _ => unimplemented!(),
        }
    }

    #[allow(unreachable_patterns)]
    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VersionedContractData::V1000(data) => data,
            _ => unimplemented!(),
        }
    }

    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.data().owner_id,
            "NOT ALLOWED"
        );
    }
}
