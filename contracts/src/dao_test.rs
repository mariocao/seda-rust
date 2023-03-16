use near_sdk::testing_env;

use super::test_utils::{get_context, make_test_account, new_contract};
use crate::dao::UpdateConfig;

#[test]
fn update_config() {
    let mut contract = new_contract();
    let dao = make_test_account("dao_near".to_string());
    testing_env!(get_context(dao));
    contract.update_config(UpdateConfig::MinimumStake, 100);
}

#[test]
#[should_panic(expected = "Only DAO can call this method")]
fn update_config_wrong_account() {
    let mut contract = new_contract();
    let bob = make_test_account("bob_near".to_string());
    testing_env!(get_context(bob));
    contract.update_config(UpdateConfig::MinimumStake, 100);
}
