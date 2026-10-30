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
    #[payable]
    pub fn withdraw(&mut self, token_id: String, amount: Option<U128>) {
        assert_one_yocto();
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

    #[payable]
    pub fn burn(&mut self, token_id: String, amount: Option<U128>) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&account_id);

        if let Some(mut lock_info) = account.locked_tokens.remove(&token_id) {
            let unlock_time_sec = lock_info.unlock_time_sec;
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
            self.burn_token(&account_id, token_id.clone(), amount, unlock_time_sec);
            Event::BurnStarted {
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

    #[private]
    pub fn after_token_burn(
        &mut self,
        account_id: AccountId,
        token_id: String,
        amount: U128,
        unlock_time_sec: u32,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            if let Some(mut account) = self.internal_get_account(&account_id) {
                if let Some(lock_info) = account.locked_tokens.get_mut(&token_id.to_string()) {
                    lock_info.locked_balance = U128(lock_info.locked_balance.0 + amount.0);
                    lock_info.unlock_time_sec = lock_info.unlock_time_sec.max(unlock_time_sec);
                } else {
                    account.locked_tokens.insert(
                        token_id.clone(),
                        LockInfo {
                            locked_balance: amount,
                            unlock_time_sec,
                        },
                    );
                }
                self.internal_set_account(&account_id, account);
                Event::BurnFailed {
                    account_id: &account_id,
                    token_id: &token_id,
                    amount: &amount,
                }
                .emit();
            } else {
                Event::BurnLostfound {
                    account_id: &account_id,
                    token_id: &token_id,
                    amount: &amount,
                }
                .emit();
            }
        } else {
            Event::BurnSucceeded {
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

    pub fn burn_token(
        &self,
        account_id: &AccountId,
        token_id: String,
        amount: U128,
        unlock_time_sec: u32,
    ) {
        let burn_account_id = self
            .data()
            .burn_account_id
            .clone()
            .expect("Missing burn_account_id");
        let (contract_id, mft_token_id) = parse_token_id(&token_id);
        if let Some(mft_token_id) = mft_token_id {
            ext_multi_fungible_token::ext(contract_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(GAS_FOR_TOKEN_TRANSFER)
                .mft_transfer(mft_token_id, burn_account_id, amount, None)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_AFTER_TOKEN_BURN)
                        .after_token_burn(
                            account_id.clone(),
                            token_id.clone(),
                            amount,
                            unlock_time_sec,
                        ),
                )
        } else {
            ext_fungible_token::ext(contract_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(GAS_FOR_TOKEN_TRANSFER)
                .ft_transfer(burn_account_id, amount, None)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_AFTER_TOKEN_BURN)
                        .after_token_burn(
                            account_id.clone(),
                            token_id.to_string(),
                            amount,
                            unlock_time_sec,
                        ),
                )
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{
        mock::MockAction,
        test_utils::{get_created_receipts, VMContextBuilder},
        testing_env, PromiseResult,
    };

    const NOW_SEC: u32 = 1_700_000_000;
    const DAY_SEC: u32 = 86_400;

    fn begin_full_burn(token_id: &str, unlock_time_sec: u32) -> (Contract, AccountId, String, u32) {
        let account_id: AccountId = "alice.near".parse().unwrap();
        let mut context = VMContextBuilder::new();
        context
            .current_account_id("locker.near".parse().unwrap())
            .predecessor_account_id(account_id.clone())
            .attached_deposit(NearToken::from_yoctonear(1))
            .block_timestamp(u64::from(NOW_SEC) * 1_000_000_000);
        testing_env!(context.build());

        let mut contract = Contract::new(account_id.clone());
        contract.set_burn_account_id("burn.near".parse().unwrap());
        let token_id = token_id.to_string();
        let mut account = Account::new(&account_id);
        account.add_lock(&token_id, U128(100), unlock_time_sec);
        contract.internal_set_account(&account_id, account);

        contract.burn(token_id.clone(), None);
        assert!(contract
            .internal_unwrap_account(&account_id)
            .locked_tokens
            .is_empty());

        let callback_args = get_created_receipts()
            .into_iter()
            .flat_map(|receipt| receipt.actions)
            .find_map(|action| match action {
                MockAction::FunctionCallWeight {
                    method_name, args, ..
                } if method_name.as_slice() == b"after_token_burn" => Some(args),
                _ => None,
            })
            .expect("burn callback receipt");
        let callback_args: serde_json::Value = serde_json::from_slice(&callback_args).unwrap();
        let callback_expiry = callback_args["unlock_time_sec"].as_u64().unwrap() as u32;
        assert_eq!(callback_expiry, unlock_time_sec);

        (contract, account_id, token_id, callback_expiry)
    }

    fn finish_failed_burn(
        contract: &mut Contract,
        account_id: &AccountId,
        token_id: &str,
        unlock_time_sec: u32,
    ) {
        let mut context = VMContextBuilder::new();
        context
            .current_account_id("locker.near".parse().unwrap())
            .predecessor_account_id("locker.near".parse().unwrap())
            .block_timestamp(u64::from(NOW_SEC + 1) * 1_000_000_000);
        testing_env!(
            context.build(),
            near_sdk::test_vm_config(),
            near_sdk::RuntimeFeesConfig::test(),
            HashMap::default(),
            vec![PromiseResult::Failed]
        );
        assert!(!contract.after_token_burn(
            account_id.clone(),
            token_id.to_string(),
            U128(100),
            unlock_time_sec,
        ));
    }

    #[test]
    fn failed_full_burn_keeps_original_expiry_when_new_lock_is_shorter() {
        let old_expiry = NOW_SEC + 365 * DAY_SEC;
        let (mut contract, account_id, token_id, callback_expiry) =
            begin_full_burn("token.near", old_expiry);
        let mut account = contract.internal_unwrap_account(&account_id);
        account.add_lock(&token_id, U128(1), NOW_SEC + DAY_SEC);
        contract.internal_set_account(&account_id, account);

        finish_failed_burn(&mut contract, &account_id, &token_id, callback_expiry);
        let account = contract.internal_unwrap_account(&account_id);
        let lock = &account.locked_tokens[&token_id];
        assert_eq!(lock.locked_balance, U128(101));
        assert_eq!(lock.unlock_time_sec, old_expiry);
    }

    #[test]
    fn failed_full_burn_restores_expiry_without_replacement_lock() {
        let old_expiry = NOW_SEC + 365 * DAY_SEC;
        let (mut contract, account_id, token_id, callback_expiry) =
            begin_full_burn("token.near@:0", old_expiry);

        finish_failed_burn(&mut contract, &account_id, &token_id, callback_expiry);
        let account = contract.internal_unwrap_account(&account_id);
        let lock = &account.locked_tokens[&token_id];
        assert_eq!(lock.locked_balance, U128(100));
        assert_eq!(lock.unlock_time_sec, old_expiry);
    }

    #[test]
    fn failed_full_burn_keeps_later_replacement_expiry() {
        let old_expiry = NOW_SEC + 365 * DAY_SEC;
        let later_expiry = old_expiry + DAY_SEC;
        let (mut contract, account_id, token_id, callback_expiry) =
            begin_full_burn("token.near", old_expiry);
        let mut account = contract.internal_unwrap_account(&account_id);
        account.add_lock(&token_id, U128(1), later_expiry);
        contract.internal_set_account(&account_id, account);

        finish_failed_burn(&mut contract, &account_id, &token_id, callback_expiry);
        let account = contract.internal_unwrap_account(&account_id);
        let lock = &account.locked_tokens[&token_id];
        assert_eq!(lock.locked_balance, U128(101));
        assert_eq!(lock.unlock_time_sec, later_expiry);
    }
}
