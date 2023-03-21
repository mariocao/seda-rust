use std::collections::HashMap;

use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env, AccountId};
use rand::Rng;

use super::test_utils::{
    bn254_sign,
    get_context_for_ft_transfer,
    get_context_for_post_signed_batch,
    get_context_with_deposit,
    get_context_with_deposit_at_block,
    new_contract,
};
use crate::{
    consts::INIT_MINIMUM_STAKE,
    tests::test_utils::{bn254_sign_aggregate, make_test_account, TestAccount},
};

#[test]
fn post_signed_batch() {
    let mut contract = new_contract();
    let dao = make_test_account("dao_near".to_string());

    let deposit_amount = U128(INIT_MINIMUM_STAKE);
    let test_acc = make_test_account("test_near".to_string());

    // post some data requests to the accumulator
    testing_env!(get_context_with_deposit(test_acc.clone()));
    contract.post_data_request("data_request_1".to_string());
    contract.post_data_request("data_request_2".to_string());
    contract.post_data_request("data_request_3".to_string());

    let mut test_accounts: HashMap<AccountId, TestAccount> = HashMap::new();
    let num_of_nodes = 20;
    for x in 0..num_of_nodes {
        let acc_str = format!("{x:}_near");
        let acc = make_test_account(acc_str.clone());

        // transfer some tokens
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        test_accounts.insert(acc_str.parse().unwrap(), acc.clone());
        let sig = bn254_sign(&acc.bn254_private_key.clone(), acc_str.as_bytes());

        testing_env!(get_context_with_deposit(acc.clone()));
        // register nodes
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            acc.bn254_public_key.to_compressed().unwrap(),
            sig.to_compressed().unwrap(),
        );
        // deposit into contract
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.clone().into());
    }

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(test_acc.clone(), 1000000));
    contract.process_epoch();

    // assert we have committees for this epoch and the next 2
    assert_eq!(contract.get_committees().len(), 3);

    // assert each committee has config.committee_size members
    contract
        .get_committees()
        .into_iter()
        .for_each(|comittee| assert_eq!(comittee.len() as u64, contract.config.committee_size));

    // get the merkle root (for all nodes to sign)
    let merkle_root = contract.compute_merkle_root();

    // gather the chosen committee test accounts for signing
    let chosen_committee_account_ids = contract.get_committees().first().unwrap().clone();
    let chosen_committee: Vec<TestAccount> = chosen_committee_account_ids
        .iter()
        .map(|acc_id| test_accounts.get(acc_id).unwrap().clone())
        .collect();
    let (agg_signature, agg_public_key) = bn254_sign_aggregate(chosen_committee, &merkle_root);

    assert_eq!(contract.num_batches, 0);

    // find the slot leader
    testing_env!(get_context_for_post_signed_batch(test_acc.clone()));
    let slot_leader_account_id = contract.get_current_slot_leader();
    let slot_leader_test_account = test_accounts.get(&slot_leader_account_id).unwrap();

    // sign and post the batch
    testing_env!(get_context_for_post_signed_batch(slot_leader_test_account.clone()));
    let num_batches = contract.num_batches;
    let leader_sig = bn254_sign(
        &slot_leader_test_account.bn254_private_key,
        &contract.last_generated_random_number.to_le_bytes(),
    );
    contract.post_signed_batch(
        agg_signature.to_compressed().unwrap(),
        agg_public_key.to_compressed().unwrap(),
        chosen_committee_account_ids,
        leader_sig.to_compressed().unwrap(),
    );
    assert_eq!(contract.num_batches, num_batches + 1);
}

#[test]
#[should_panic(expected = "Invalid slot leader signature")]
fn post_signed_batch_with_wrong_leader_sig() {
    let mut contract = new_contract();
    let deposit_amount = U128(INIT_MINIMUM_STAKE);
    let dao = make_test_account("dao_near".to_string());

    let test_acc = make_test_account("test_near".to_string());

    // post some data requests to the accumulator
    testing_env!(get_context_with_deposit(test_acc.clone()));
    contract.post_data_request("data_request_1".to_string());
    contract.post_data_request("data_request_2".to_string());
    contract.post_data_request("data_request_3".to_string());

    let mut test_accounts: HashMap<AccountId, TestAccount> = HashMap::new();
    let num_of_nodes = 20;
    for x in 0..num_of_nodes {
        let acc_str = format!("{x:}_near");
        let acc = make_test_account(acc_str.clone());
        // transfer some tokens
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        test_accounts.insert(acc_str.parse().unwrap(), acc.clone());
        let sig = bn254_sign(&acc.bn254_private_key.clone(), acc_str.as_bytes());

        testing_env!(get_context_with_deposit(acc.clone()));
        // register nodes
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            acc.bn254_public_key.to_compressed().unwrap(),
            sig.to_compressed().unwrap(),
        );
        // deposit into contract
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.clone().into());
    }

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(test_acc.clone(), 1000000));
    contract.process_epoch();

    // get the merkle root (for all nodes to sign)
    let merkle_root = contract.compute_merkle_root();

    // gather the chosen committee test accounts for signing
    let chosen_committee_account_ids = contract.get_committees().first().unwrap().clone();
    let chosen_committee: Vec<TestAccount> = chosen_committee_account_ids
        .iter()
        .map(|acc_id| test_accounts.get(acc_id).unwrap().clone())
        .collect();
    let (agg_signature, agg_public_key) = bn254_sign_aggregate(chosen_committee, &merkle_root);

    // find the slot leader
    testing_env!(get_context_for_post_signed_batch(test_acc.clone()));
    let slot_leader_account_id = contract.get_current_slot_leader();
    let slot_leader_test_account = test_accounts.get(&slot_leader_account_id).unwrap();

    // sign and post the batch with an invalid signature
    testing_env!(get_context_for_post_signed_batch(slot_leader_test_account.clone()));
    let num_batches = contract.num_batches;
    let mut rng = rand::thread_rng();
    let random_seed = rng.gen::<u64>();
    let invalid_leader_sig = bn254_sign(&slot_leader_test_account.bn254_private_key, &random_seed.to_le_bytes());
    contract.post_signed_batch(
        agg_signature.to_compressed().unwrap(),
        agg_public_key.to_compressed().unwrap(),
        chosen_committee_account_ids,
        invalid_leader_sig.to_compressed().unwrap(),
    );
    assert_eq!(contract.num_batches, num_batches + 1);
}
