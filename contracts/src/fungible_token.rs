use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use near_sdk::{json_types::U128, log, near_bindgen, AccountId, Balance, PromiseOrValue};

use crate::{MainchainContract, MainchainContractExt};

/// Private contract
impl MainchainContract {
    pub fn mint(&mut self, account_id: &AccountId, amount: Balance) {
        let new_user_balance = self.token.accounts.get(account_id).unwrap() + amount;
        self.token.accounts.insert(account_id, &new_user_balance);
    }

    pub fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        let new_user_balance = self.token.accounts.get(account_id).unwrap() - amount;
        self.token.accounts.insert(account_id, &new_user_balance);
    }
}

/// Public contract methods
#[near_bindgen]
impl MainchainContract {
    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(MainchainContract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(MainchainContract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for MainchainContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
