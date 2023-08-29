use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, AccountId, Balance, PanicOnDefault};

pub mod investment {
    pub mod investment;
}

use crate::investment::investment::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,

    pub token_contract_address: AccountId,

    pub total_supply: Balance,

    pub investors: UnorderedMap<AccountId, Investment>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        token_contract_address: AccountId
    ) -> Self {
        let this = Self {
            owner_id: owner_id,
            token_contract_address: token_contract_address,
            total_supply: total_supply.into(),
            investors: UnorderedMap::new("Investors".try_to_vec().unwrap()),
        };
        this
    }
}