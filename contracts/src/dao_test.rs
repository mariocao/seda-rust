use near_sdk::testing_env;

use super::test_utils::{bob, dao, get_context, new_contract};
use crate::dao::UpdateConfig;

#[test]
fn update_config() {
    let mut contract = new_contract();
    testing_env!(get_context(dao()));
    contract.update_config(UpdateConfig::MinimumStake, 100);
}

#[test]
#[should_panic(expected = "Only DAO can call this method")]
fn update_config_wrong_account() {
    let mut contract = new_contract();
    testing_env!(get_context(bob()));
    contract.update_config(UpdateConfig::MinimumStake, 100);
}
