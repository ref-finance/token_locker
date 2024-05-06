use contract::{nano_to_sec, Metadata};
use near_sdk::{json_types::U128, serde_json::json, AccountId, NearToken};
use near_workspaces::{result::{ExecutionFinalResult, Result}, Account, Contract};
use contract::Account as ContractAccount;

const FT_WASM: &str = "../../res/mock_ft.wasm";
const MFT_WASM: &str = "../../res/mock_mft.wasm";
const TOKEN_LOCKER_WASM: &str = "../../res/token_locker.wasm";

#[tokio::test]
async fn test_base() -> Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let alice = root.create_subaccount("alice").initial_balance(NearToken::from_near(50)).transact().await?.unwrap();

    let token_locker_contract = deploy_token_locker(&root).await?;
    let ft_token_contract = deploy_mock_ft(&root).await?;
    let mft_token_contract = deploy_mock_mft(&root).await?;

    check!(extend_token_white_list(&token_locker_contract, &root, vec![ft_token_contract.id(), mft_token_contract.id()]));

    check!(logs storage_deposit(&token_locker_contract, &alice.id()));
    check!(storage_deposit(&ft_token_contract, &alice.id()));
    check!(storage_deposit(&ft_token_contract, &token_locker_contract.id()));

    check!(view get_metadata(&token_locker_contract));

    check!(mint_ft(&ft_token_contract, alice.id(), NearToken::from_near(200).as_yoctonear()));
    check!(mint_mft(&mft_token_contract, alice.id(), "0".to_string(), NearToken::from_near(100).as_yoctonear()));

    check!(view "alice ft" ft_balance_of(&ft_token_contract, alice.id()));
    check!(view "alice mft" mft_balance_of(&mft_token_contract, ":0".to_string(), alice.id()));

    let current_timestamp = worker.view_block().await?.timestamp();
    let unlock_time_sec = nano_to_sec(current_timestamp) + 120;
    let msg = json!({
        "Lock": {
            "unlock_time_sec": unlock_time_sec
        }
    }).to_string();
    check!(logs ft_transfer_call(&ft_token_contract, &alice, token_locker_contract.id(), NearToken::from_near(1).as_yoctonear(), msg.clone()));
    check!(view "token_locker_contract ft" ft_balance_of(&ft_token_contract, token_locker_contract.id()));

    check!(mft_register(&mft_token_contract, token_locker_contract.id(), ":0".to_string()));
    check!(logs mft_transfer_call(&mft_token_contract, &alice, ":0".to_string(), token_locker_contract.id(), NearToken::from_near(1).as_yoctonear(), msg.clone()));
    check!(view "token_locker_contract mft" mft_balance_of(&mft_token_contract, ":0".to_string(), token_locker_contract.id()));

    check!(logs ft_transfer_call(&ft_token_contract, &alice, token_locker_contract.id(), NearToken::from_near(1).as_yoctonear(), msg.clone()));
    check!(view "token_locker_contract ft" ft_balance_of(&ft_token_contract, token_locker_contract.id()));

    check!(logs mft_transfer_call(&mft_token_contract, &alice, ":0".to_string(), token_locker_contract.id(), NearToken::from_near(1).as_yoctonear(), msg.clone()));
    check!(view "token_locker_contract mft" mft_balance_of(&mft_token_contract, ":0".to_string(), token_locker_contract.id()));

    check!(view get_account(&token_locker_contract,alice.id()));

    check!(withdraw(&token_locker_contract, &alice, ft_token_contract.id().to_string(), Some(U128(NearToken::from_near(1).as_yoctonear()))), "Token still locked");
    check!(withdraw(&token_locker_contract, &alice, "mock_mft.test.near@:0".to_string(), Some(U128(NearToken::from_near(1).as_yoctonear()))), "Token still locked");

    while nano_to_sec(worker.view_block().await?.timestamp()) < unlock_time_sec {
        worker.fast_forward(20).await?;
    }

    check!(view "alice ft" ft_balance_of(&ft_token_contract, alice.id()));
    check!(withdraw(&token_locker_contract, &alice, ft_token_contract.id().to_string(), Some(U128(NearToken::from_near(1).as_yoctonear()))));
    check!(view "alice ft" ft_balance_of(&ft_token_contract, alice.id()));

    check!(storage_unregister(&token_locker_contract, &alice, None), "STILL HAS TOKENS");

    check!(logs withdraw(&token_locker_contract, &alice, ft_token_contract.id().to_string(), None));
    check!(logs withdraw(&token_locker_contract, &alice, "mock_mft.test.near@:0".to_string(), None));
    check!(view "alice ft" ft_balance_of(&ft_token_contract, alice.id()));
    check!(view "alice mft" mft_balance_of(&mft_token_contract, ":0".to_string(), alice.id()));

    check!(view "token_locker_contract ft" ft_balance_of(&ft_token_contract, token_locker_contract.id()));
    check!(view "token_locker_contract mft" mft_balance_of(&mft_token_contract, ":0".to_string(), token_locker_contract.id()));
    check!(view get_account(&token_locker_contract,alice.id()));
    check!(view get_metadata(&token_locker_contract));

    check!(logs storage_unregister(&token_locker_contract, &alice, None));
    check!(view get_metadata(&token_locker_contract));
    
    check!(storage_deposit(&token_locker_contract, &alice.id()));
    let current_timestamp = worker.view_block().await?.timestamp();
    let unlock_time_sec = nano_to_sec(current_timestamp) + 120;
    let msg = json!({
        "Lock": {
            "unlock_time_sec": unlock_time_sec
        }
    }).to_string();
    check!(ft_transfer_call(&ft_token_contract, &alice, token_locker_contract.id(), NearToken::from_near(1).as_yoctonear(), msg.clone()));
    check!(storage_unregister(&ft_token_contract, &alice, Some(true)));
    check!(mft_transfer_call(&mft_token_contract, &alice, ":0".to_string(), token_locker_contract.id(), NearToken::from_near(5).as_yoctonear(), msg.clone()));
    check!(mft_unregister(&mft_token_contract, &alice, ":0".to_string(), Some(true)));
    
    while nano_to_sec(worker.view_block().await?.timestamp()) < unlock_time_sec {
        worker.fast_forward(20).await?;
    }
    check!(view get_account(&token_locker_contract,alice.id()));
    check!(withdraw(&token_locker_contract, &alice, ft_token_contract.id().to_string(), None), "The account alice.test.near is not registered");
    check!(withdraw(&token_locker_contract, &alice, "mock_mft.test.near@:0".to_string(), Some(U128(NearToken::from_near(1).as_yoctonear()))), "ERR_RECEIVER_NOT_REGISTERED");
    check!(view get_account(&token_locker_contract,alice.id()));

    Ok(())
}

pub async fn extend_token_white_list(
    contract: &Contract,
    sender: &Account,
    token_ids: Vec<&AccountId>,
) -> Result<ExecutionFinalResult> {
    sender
        .call(contract.id(), "extend_token_white_list")
        .args_json(json!({
            "token_ids": token_ids,
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn withdraw(
    contract: &Contract,
    sender: &Account,
    token_id: String, 
    amount: Option<U128>
) -> Result<ExecutionFinalResult> {
    sender
        .call(contract.id(), "withdraw")
        .args_json(json!({
            "token_id": token_id,
            "amount": amount
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn get_account(
    contract: &Contract,
    account_id: &AccountId,
) -> Result<ContractAccount> {
    contract
        .call("get_account")
        .args_json(json!({
            "account_id": account_id
        }))
        .view()
        .await?
        .json::<ContractAccount>()
}

pub async fn get_metadata(
    contract: &Contract,
) -> Result<Metadata> {
    contract
        .call("get_metadata")
        .args_json(json!({
        }))
        .view()
        .await?
        .json::<Metadata>()
}


pub async fn mft_transfer_call(
    contract: &Contract,
    sender: &Account,
    token_id: String,
    receiver_id: &AccountId,
    amount: u128,
    msg: String,
) -> Result<ExecutionFinalResult> {
    sender
        .call(contract.id(), "mft_transfer_call")
        .args_json(json!({
            "token_id": token_id,
            "receiver_id": receiver_id,
            "amount": U128(amount),
            "memo": Option::<String>::None,
            "msg": msg.clone(),
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn ft_transfer_call(
    contract: &Contract,
    sender: &Account,
    receiver_id: &AccountId,
    amount: u128,
    msg: String,
) -> Result<ExecutionFinalResult> {
    sender
        .call(contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": receiver_id,
            "amount": U128::from(amount),
            "memo": Option::<String>::None,
            "msg": msg.clone(),
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn deploy_token_locker(
    root: &Account,
) -> Result<Contract> {
    let token_locker = root
        .create_subaccount("token_locker")
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?
        .unwrap();
    let token_locker = token_locker
        .deploy(&std::fs::read(TOKEN_LOCKER_WASM).unwrap())
        .await?
        .unwrap();
    assert!(token_locker
        .call("new")
        .args_json(json!({
            "owner_id": root.id(),
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(token_locker)
}

pub async fn deploy_mock_ft(
    root: &Account,
) -> Result<Contract> {
    let mock_ft = root
        .create_subaccount("ft")
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?
        .unwrap();
    let mock_ft = mock_ft
        .deploy(&std::fs::read(FT_WASM).unwrap())
        .await?
        .unwrap();
    assert!(mock_ft
        .call("new_default_meta")
        .args_json(json!({
            "owner_id": root.id(),
            "total_supply": U128(u128::MAX / 2),
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(mock_ft)
}

pub async fn deploy_mock_mft(
    root: &Account,
) -> Result<Contract> {
    let mock_mft = root
        .create_subaccount("mock_mft")
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?
        .unwrap();
    let mock_mft = mock_mft
        .deploy(&std::fs::read(MFT_WASM).unwrap())
        .await?
        .unwrap();
    assert!(mock_mft
        .call("new")
        .args_json(json!({
            "name": "Multi Fungible Token".to_string(),
            "symbol": "MFT".to_string(),
            "decimals": 24
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());
    Ok(mock_mft)
}

pub async fn storage_deposit(
    contract: &Contract,
    account_id: &AccountId,
) -> Result<ExecutionFinalResult> {
    contract
        .call("storage_deposit")
        .args_json(json!({
            "account_id": Some(account_id),
            "registration_only": Option::<bool>::None,
        }))
        .max_gas()
        .deposit(NearToken::from_millinear(100))
        .transact()
        .await
}

pub async fn storage_unregister(
    contract: &Contract,
    account: &Account,
    force: Option<bool>
) -> Result<ExecutionFinalResult> {
    account
        .call(contract.id(), "storage_unregister")
        .args_json(json!({
            "force": force
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn mft_register(
    contract: &Contract,
    account_id: &AccountId,
    mft_token_id: String,
) -> Result<ExecutionFinalResult> {
    contract
        .call("mft_register")
        .args_json(json!({
            "token_id": mft_token_id,
            "account_id": account_id,
        }))
        .max_gas()
        .deposit(NearToken::from_millinear(100))
        .transact()
        .await
}

pub async fn mft_unregister(
    contract: &Contract,
    account: &Account,
    token_id: String, 
    force: Option<bool>
) -> Result<ExecutionFinalResult> {
    account
        .call(contract.id(), "mft_unregister")
        .args_json(json!({
            "token_id": token_id,
            "force": force,
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await
}

pub async fn mint_ft(
    contract: &Contract,
    account_id: &AccountId,
    amount: u128
) -> Result<ExecutionFinalResult> {
    contract
        .call("mint")
        .args_json(json!({
            "account_id": account_id,
            "amount": U128(amount),
        }))
        .max_gas()
        .transact()
        .await
}

pub async fn mint_mft(
    contract: &Contract,
    account_id: &AccountId,
    inner_id: String, 
    amount: u128
) -> Result<ExecutionFinalResult> {
    contract
        .call("mint")
        .args_json(json!({
            "account_id": account_id,
            "inner_id": inner_id,
            "amount": U128(amount),
        }))
        .max_gas()
        .transact()
        .await
}

pub async fn ft_balance_of(
    contract: &Contract,
    account_id: &AccountId,
) -> Result<U128> {
    contract
        .call("ft_balance_of")
        .args_json(json!({
            "account_id": account_id
        }))
        .view()
        .await?
        .json::<U128>()
}

pub async fn mft_balance_of(
    contract: &Contract,
    mft_token_id: String,
    account_id: &AccountId,
) -> Result<U128> {
    contract
        .call("mft_balance_of")
        .args_json(json!({
            "token_id": mft_token_id,
            "account_id": account_id
        }))
        .view()
        .await?
        .json::<U128>()
}

pub fn tool_err_msg(outcome: Result<ExecutionFinalResult>) -> String {
    match outcome {
        Ok(res) => {
            let mut msg = "".to_string();
            for r in res.receipt_failures(){
                match r.clone().into_result() {
                    Ok(_) => {},
                    Err(err) => {
                        msg += &format!("{:?}", err);
                        msg += "\n";
                    }
                }
            }
            msg
        },
        Err(err) => err.to_string()
    }
}

#[macro_export]
macro_rules! check{
    ($exec_func: expr)=>{
        let outcome = $exec_func.await?;
        assert!(outcome.is_success() && outcome.receipt_failures().is_empty());
    };
    (print $exec_func: expr)=>{
        let outcome = $exec_func.await;
        let err_msg = tool_err_msg(outcome);
        if err_msg.is_empty() {
            println!("success");
        } else {
            println!("{}", err_msg);
        }
    };
    (print $prefix: literal $exec_func: expr)=>{
        let outcome = $exec_func.await;
        let err_msg = tool_err_msg(outcome);
        if err_msg.is_empty() {
            println!("{} success", $prefix);
        } else {
            println!("{} {}", $prefix, err_msg);
        }
    };
    (view $exec_func: expr)=>{
        let query_result = $exec_func.await?;
        println!("{:?}", query_result);
    };
    (view $prefix: literal $exec_func: expr)=>{
        let query_result = $exec_func.await?;
        println!("{} {:?}", $prefix, query_result);
    };
    (logs $exec_func: expr)=>{
        let outcome = $exec_func.await?;
        assert!(outcome.is_success() && outcome.receipt_failures().is_empty());
        println!("{:#?}", outcome.logs());
    };
    ($exec_func: expr, $err_info: expr)=>{
        assert!(tool_err_msg($exec_func.await).contains($err_info));
    };
}