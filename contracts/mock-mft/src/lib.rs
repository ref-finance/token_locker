use near_sdk::{
    borsh::BorshSerialize, collections::LookupMap, json_types::U128, near, near_bindgen, require,
    AccountId, BorshStorageKey, PanicOnDefault,
};

mod mft;

pub type Balance = u128;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub(crate) enum StorageKey {
    Tokens,
    Accounts { inner_id: String },
}

#[near(serializers = [borsh])]
pub struct Token {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
}

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    tokens: LookupMap<String, Token>,
    name: String,
    symbol: String,
    decimals: u8,
}

#[near]
impl Contract {
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        Self {
            tokens: LookupMap::new(StorageKey::Tokens),
            name,
            symbol,
            decimals,
        }
    }

    pub fn mint(&mut self, inner_id: String, account_id: AccountId, amount: U128) {
        let mut token = self.tokens.get(&inner_id).unwrap_or_else(|| Token {
            accounts: LookupMap::new(StorageKey::Accounts {
                inner_id: inner_id.clone(),
            }),
            total_supply: 0,
        });

        let new_amount = amount.0 + token.accounts.get(&account_id).unwrap_or_default();
        token.accounts.insert(&account_id, &(new_amount));
        token.total_supply += amount.0;

        self.tokens.insert(&inner_id, &token);
    }

    pub fn burn(&mut self, inner_id: String, account_id: AccountId, amount: U128) {
        let mut token = self.tokens.get(&inner_id).unwrap_or_else(|| Token {
            accounts: LookupMap::new(StorageKey::Accounts {
                inner_id: inner_id.clone(),
            }),
            total_supply: 0,
        });
        let total = token.accounts.get(&account_id).unwrap_or_default();
        require!(total >= amount.0, "NOT_ENOUGH_BALANCE");

        token.accounts.insert(&account_id, &(total - amount.0));
        token.total_supply -= amount.0;

        self.tokens.insert(&inner_id, &token);
    }
}
