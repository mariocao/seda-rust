use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{
    bn254_sign,
    get_context_for_ft_transfer,
    get_context_with_deposit,
    make_test_account,
    new_contract,
};
use crate::consts::{INITIAL_SUPPLY, INIT_MINIMUM_STAKE};

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

#[test]
fn total_supply_includes_staked() {
    let mut contract = new_contract();
    let dao = make_test_account("dao_near".to_string());
    let bob = make_test_account("bob_near".to_string());
    let bob_signature = bn254_sign(&bob.bn254_private_key.clone(), "bob_near".to_string().as_bytes());
    let deposit_amount = U128(INIT_MINIMUM_STAKE);

    // DAO transfers tokens to bob
    testing_env!(get_context_with_deposit(dao.clone()));
    contract.storage_deposit(Some("bob_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao));
    contract.ft_transfer("bob_near".to_string().try_into().unwrap(), deposit_amount, None);

    // bob registers node
    testing_env!(get_context_with_deposit(bob.clone()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob.clone().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );

    // bob deposits
    testing_env!(get_context_with_deposit(bob.clone()));
    contract.deposit(deposit_amount, bob.clone().ed25519_public_key.into_bytes());

    assert_eq!(contract.ft_total_supply(), U128(INITIAL_SUPPLY));
}
