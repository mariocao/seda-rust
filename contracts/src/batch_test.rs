use std::collections::HashMap;

use bn254::{PrivateKey, PublicKey};
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
    tests::test_utils::{generate_bn254_key, make_test_account},
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

    let mut test_accounts: HashMap<AccountId, (PublicKey, PrivateKey)> = HashMap::new();
    let num_of_nodes = 20;
    for x in 0..num_of_nodes {
        let acc_str = format!("{x:}_near");
        let acc = make_test_account(acc_str.clone());

        // transfer some tokens
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        let (pk, sk) = generate_bn254_key();
        test_accounts.insert(acc_str.parse().unwrap(), (pk, sk.clone()));
        let sig = bn254_sign(&sk.clone(), acc_str.as_bytes());

        testing_env!(get_context_with_deposit(acc.clone()));
        // register nodes
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            pk.to_compressed().unwrap(),
            sig.to_compressed().unwrap(),
        );
        // deposit into contract
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.clone().into());
    }

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(test_acc, 1000000));
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
    let chosen_committee = contract.get_committees().first().unwrap().clone();
    let (pk, sk) = &test_accounts.get(chosen_committee.first().unwrap()).unwrap();
    let acc_merkle_root_signature = bn254_sign(sk, &merkle_root);

    let mut agg_public_key = *pk;
    let mut agg_signature = acc_merkle_root_signature;
    for index in chosen_committee.iter().skip(1) {
        let (pk, sk) = &test_accounts.get(index).unwrap();
        let acc_merkle_root_signature = bn254_sign(sk, &merkle_root);
        agg_public_key = agg_public_key + *pk;
        agg_signature = agg_signature + acc_merkle_root_signature;
    }
    assert_eq!(contract.num_batches, 0);
    for index in 0..num_of_nodes {
        let acc_str = format!("{index:}_near");
        let acc = make_test_account(acc_str);

        testing_env!(get_context_for_post_signed_batch(acc.clone()));
        let slot_leader = contract.get_current_slot_leader();

        if slot_leader == acc.account_id.clone() {
            let last_generated_random = contract.last_generated_random_number;
            let leader_sig = bn254_sign(
                &test_accounts.get(&acc.account_id).unwrap().1,
                &last_generated_random.to_le_bytes(),
            );
            contract.post_signed_batch(
                agg_signature.to_compressed().unwrap(),
                agg_public_key.to_compressed().unwrap(),
                chosen_committee.clone(),
                leader_sig.to_compressed().unwrap(),
            );
            assert_ne!(last_generated_random, contract.last_generated_random_number);
            assert_eq!(contract.num_batches, 1);

            break;
        }
    }
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

    let mut test_accounts: HashMap<AccountId, (PublicKey, PrivateKey)> = HashMap::new();
    let num_of_nodes = 20;
    for x in 0..num_of_nodes {
        let acc_str = format!("{x:}_near");
        let acc = make_test_account(acc_str.clone());
        // transfer some tokens
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        let (pk, sk) = generate_bn254_key();
        test_accounts.insert(acc_str.parse().unwrap(), (pk, sk.clone()));
        let sig = bn254_sign(&sk.clone(), acc_str.as_bytes());

        testing_env!(get_context_with_deposit(acc.clone()));
        // register nodes
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            pk.to_compressed().unwrap(),
            sig.to_compressed().unwrap(),
        );
        // deposit into contract
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.clone().into());
    }

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(test_acc, 1000000));
    contract.process_epoch();

    // get the merkle root (for all nodes to sign)
    let merkle_root = contract.compute_merkle_root();
    let chosen_committee = contract.get_committees().first().unwrap().clone();
    let (pk, sk) = &test_accounts.get(chosen_committee.first().unwrap()).unwrap();
    let acc_merkle_root_signature = bn254_sign(sk, &merkle_root);

    let mut agg_public_key = *pk;
    let mut agg_signature = acc_merkle_root_signature;
    for index in chosen_committee.iter().skip(1) {
        let (pk, sk) = &test_accounts.get(index).unwrap();
        let acc_merkle_root_signature = bn254_sign(sk, &merkle_root);
        agg_public_key = agg_public_key + *pk;
        agg_signature = agg_signature + acc_merkle_root_signature;
    }
    for index in 0..num_of_nodes {
        let acc_str = format!("{index:}_near");
        let acc = make_test_account(acc_str);
        testing_env!(get_context_for_post_signed_batch(acc.clone()));
        let slot_leader = contract.get_current_slot_leader();

        if slot_leader == acc.account_id.clone() {
            let mut rng = rand::thread_rng();
            let random_seed = rng.gen::<u64>();
            let invalid_leader_sig = bn254_sign(
                &test_accounts.get(&acc.account_id).unwrap().1,
                &random_seed.to_le_bytes(),
            );
            contract.post_signed_batch(
                agg_signature.to_compressed().unwrap(),
                agg_public_key.to_compressed().unwrap(),
                chosen_committee.clone(),
                invalid_leader_sig.to_compressed().unwrap(),
            );
            break;
        }
    }
}
