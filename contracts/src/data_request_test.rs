use near_sdk::testing_env;

use super::test_utils::{get_context, get_context_with_deposit, make_test_account, new_contract};

#[test]
fn post_data_request() {
    let mut contract = new_contract();
    let bob = make_test_account("bob_near".to_string());

    // post data request
    testing_env!(get_context_with_deposit(bob.clone()));
    contract.post_data_request("data_request_1".to_string());
    contract.post_data_request("data_request_2".to_string());
    contract.post_data_request("data_request_3".to_string());

    // compute merkle root
    testing_env!(get_context(bob));
    contract.compute_merkle_root();
}

#[should_panic(expected = "Insufficient storage, need 670000000000000000000")]
#[test]
fn post_data_request_no_deposit() {
    let mut contract = new_contract();
    let bob = make_test_account("bob_near".to_string());

    // post data request
    testing_env!(get_context(bob));
    contract.post_data_request("data_request_1".to_string());
}
