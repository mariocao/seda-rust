use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{
    bn254_sign,
    call_random_data_request,
    get_context_with_deposit_at_block,
    make_register_test_accounts,
    new_contract_with_committee_size,
    ONE_EPOCH,
};
use crate::{
    consts::{INIT_MINIMUM_STAKE, SLOTS_PER_EPOCH},
    tests::test_utils::{bn254_sign_aggregate, get_context_at_block, make_test_account, TestAccount, ONE_SLOT},
};

// Realistic test parameters
// const MAX_DATA_REQUESTS: u32 = 128;
// const COMMITTEE_SIZE: u64 = 128;
// const MAX_NODES: u64 = COMMITTEE_SIZE * 2;
// const TEST_EPOCHS: u64 = 5;

// Fast test parameters
const MAX_DATA_REQUESTS: u32 = 5;
const COMMITTEE_SIZE: u64 = 3;
const MAX_NODES: u64 = COMMITTEE_SIZE;
const TEST_EPOCHS: u64 = 1;

/// Simulates committee selection and posting batches to a set amount of epochs
// TODO: test registering/depositing/withdrawing nodes during an epoch
#[test]
fn integration_test_1() {
    let mut contract = new_contract_with_committee_size(COMMITTEE_SIZE);
    let mut block_number = 0;
    let deposit_amount = U128(INIT_MINIMUM_STAKE);
    let dao = make_test_account("dao_near".to_string());
    let test_acc = make_test_account("test_near".to_string());

    // register a random amount of nodes, with a minimum of the bootstrapping
    let test_accounts = make_register_test_accounts(
        &mut contract,
        &dao,
        COMMITTEE_SIZE as usize,
        MAX_NODES as usize,
        deposit_amount,
    );

    // time travel to the beginning of an epoch and activate nodes
    block_number += ONE_EPOCH * 2;
    println!("Block number: {}", block_number);
    testing_env!(get_context_with_deposit_at_block(test_acc.clone(), block_number));
    contract.process_epoch();

    // loop through a predefined number of epochs
    for _ in 0..TEST_EPOCHS {
        testing_env!(get_context_at_block(block_number));
        let epoch = contract.get_current_epoch();
        println!("\nEpoch: {}", epoch);

        // loop through every slot in this epoch and post a batch
        for _ in 0..SLOTS_PER_EPOCH {
            testing_env!(get_context_at_block(block_number));
            assert_eq!(contract.get_current_epoch(), epoch, "Epoch changed unexpectedly");

            // post some data requests to the accumulator
            testing_env!(get_context_with_deposit_at_block(test_acc.clone(), block_number));
            let num_data_requests = call_random_data_request(&mut contract, 0, MAX_DATA_REQUESTS as usize);
            if num_data_requests == 0 {
                println!("No data requests/batch posted for this slot");
                block_number += ONE_SLOT;
                continue;
            }

            // get the merkle root (for all nodes to sign)
            let merkle_root = contract.compute_merkle_root().merkle_root;

            // gather the chosen committee test accounts for signing
            let chosen_committee_account_ids = contract.get_committee(contract.get_current_epoch()).unwrap();
            let chosen_committee: Vec<TestAccount> = chosen_committee_account_ids
                .iter()
                .map(|acc_id| test_accounts.get(acc_id).unwrap().clone())
                .collect();
            let (agg_signature, agg_public_key) = bn254_sign_aggregate(chosen_committee, &merkle_root);

            // find the slot leader
            testing_env!(get_context_at_block(block_number));
            let slot_leader_account_id = contract.get_current_slot_leader().unwrap();
            let slot_leader_test_account = test_accounts.get(&slot_leader_account_id).unwrap();
            println!("Slot leader {} at block {}", slot_leader_account_id, block_number);

            // sign and post the batch
            let num_batches = contract.num_batches;
            testing_env!(get_context_with_deposit_at_block(
                slot_leader_test_account.clone(),
                block_number
            ));
            let leader_sig = bn254_sign(
                &slot_leader_test_account.bn254_private_key,
                &contract.last_generated_random_number.to_le_bytes(),
            );
            contract.post_signed_batch(
                agg_signature.to_uncompressed().unwrap(),
                agg_public_key.to_uncompressed().unwrap(),
                chosen_committee_account_ids,
                leader_sig.to_uncompressed().unwrap(),
            );
            assert_eq!(contract.num_batches, num_batches + 1);

            let slot = contract.get_current_slot();
            println!("Posted batch for slot {}", slot);

            // time travel to the next slot
            block_number += ONE_SLOT;
        }
    }
}
