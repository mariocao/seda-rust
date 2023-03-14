use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{
    json_types::{U128, U64},
    test_utils::get_logs,
    testing_env,
};

use super::test_utils::{
    alice,
    bn254_sign,
    bob,
    carol,
    dao,
    get_context,
    get_context_for_ft_transfer,
    get_context_view,
    get_context_with_deposit,
    get_context_with_deposit_at_block,
    new_contract,
};
use crate::{
    consts::INIT_MINIMUM_STAKE,
    node_registry::{HumanReadableNode, UpdateNode},
};

#[test]
fn register_and_get_node() {
    let mut contract = new_contract();
    let bob_signature = bn254_sign(bob(), &bob().account_id.as_bytes());

    // register node
    testing_env!(get_context_with_deposit(bob()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node"]);
    // check owner and multi_addr
    testing_env!(get_context_view());
    assert_eq!(
        "0.0.0.0:8080".to_string(),
        contract.get_node(bob().account_id).unwrap().multi_addr
    );
}

#[test]
#[should_panic(expected = "Insufficient storage, need 5240000000000000000000")]
fn register_not_enough_storage() {
    let mut contract = new_contract();
    let bob_signature = bn254_sign(bob(), &bob().account_id.as_bytes());

    // register node
    testing_env!(get_context(bob()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
}

#[test]
fn set_node_multi_addr() {
    let mut contract = new_contract();
    let bob_signature = bn254_sign(bob(), &bob().account_id.as_bytes());

    // register node
    testing_env!(get_context_with_deposit(bob()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node"]);

    // update the multi_addr
    contract.update_node(UpdateNode::SetSocketAddress("1.1.1.1:8081".to_string()));

    // check the multi_addr after updating
    testing_env!(get_context_view());
    assert_eq!(
        "1.1.1.1:8081".to_string(),
        contract.get_node(bob().account_id).unwrap().multi_addr
    );
}

#[test]
fn get_nodes() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);
    let bob_signature = bn254_sign(bob(), "bob_near".to_string().as_bytes());
    let alice_signature = bn254_sign(alice(), "alice_near".to_string().as_bytes());
    let carol_signature = bn254_sign(carol(), "carol_near".to_string().as_bytes());

    // DAO transfers tokens to bob, alice, and carol
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    contract.storage_deposit(Some("bob_near".to_string().try_into().unwrap()), None);
    contract.storage_deposit(Some("carol_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);
    contract.ft_transfer("bob_near".to_string().try_into().unwrap(), deposit_amount, None);
    contract.ft_transfer("carol_near".to_string().try_into().unwrap(), deposit_amount, None);

    // register three nodes
    testing_env!(get_context_with_deposit(bob()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node",]);
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "1.1.1.1:8080".to_string(),
        alice().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["alice_near registered node",]);
    testing_env!(get_context_with_deposit(carol()));
    contract.register_node(
        "2.2.2.2:8080".to_string(),
        carol().bn254_public_key.to_compressed().unwrap(),
        carol_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["carol_near registered node",]);

    // all nodes deposit the minimum stake
    testing_env!(get_context_with_deposit(bob()));
    contract.deposit(deposit_amount, bob().ed25519_public_key.into_bytes());
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.into_bytes());
    testing_env!(get_context_with_deposit(carol()));
    contract.deposit(deposit_amount, carol().ed25519_public_key.into_bytes());

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(bob(), 1000000));
    contract.process_epoch();

    // define expected nodes
    let node1 = HumanReadableNode {
        account_id:         "bob_near".to_string().try_into().unwrap(),
        balance:            deposit_amount.0,
        multi_addr:         "0.0.0.0:8080".to_string(),
        bn254_public_key:   bob().bn254_public_key.to_compressed().unwrap(),
        ed25519_public_key: bob().ed25519_public_key.into_bytes(),
    };
    let node2 = HumanReadableNode {
        account_id:         "alice_near".to_string().try_into().unwrap(),
        balance:            deposit_amount.0,
        multi_addr:         "1.1.1.1:8080".to_string(),
        bn254_public_key:   alice().bn254_public_key.to_compressed().unwrap(),
        ed25519_public_key: alice().ed25519_public_key.into_bytes(),
    };
    let node3 = HumanReadableNode {
        account_id:         "carol_near".to_string().try_into().unwrap(),
        balance:            deposit_amount.0,
        multi_addr:         "2.2.2.2:8080".to_string(),
        bn254_public_key:   carol().bn254_public_key.to_compressed().unwrap(),
        ed25519_public_key: carol().ed25519_public_key.into_bytes(),
    };

    // get the first node
    testing_env!(get_context_view());
    let get_node = contract.get_node("bob_near".to_string().try_into().unwrap());
    assert_eq!(get_node.unwrap(), node1);

    // check the latest 2 nodes
    let latest_2_nodes = contract.get_nodes(U64(2), U64(0));
    assert_eq!(latest_2_nodes, vec![node3.clone(), node2.clone()]);

    // check the latest 3 nodes
    let latest_3_nodes = contract.get_nodes(U64(100), U64(0));
    assert_eq!(latest_3_nodes, vec![node3, node2.clone(), node1.clone()]);

    // check offset of 1
    let latest_nodes_offset = contract.get_nodes(U64(100), U64(1));
    assert_eq!(latest_nodes_offset, vec![node2, node1]);
}

#[test]
#[should_panic(expected = "bn254_public_key already exists")]
fn duplicated_key() {
    let mut contract = new_contract();
    let bob_signature = bn254_sign(bob(), "bob_near".to_string().as_bytes());
    let alice_signature = bn254_sign(bob(), "alice_near".to_string().as_bytes());

    // bob registers node
    testing_env!(get_context_with_deposit(bob()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );

    // alice registers node with duplicated key
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "1.1.1.1:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );
}

#[test]
fn deposit_withdraw() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice registers node
    let alice_signature = bn254_sign(alice(), &alice().account_id.as_bytes());
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        alice().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );

    // alice deposits into pool
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());

    // check alice's balance is now zero
    assert_eq!(
        contract.ft_balance_of("alice_near".to_string().try_into().unwrap()),
        U128(0)
    );

    // check alice is not active
    assert_eq!(
        contract.is_node_active("alice_near".to_string().try_into().unwrap()),
        false
    );

    // check alice's deposited amount
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, deposit_amount);

    // alice withdraws
    testing_env!(get_context(alice()));
    contract.withdraw(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());

    // check alice's balance has increased again and the node balance has decreased
    assert_eq!(
        contract.ft_balance_of("alice_near".to_string().try_into().unwrap()),
        deposit_amount
    );
    assert_eq!(
        contract.get_node_balance("alice_near".to_string().try_into().unwrap()),
        U128(0)
    );
}

#[test]
#[should_panic(expected = "No deposit info found for this account")]
fn withdraw_wrong_account() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice registers node
    let alice_signature = bn254_sign(alice(), &alice().account_id.as_bytes());
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        alice().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );

    // alice deposits into pool
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());

    // check alice's balance is now zero
    assert_eq!(
        contract.ft_balance_of("alice_near".to_string().try_into().unwrap()),
        U128(0)
    );

    // check alice is not active
    assert_eq!(
        contract.is_node_active("alice_near".to_string().try_into().unwrap()),
        false
    );

    // check alice's deposited amount
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, deposit_amount);

    // bob tries withdrawing from alice's account
    testing_env!(get_context(bob()));
    contract.withdraw(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());
}

#[test]
fn deposit_withdraw_one_node_two_depositors() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);

    // DAO transfers tokens to alice and bob
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    contract.storage_deposit(Some("bob_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);
    contract.ft_transfer("bob_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice registers node
    let alice_signature = bn254_sign(alice(), &alice().account_id.as_bytes());
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        alice().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );

    // alice and bob deposit into alice's pool
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());
    testing_env!(get_context_with_deposit(bob()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());

    // check total deposited amount is now 2x deposit amount
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(deposit_amount.0 * 2));

    // alice and bob withdraw
    testing_env!(get_context(alice()));
    contract.withdraw(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());
    testing_env!(get_context(bob()));
    contract.withdraw(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());

    // check total deposited amount is now 0
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(0));
}

#[test]
fn deposit_withdraw_two_nodes_one_depositor() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice and bob register nodes
    let alice_signature = bn254_sign(alice(), &alice().account_id.as_bytes());
    let bob_signature = bn254_sign(bob(), &bob().account_id.as_bytes());
    testing_env!(get_context_with_deposit(alice()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        alice().bn254_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );
    testing_env!(get_context_with_deposit(bob()));
    contract.register_node(
        "1.1.1.1:8080".to_string(),
        bob().bn254_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );

    // alice deposits into alice and bob's pool
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(
        U128(deposit_amount.0 / 2),
        alice().ed25519_public_key.as_bytes().to_vec(),
    );
    contract.deposit(U128(deposit_amount.0 / 2), bob().ed25519_public_key.as_bytes().to_vec());

    // assert alice has deposits in 2 pools
    let alice_deposits = contract.get_deposits(alice().account_id);
    assert_eq!(alice_deposits.len(), 2);

    // check alice's balance is now zero
    assert_eq!(
        contract.ft_balance_of("alice_near".to_string().try_into().unwrap()),
        U128(0)
    );

    // check deposited amounts for both pools is deposit amount / 2
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(deposit_amount.0 / 2));
    let node_balance = contract.get_node_balance("bob_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(deposit_amount.0 / 2));

    // alice withdraws from both pools
    testing_env!(get_context(alice()));
    contract.withdraw(
        U128(deposit_amount.0 / 2),
        alice().ed25519_public_key.as_bytes().to_vec(),
    );
    contract.withdraw(U128(deposit_amount.0 / 2), bob().ed25519_public_key.as_bytes().to_vec());

    // check alice's balance is now original deposit amount
    assert_eq!(
        contract.ft_balance_of("alice_near".to_string().try_into().unwrap()),
        deposit_amount
    );

    // check deposited amounts for both pools is 0
    let node_balance = contract.get_node_balance("alice_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(0));
    let node_balance = contract.get_node_balance("bob_near".to_string().try_into().unwrap());
    assert_eq!(node_balance, U128(0));
}

#[test]
#[should_panic(
    expected = "Node [0, 16, 116, 94, 20, 231, 206, 129, 110, 161, 68, 52, 61, 13, 115, 32, 105, 1, 254, 31, 107, 86, 214, 80, 168, 253, 13, 234, 69, 203, 224, 201, 242] does not exist"
)]
fn deposit_nonexistent_node() {
    let mut contract = new_contract();
    let deposit_amount = U128(100_000_000_000_000_000_000_000);

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit(dao()));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer(dao()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice deposits into pool
    testing_env!(get_context_with_deposit(alice()));
    contract.deposit(deposit_amount, alice().ed25519_public_key.as_bytes().to_vec());
}
