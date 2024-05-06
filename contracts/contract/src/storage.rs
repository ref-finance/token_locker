use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

pub const STORAGE_BALANCE_MIN_BOUND: NearToken = NearToken::from_millinear(100);

#[near]
impl StorageManagement for Contract {
    #[allow(unused_variables)]
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let already_registered = self.internal_get_account(&account_id).is_some();
        if amount < STORAGE_BALANCE_MIN_BOUND && !already_registered {
            env::panic_str("Insufficient deposit");
        }

        if already_registered {
            if !amount.is_zero() {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            self.internal_set_account(&account_id, Account::new(&account_id).into());
            let refund = amount.checked_sub(STORAGE_BALANCE_MIN_BOUND).unwrap();
            if !refund.is_zero() {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
            Event::AccountRegister { account_id: &account_id }.emit();
        }
        self.storage_balance_of(account_id).unwrap()
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        unimplemented!()
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        if let Some(account) = self.internal_get_account(&account_id) {
            if !force.unwrap_or(false) {
                require!(account.locked_tokens.is_empty(), "STILL HAS TOKENS");
            }
            self.data_mut().accounts.remove(&account_id);
            Promise::new(account_id.clone()).transfer(STORAGE_BALANCE_MIN_BOUND);
            Event::AccountUnregister { account_id: &account_id }.emit();
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: STORAGE_BALANCE_MIN_BOUND,
            max: Some(STORAGE_BALANCE_MIN_BOUND),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        if self.internal_get_account(&account_id).is_some() {
            Some(StorageBalance {
                total: STORAGE_BALANCE_MIN_BOUND,
                available: NearToken::from_near(0),
            })
        } else {
            None
        }
    }
}
