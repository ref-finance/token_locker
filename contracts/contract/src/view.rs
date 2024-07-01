use crate::*;

#[near(serializers = [json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct Metadata {
    owner_id: AccountId,
    current_account_num: u64,
    token_white_list: Vec<AccountId>,
    burn_account_id: Option<AccountId>,
}

#[near]
impl Contract {
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            owner_id: self.data().owner_id.clone(),
            current_account_num: self.data().accounts.len(),
            token_white_list: self.data().token_white_list.to_vec(),
            burn_account_id: self.data().burn_account_id.clone(),
        }
    }

    pub fn get_account(&self, account_id: AccountId) -> Option<Account> {
        self.internal_get_account(&account_id)
    }

    pub fn get_accounts_paged(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Account> {
        let values = self.data().accounts.values_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(values.len());
        (from_index..std::cmp::min(values.len(), from_index + limit))
            .map(|index| values.get(index).unwrap().into())
            .collect()
    }
}
