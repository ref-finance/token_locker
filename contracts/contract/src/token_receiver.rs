use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[near(serializers = [json])]
enum TokenReceiverMessage {
    Lock { unlock_time_sec: u32 },
}

#[near]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&sender_id);

        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("INVALID MSG");
        match message {
            TokenReceiverMessage::Lock { unlock_time_sec } => {
                if let Some(lock_info) = account.locked_tokens.get_mut(&token_id.to_string()) {
                    lock_info.append_lock(amount, unlock_time_sec);
                    Event::AppendToken {
                        account_id: &sender_id,
                        token_id: &token_id.to_string(),
                        amount: &amount,
                        unlock_time_sec,
                    }
                    .emit();
                } else {
                    account.add_lock(&token_id.to_string(), amount, unlock_time_sec);
                    Event::LockedToken {
                        account_id: &sender_id,
                        token_id: &token_id.to_string(),
                        amount: &amount,
                        unlock_time_sec,
                    }
                    .emit();
                }
            }
        }
        self.internal_set_account(&sender_id, account);
        PromiseOrValue::Value(U128(0))
    }
}

pub trait MFTTokenReceiver {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[near]
impl MFTTokenReceiver for Contract {
    fn mft_on_transfer(
        &mut self,
        token_id: String,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = generate_mft_token_id(token_id);
        let mut account = self.internal_unwrap_account(&sender_id);

        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("INVALID MSG");
        match message {
            TokenReceiverMessage::Lock { unlock_time_sec } => {
                if let Some(lock_info) = account.locked_tokens.get_mut(&token_id) {
                    lock_info.append_lock(amount, unlock_time_sec);
                    Event::AppendToken {
                        account_id: &sender_id,
                        token_id: &token_id.to_string(),
                        amount: &amount,
                        unlock_time_sec,
                    }
                    .emit();
                } else {
                    account.add_lock(&token_id, amount, unlock_time_sec);
                    Event::LockedToken {
                        account_id: &sender_id,
                        token_id: &token_id.to_string(),
                        amount: &amount,
                        unlock_time_sec,
                    }
                    .emit();
                }
            }
        }
        self.internal_set_account(&sender_id, account);
        PromiseOrValue::Value(U128(0))
    }
}
