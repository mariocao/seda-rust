use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{get_context_for_ft_transfer, get_context_with_deposit, make_test_account, new_contract};
use crate::consts::INITIAL_SUPPLY;

#[test]
fn total_supply() {
    let contract = new_contract();
    assert_eq!(contract.ft_total_supply(), U128(INITIAL_SUPPLY));
}

#[test]
fn simple_transfer() {
    let mut contract = new_contract();
    let dao = make_test_account("dao_near".to_string());
    let transfer_amount = U128(100);

    let initial_dao_balance = contract.ft_balance_of("dao_near".to_string().try_into().unwrap());

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit(dao.clone()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), transfer_amount, None);

    let dao_balance = contract.ft_balance_of("dao_near".to_string().try_into().unwrap());
    let alice_balance = contract.ft_balance_of("alice_near".to_string().try_into().unwrap());

    assert_eq!(dao_balance, U128(initial_dao_balance.0 - transfer_amount.0));
    assert_eq!(alice_balance, transfer_amount);
}
