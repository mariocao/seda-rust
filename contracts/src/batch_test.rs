use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{
    bn254_sign,
    generate_bn254_key,
    get_context_for_ft_transfer,
    get_context_for_post_signed_batch,
    get_context_with_deposit,
    get_context_with_deposit_at_block,
    new_contract,
};
use crate::consts::INIT_MINIMUM_STAKE;

// #[test]
// fn post_signed_batch() {
//     let mut contract = new_contract();
//     let deposit_amount = U128(100_000_000_000_000_000_000_000);

//     // post some data requests to the accumulator
//     testing_env!(get_context_with_deposit("bob_near".to_string()));
//     contract.post_data_request("data_request_1".to_string());
//     contract.post_data_request("data_request_2".to_string());
//     contract.post_data_request("data_request_3".to_string());

//     // transfer some tokens to alice and bob
//     testing_env!(get_context_with_deposit("dao_near".to_string(),));
//     contract.storage_deposit(Some("alice_near".to_string().try_into().unwrap()), None);
//     contract.storage_deposit(Some("bob_near".to_string().try_into().unwrap()), None);
//     testing_env!(get_context_for_ft_transfer("dao_near".to_string()));
//     contract.ft_transfer("alice_near".to_string().try_into().unwrap(), deposit_amount, None);
//     contract.ft_transfer("bob_near".to_string().try_into().unwrap(), deposit_amount, None);

//     // register nodes for alice and bob
//     let (alice_public_key, alice_private_key) = generate_bn254_key();
//     let (bob_public_key, bob_private_key) = generate_bn254_key();
//     let alice_signature = bn254_sign(&alice_private_key, "alice_near".to_string().as_bytes());
//     let bob_signature = bn254_sign(&bob_private_key, "bob_near".to_string().as_bytes());
//     testing_env!(get_context_with_deposit("bob_near".to_string()));
//     contract.register_node(
//         "0.0.0.0:8080".to_string(),
//         bob_public_key.to_compressed().unwrap(),
//         bob_signature.to_compressed().unwrap(),
//     );
//     testing_env!(get_context_with_deposit("alice_near".to_string()));
//     contract.register_node(
//         "1.1.1.1:8080".to_string(),
//         alice_public_key.to_compressed().unwrap(),
//         alice_signature.to_compressed().unwrap(),
//     );

//     // alice and bob deposit into contract
//     testing_env!(get_context_with_deposit("alice_near".to_string()));
//     contract.deposit(deposit_amount);
//     testing_env!(get_context_with_deposit("bob_near".to_string()));
//     contract.deposit(deposit_amount);

//     // time travel and activate nodes
//     testing_env!(get_context_with_deposit_at_block("bob_near".to_string(), 1000000));
//     contract.process_epoch();

//     // assert we have committees for this epoch and the next 2
//     assert_eq!(contract.get_committees().len(), 3);

//     // assert each committee has SLOTS_PER_EPOCH members
//     for committee in contract.get_committees() {
//         assert_eq!(committee.len() as u64, SLOTS_PER_EPOCH);
//     }

//     // get the merkle root (for all nodes to sign)
//     let merkle_root = contract.compute_merkle_root();

//     // alice and bob sign the merkle root
//     let alice_merkle_root_signature = bn254_sign(&alice_private_key, &merkle_root);
//     let bob_merkle_root_signature = bn254_sign(&bob_private_key, &merkle_root);

//     // aggregate the signatures
//     let agg_public_key = alice_public_key + bob_public_key;
//     let agg_signature = alice_merkle_root_signature + bob_merkle_root_signature;

//     // alice posts the signed batch
//     testing_env!(get_context_for_post_signed_batch("alice_near".to_string()));
//     contract.post_signed_batch(
//         agg_signature.to_compressed().unwrap(),
//         agg_public_key.to_compressed().unwrap(),
//         [
//             "alice_near".to_string().try_into().unwrap(),
//             "bob_near".to_string().try_into().unwrap(),
//         ]
//         .to_vec(),
//     )
// }
