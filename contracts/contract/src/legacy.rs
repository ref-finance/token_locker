use crate::*;

#[near(serializers = [borsh])]
pub struct ContractDataV1000 {
    owner_id: AccountId,
    accounts: UnorderedMap<AccountId, VAccount>,
    token_white_list: UnorderedSet<AccountId>,
}

impl From<ContractDataV1000> for ContractData {
    fn from(a: ContractDataV1000) -> Self {
        let ContractDataV1000 {
            owner_id,
            accounts,
            token_white_list,
        } = a;
        Self {
            owner_id,
            accounts,
            token_white_list,
            burn_account_id: None,
        }
    }
}
