use crate::*;

use std::collections::HashMap;

#[near(serializers = [borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct LockInfo {
    pub locked_balance: U128,
    pub unlock_time_sec: u32,
}

impl LockInfo {
    pub fn append_lock(&mut self, amount: U128, unlock_time_sec: u32) {
        require!(
            self.unlock_time_sec <= unlock_time_sec
                && nano_to_sec(env::block_timestamp()) < unlock_time_sec,
            "Invalid unlock_time_sec"
        );
        self.locked_balance = U128(self.locked_balance.0 + amount.0);
        self.unlock_time_sec = unlock_time_sec;
    }
}

#[near(serializers = [borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct Account {
    pub account_id: AccountId,
    pub locked_tokens: HashMap<String, LockInfo>,
}

#[near(serializers = [borsh])]
pub enum VAccount {
    Current(Account),
}

impl From<VAccount> for Account {
    fn from(v: VAccount) -> Self {
        match v {
            VAccount::Current(c) => c,
        }
    }
}

impl From<Account> for VAccount {
    fn from(c: Account) -> Self {
        VAccount::Current(c)
    }
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            account_id: account_id.clone(),
            locked_tokens: HashMap::new(),
        }
    }

    pub fn add_lock(&mut self, token_id: &String, amount: U128, unlock_time_sec: u32) {
        require!(
            self.locked_tokens.len() < MAX_LOCK_NUM,
            "Exceed MAX_LOCK_NUM"
        );
        require!(
            nano_to_sec(env::block_timestamp()) < unlock_time_sec,
            "Invalid unlock_time_sec"
        );
        self.locked_tokens.insert(
            token_id.clone(),
            LockInfo {
                locked_balance: amount,
                unlock_time_sec,
            },
        );
    }
}

impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.data().accounts.get(account_id).map(|o| o.into())
    }

    pub fn internal_unwrap_account(&self, account_id: &AccountId) -> Account {
        self.internal_get_account(account_id)
            .expect("ACCOUNT NOT REGISTERED")
    }

    pub fn internal_set_account(&mut self, account_id: &AccountId, account: Account) {
        self.data_mut().accounts.insert(account_id, &account.into());
    }
}

#[near]
impl Contract {
    pub fn withdraw(&mut self, token_id: String, amount: Option<U128>) {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);

        if let Some(mut lock_info) = account.locked_tokens.remove(&token_id) {
            require!(
                lock_info.unlock_time_sec <= nano_to_sec(env::block_timestamp()),
                "Token still locked"
            );
            let amount = amount.unwrap_or(lock_info.locked_balance);
            lock_info.locked_balance = U128(
                lock_info
                    .locked_balance
                    .0
                    .checked_sub(amount.0)
                    .expect("Lock balance not enough"),
            );
            if lock_info.locked_balance.0 > 0 {
                account.locked_tokens.insert(token_id.clone(), lock_info);
            }
            self.internal_set_account(&account_id, account);
            self.transfer_token(&account_id, token_id.clone(), amount);
            Event::WithdrawStarted {
                account_id: &account_id,
                token_id: &token_id,
                amount: &amount,
            }
            .emit();
        } else {
            env::panic_str("Invalid token_id");
        }
    }

    #[private]
    pub fn after_token_transfer(
        &mut self,
        account_id: AccountId,
        token_id: String,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            if let Some(mut account) = self.internal_get_account(&account_id) {
                if let Some(lock_info) = account.locked_tokens.get_mut(&token_id.to_string()) {
                    lock_info.locked_balance = U128(lock_info.locked_balance.0 + amount.0);
                } else {
                    account.locked_tokens.insert(
                        token_id.clone(),
                        LockInfo {
                            locked_balance: amount,
                            unlock_time_sec: nano_to_sec(env::block_timestamp()),
                        },
                    );
                }
                self.internal_set_account(&account_id, account);
                Event::WithdrawFailed {
                    account_id: &account_id,
                    token_id: &token_id,
                    amount: &amount,
                }
                .emit();
            } else {
                Event::WithdrawLostfound {
                    account_id: &account_id,
                    token_id: &token_id,
                    amount: &amount,
                }
                .emit();
            }
        } else {
            Event::WithdrawSucceeded {
                account_id: &account_id,
                token_id: &token_id,
                amount: &amount,
            }
            .emit();
            
        }
        promise_success
    }
}

impl Contract {
    pub fn transfer_token(&self, account_id: &AccountId, token_id: String, amount: U128) {
        let (contract_id, mft_token_id) = parse_token_id(&token_id);
        if let Some(mft_token_id) = mft_token_id {
            ext_multi_fungible_token::ext(contract_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(GAS_FOR_TOKEN_TRANSFER)
                .mft_transfer(mft_token_id, account_id.clone(), amount, None)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_AFTER_TOKEN_TRANSFER)
                        .after_token_transfer(account_id.clone(), token_id.clone(), amount),
                )
        } else {
            ext_fungible_token::ext(contract_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(GAS_FOR_TOKEN_TRANSFER)
                .ft_transfer(account_id.clone(), amount, None)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_AFTER_TOKEN_TRANSFER)
                        .after_token_transfer(account_id.clone(), token_id.to_string(), amount),
                )
        };
    }
}
