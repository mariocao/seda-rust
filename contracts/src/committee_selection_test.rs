use bn254::{PrivateKey, PublicKey, ECDSA};
use near_contract_standards::{fungible_token::core::FungibleTokenCore, storage_management::StorageManagement};
use near_sdk::{json_types::U128, testing_env};

use super::test_utils::{get_context_view, get_context_with_deposit, new_contract};
use crate::{
    consts::INIT_MINIMUM_STAKE,
    tests::test_utils::{get_context_for_ft_transfer, get_context_with_deposit_at_block},
};

fn get_x_key_and_signature(x: &str) -> (Vec<u8>, Vec<u8>) {
    let rng = &mut rand::thread_rng();
    let sk = PrivateKey::random(rng);
    let pk = PublicKey::from_private_key(&sk).to_compressed().unwrap();

    let msg = x.as_bytes();
    let signature = ECDSA::sign(msg, &sk).unwrap().to_compressed().unwrap();
    (pk, signature)
}

#[test]
fn test_committee_selection() {
    let mut contract = new_contract();
    let deposit_amount = U128(INIT_MINIMUM_STAKE);

    for x in 1..200 {
        let acc = format!("{x:}_near");
        let (pk, sig) = get_x_key_and_signature(&acc);
        testing_env!(get_context_with_deposit("dao_near".to_string(),));
        contract.storage_deposit(Some(acc.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer("dao_near".to_string()));
        contract.ft_transfer(acc.clone().try_into().unwrap(), deposit_amount, None);

        // register node
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.register_node("0.0.0.0:8080".to_string(), pk, sig);
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount);

        // check owner and multi_addr
        testing_env!(get_context_view());
        assert_eq!(
            "0.0.0.0:8080".to_string(),
            contract.get_node(acc.try_into().unwrap()).unwrap().multi_addr
        );
    }
    let acc = "1_near".to_string();

    // time travel and activate nodes
    testing_env!(get_context_with_deposit_at_block(acc, 1000000));
    contract.process_epoch();

    // assert we have committees for this epoch and the next 2
    assert_eq!(contract.get_committees().len(), 3);

    // assert each committee has DEFAULT_COMMITTEE_SIZE members
    for committee in contract.get_committees() {
        assert_eq!(committee.len() as u64, contract.config.committee_size);
    }
}
