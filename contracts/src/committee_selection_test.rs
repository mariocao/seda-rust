use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{get_context_view, get_context_with_deposit, new_contract};
use crate::{
    consts::INIT_MINIMUM_STAKE,
    tests::test_utils::{
        bn254_sign,
        get_context_for_ft_transfer,
        get_context_with_deposit_at_block,
        make_test_account,
    },
};

#[test]
fn test_committee_selection() {
    let mut contract = new_contract();
    let dao = make_test_account("dao_near".to_string());
    let bob = make_test_account("bob_near".to_string());

    let deposit_amount = U128(INIT_MINIMUM_STAKE);
    let num_of_nodes = 20;
    for x in 1..num_of_nodes {
        let acc_str = format!("{x:}_near");
        let acc = make_test_account(acc_str.clone());

        let sig = bn254_sign(&acc.bn254_private_key, acc_str.as_bytes());
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        // register node
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            acc.bn254_public_key.to_uncompressed().unwrap(),
            sig.to_uncompressed().unwrap(),
        );
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.into_bytes());

        // check owner and multi_addr
        testing_env!(get_context_view());
        assert_eq!(
            "0.0.0.0:8080".to_string(),
            contract.get_node(acc.account_id).unwrap().multi_addr
        );
    }

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(bob, 1000000));
    contract.process_epoch();

    // assert we have committees for this epoch and the next 2
    assert_ne!(contract.get_committee(contract.get_current_epoch()).unwrap().len(), 0);
    assert_ne!(
        contract.get_committee(contract.get_current_epoch() + 1).unwrap().len(),
        0
    );
    assert_ne!(
        contract.get_committee(contract.get_current_epoch() + 2).unwrap().len(),
        0
    );
    assert_eq!(contract.get_committee(contract.get_current_epoch() + 3), None);

    // assert each committee has DEFAULT_COMMITTEE_SIZE members
    for i in 0..3 {
        assert_eq!(
            contract.get_committee(contract.get_current_epoch() + i).unwrap().len() as u64,
            contract.config.committee_size
        );
    }
}
