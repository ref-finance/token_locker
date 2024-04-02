use crate::*;
use near_sdk::serde::Serialize;

const EVENT_STANDARD: &str = "token-locker";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    LockedToken {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
        unlock_time_sec: u32,
    },
    AppendToken {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
        unlock_time_sec: u32,
    },
    WithdrawStarted {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
    },
    WithdrawSucceeded {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
    },
    WithdrawFailed {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
    },
    WithdrawLostfound {
        account_id: &'a AccountId,
        token_id: &'a String,
        amount: &'a U128,
    },
    AccountRegister {
        account_id: &'a AccountId,
    },
    AccountUnregister {
        account_id: &'a AccountId,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!(format!("EVENT_JSON:{}", event_json));
}
