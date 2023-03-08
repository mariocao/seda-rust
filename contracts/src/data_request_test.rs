use near_sdk::testing_env;

use super::test_utils::{get_context, get_context_with_deposit, new_contract};

#[test]
fn post_data_request() {
    let mut contract = new_contract();

    // post data request
    testing_env!(get_context_with_deposit("bob_near".to_string()));
    contract.post_data_request("data_request_1".to_string());
    contract.post_data_request("data_request_2".to_string());
    contract.post_data_request("data_request_3".to_string());

    // compute merkle root
    testing_env!(get_context("bob_near".to_string()));
    contract.compute_merkle_root();
}

#[should_panic(expected = "Insufficient storage, need 670000000000000000000")]
#[test]
fn post_data_request_no_deposit() {
    let mut contract = new_contract();

    // post data request
    testing_env!(get_context("bob_near".to_string()));
    contract.post_data_request("data_request_1".to_string());
}
