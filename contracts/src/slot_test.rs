use near_sdk::testing_env;

use super::test_utils::{get_context_at_block, new_contract};
use crate::consts::{NEAR_BLOCKS_PER_SEDA_SLOT, SLOTS_PER_EPOCH};

#[test]
fn get_current_slot() {
    let contract = new_contract();

    testing_env!(get_context_at_block(0)); // block 0
    assert_eq!(contract.get_current_slot(), 0); // slot 0

    testing_env!(get_context_at_block(NEAR_BLOCKS_PER_SEDA_SLOT - 1)); // block 9
    assert_eq!(contract.get_current_slot(), 0); // slot 0

    testing_env!(get_context_at_block(NEAR_BLOCKS_PER_SEDA_SLOT)); // block 10
    assert_eq!(contract.get_current_slot(), 1); // slot 1
}

#[test]
fn get_current_epoch() {
    let contract = new_contract();

    testing_env!(get_context_at_block(0)); // block 0
    assert_eq!(contract.get_current_epoch(), 0); // epoch 0

    testing_env!(get_context_at_block(NEAR_BLOCKS_PER_SEDA_SLOT * SLOTS_PER_EPOCH - 1)); // block 319
    assert_eq!(contract.get_current_slot(), 31); // slot 31
    assert_eq!(contract.get_current_epoch(), 0); // epoch 0

    testing_env!(get_context_at_block(NEAR_BLOCKS_PER_SEDA_SLOT * SLOTS_PER_EPOCH)); // block 320
    assert_eq!(contract.get_current_slot(), 32); // slot 32
    assert_eq!(contract.get_current_epoch(), 1); // epoch 1
}
