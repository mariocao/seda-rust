use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{
    json_types::{U128, U64},
    test_utils::get_logs,
    testing_env,
};

use super::test_utils::{
    bn254_sign,
    generate_bn254_key,
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
    let (bob_public_key, bob_private_key) = generate_bn254_key();
    let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());

    // register node
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node"]);
    // check owner and multi_addr
    testing_env!(get_context_view());
    assert_eq!(
        "0.0.0.0:8080".to_string(),
        contract
            .get_node("bob_near".to_string().try_into().unwrap())
            .unwrap()
            .multi_addr
    );
}

#[test]
#[should_panic(expected = "Insufficient storage, need 3970000000000000000000")]
fn register_not_enough_storage() {
    let mut contract = new_contract();
    let (bob_public_key, bob_private_key) = generate_bn254_key();
    let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());

    // register node
    testing_env!(get_context("bob_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
}

#[test]
fn set_node_multi_addr() {
    let mut contract = new_contract();
    let (bob_public_key, bob_private_key) = generate_bn254_key();
    let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());

    // register node
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node"]);

    // update the multi_addr
    contract.update_node(UpdateNode::SetSocketAddress("1.1.1.1:8081".to_string()));

    // check the multi_addr after updating
    testing_env!(get_context_view());
    assert_eq!(
        "1.1.1.1:8081".to_string(),
        contract
            .get_node("bob_near".to_string().try_into().unwrap())
            .unwrap()
            .multi_addr
    );
}

#[test]
fn get_nodes() {
    let mut contract = new_contract();
    let deposit_amount = U128(INIT_MINIMUM_STAKE);
    let (bob_public_key, bob_private_key) = generate_bn254_key();
    let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());
    let (alice_public_key, alice_private_key) = generate_bn254_key();
    let alice_signature = bn254_sign(&alice_private_key, "alice_near".to_string().as_bytes());
    let (carol_public_key, carol_private_key) = generate_bn254_key();
    let carol_signature = bn254_sign(&carol_private_key, "carol_near".to_string().as_bytes());

    // DAO transfers tokens to bob, alice, and carol
    testing_env!(get_context_with_deposit("dao_near".to_string(),));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    contract.storage_deposit(Some("bob_near".to_string().try_into().unwrap()), None);
    contract.storage_deposit(Some("carol_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer("dao_near".to_string()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);
    contract.ft_transfer("bob_near".to_string().try_into().unwrap(), deposit_amount, None);
    contract.ft_transfer("carol_near".to_string().try_into().unwrap(), deposit_amount, None);

    // register three nodes
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["bob_near registered node",]);
    testing_env!(get_context_with_deposit("alice_near".to_string()));
    contract.register_node(
        "1.1.1.1:8080".to_string(),
        alice_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["alice_near registered node",]);
    testing_env!(get_context_with_deposit("carol_near".to_string()));
    contract.register_node(
        "2.2.2.2:8080".to_string(),
        carol_public_key.to_compressed().unwrap(),
        carol_signature.to_compressed().unwrap(),
    );
    assert_eq!(get_logs(), vec!["carol_near registered node",]);

    // all nodes deposit the minimum stake
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.deposit(deposit_amount);
    testing_env!(get_context_with_deposit("alice_near".to_string()));
    contract.deposit(deposit_amount);
    testing_env!(get_context_with_deposit("carol_near".to_string()));
    contract.deposit(deposit_amount);

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block("bob_near".to_string(), 1000000));
    contract.process_epoch();

    // define expected nodes
    let node1 = HumanReadableNode {
        account_id:       "bob_near".to_string().try_into().unwrap(),
        balance:          deposit_amount.0,
        multi_addr:       "0.0.0.0:8080".to_string(),
        bn254_public_key: bob_public_key.to_compressed().unwrap(),
    };
    let node2 = HumanReadableNode {
        account_id:       "alice_near".to_string().try_into().unwrap(),
        balance:          deposit_amount.0,
        multi_addr:       "1.1.1.1:8080".to_string(),
        bn254_public_key: alice_public_key.to_compressed().unwrap(),
    };
    let node3 = HumanReadableNode {
        account_id:       "carol_near".to_string().try_into().unwrap(),
        balance:          deposit_amount.0,
        multi_addr:       "2.2.2.2:8080".to_string(),
        bn254_public_key: carol_public_key.to_compressed().unwrap(),
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
    let (bob_public_key, bob_private_key) = generate_bn254_key();
    let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());
    let alice_signature = bn254_sign(&bob_private_key, "alice_near".to_string().as_bytes());

    // bob registers node
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        bob_signature.to_compressed().unwrap(),
    );

    // alice registers node with duplicated key
    testing_env!(get_context_with_deposit("alice_near".to_string()));
    contract.register_node(
        "1.1.1.1:8080".to_string(),
        bob_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );
}

#[test]
fn deposit_withdraw() {
    let mut contract = new_contract();
    let deposit_amount = U128(INIT_MINIMUM_STAKE);

    // DAO transfers tokens to alice
    testing_env!(get_context_with_deposit("dao_near".to_string(),));
    contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
    testing_env!(get_context_for_ft_transfer("dao_near".to_string()));
    contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);

    // alice registers node
    let (alice_public_key, alice_private_key) = generate_bn254_key();
    let alice_signature = bn254_sign(&alice_private_key, "alice_near".to_string().as_bytes());
    testing_env!(get_context_with_deposit("alice_near".to_string()));
    contract.register_node(
        "0.0.0.0:8080".to_string(),
        alice_public_key.to_compressed().unwrap(),
        alice_signature.to_compressed().unwrap(),
    );

    // alice deposits into pool
    testing_env!(get_context_with_deposit("alice_near".to_string()));
    contract.deposit(deposit_amount);

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
    testing_env!(get_context("alice_near".to_string()));
    contract.withdraw(deposit_amount);

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
