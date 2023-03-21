use std::collections::HashMap;

use bn254::{PrivateKey, PublicKey, Signature, ECDSA};
use near_contract_standards::{
    fungible_token::{
        core::FungibleTokenCore,
        metadata::{FungibleTokenMetadata, FT_METADATA_SPEC},
    },
    storage_management::StorageManagement,
};
use near_sdk::{json_types::U128, test_utils::VMContextBuilder, testing_env, AccountId, Balance, VMContext};
use rand::Rng;

use crate::{
    consts::{DATA_IMAGE_SVG_ICON, INITIAL_SUPPLY},
    MainchainContract,
};

pub const ONE_EPOCH: u64 = 320; // one SEDA epoch in terms of NEAR blocks
pub const ONE_SLOT: u64 = 10; // one SEDA slot in terms of NEAR blocks
const TEST_DEPOSIT_AMOUNT: Balance = 618_720_000_000_000_000_000_000; // enough deposit to cover storage for all functions that require it
pub const TEST_COMMITTEE_SIZE: u64 = 2;

pub fn new_contract() -> MainchainContract {
    new_contract_with_committee_size(TEST_COMMITTEE_SIZE)
}

pub fn new_contract_with_committee_size(committee_size: u64) -> MainchainContract {
    let mut rng = rand::thread_rng();
    let random_seed = rng.gen::<u64>();
    MainchainContract::new(
        "dao_near".to_string().try_into().unwrap(),
        U128(INITIAL_SUPPLY),
        FungibleTokenMetadata {
            spec:           FT_METADATA_SPEC.to_string(),
            name:           "Example NEAR fungible token".to_string(),
            symbol:         "EXAMPLE".to_string(),
            icon:           Some(DATA_IMAGE_SVG_ICON.to_string()),
            reference:      None,
            reference_hash: None,
            decimals:       24,
        },
        committee_size,
        random_seed.into(),
    )
}

#[derive(Clone)]
pub struct TestAccount {
    pub account_id:         AccountId,
    pub ed25519_public_key: near_sdk::PublicKey,
    pub bn254_private_key:  PrivateKey,
    pub bn254_public_key:   PublicKey,
}

pub fn make_test_account(account_id: String) -> TestAccount {
    // reroll until we get a valid ed25519_public_key
    // TODO: this is ugly but works for now
    let truncated;
    loop {
        let rng = &mut rand::thread_rng();
        let bytes = rng.gen::<[u8; 32]>();
        let encoded = bs58::encode(bytes).into_string();
        if encoded.len() < 44 {
            continue;
        }
        truncated = encoded[..44].to_string();
        break;
    }

    let ed25519_public_key: near_sdk::PublicKey = truncated.parse().unwrap();

    let rng = &mut rand::thread_rng();
    let bn254_private_key = PrivateKey::random(rng);
    let bn254_public_key = PublicKey::from_private_key(&bn254_private_key);

    TestAccount {
        account_id: account_id.try_into().unwrap(),
        ed25519_public_key,
        bn254_private_key,
        bn254_public_key,
    }
}

pub fn get_context_view() -> VMContext {
    VMContextBuilder::new().is_view(true).build()
}
pub fn get_context(test_account: TestAccount) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(test_account.account_id.clone())
        .signer_account_pk(test_account.ed25519_public_key)
        .predecessor_account_id(test_account.account_id)
        .is_view(false)
        .build()
}
pub fn get_context_for_post_signed_batch(test_account: TestAccount) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(test_account.account_id)
        .signer_account_pk(test_account.ed25519_public_key)
        .is_view(false)
        .attached_deposit(TEST_DEPOSIT_AMOUNT)
        .block_index(100000000)
        .build()
}
pub fn get_context_with_deposit(test_account: TestAccount) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(test_account.account_id)
        .signer_account_pk(test_account.ed25519_public_key)
        .is_view(false)
        .attached_deposit(TEST_DEPOSIT_AMOUNT)
        .build()
}
pub fn get_context_for_ft_transfer(test_account: TestAccount) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(test_account.account_id.clone())
        .signer_account_pk(test_account.ed25519_public_key)
        .predecessor_account_id(test_account.account_id)
        .is_view(false)
        .attached_deposit(1)
        .build()
}
pub fn get_context_at_block(block_index: u64) -> VMContext {
    VMContextBuilder::new().block_index(block_index).is_view(true).build()
}
pub fn get_context_with_deposit_at_block(test_account: TestAccount, block_index: u64) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(test_account.account_id.clone())
        .signer_account_pk(test_account.ed25519_public_key)
        .is_view(false)
        .attached_deposit(TEST_DEPOSIT_AMOUNT)
        .block_index(block_index)
        .build()
}

pub fn bn254_sign(private_key: &PrivateKey, message: &[u8]) -> Signature {
    ECDSA::sign(message, private_key).unwrap()
}

pub fn bn254_sign_aggregate(accounts: Vec<TestAccount>, message: &[u8]) -> (Signature, PublicKey) {
    // initialize with first account
    let mut agg_signature = bn254_sign(&accounts[0].bn254_private_key, &message);
    let mut agg_public_key = accounts[0].bn254_public_key.clone();

    // aggregate the rest
    for account in accounts.iter().skip(1) {
        let signature = bn254_sign(&account.bn254_private_key, &message);
        agg_public_key = agg_public_key + account.bn254_public_key.clone();
        agg_signature = agg_signature + signature;
    }

    (agg_signature, agg_public_key)
}

pub fn call_random_data_request(
    contract: &mut MainchainContract,
    min_data_requests: usize,
    max_data_requests: usize,
) -> usize {
    let mut rng = rand::thread_rng();
    let num_data_requests = rng.gen_range(min_data_requests..=max_data_requests);

    for i in 0..num_data_requests {
        let data_request_name = format!("data_request_{}", i);
        contract.post_data_request(data_request_name);
    }
    println!("Posted {} data requests", num_data_requests);
    num_data_requests
}

pub fn make_register_test_accounts(
    contract: &mut MainchainContract,
    dao: &TestAccount,
    min_nodes: usize,
    max_nodes: usize,
    deposit_amount: U128,
) -> HashMap<AccountId, TestAccount> {
    let mut test_accounts: HashMap<AccountId, TestAccount> = HashMap::new();

    let mut rng = rand::thread_rng();
    let num_of_nodes = rng.gen_range(min_nodes..=max_nodes);

    for x in 0..num_of_nodes {
        let acc_str = format!("{x}_near", x = x);
        let acc = make_test_account(acc_str.clone());

        // transfer some tokens
        testing_env!(get_context_with_deposit(dao.clone(),));
        contract.storage_deposit(Some(acc_str.clone().try_into().unwrap()), None);
        testing_env!(get_context_for_ft_transfer(dao.clone()));
        contract.ft_transfer(acc_str.clone().try_into().unwrap(), deposit_amount, None);

        test_accounts.insert(acc_str.parse().unwrap(), acc.clone());
        let sig = bn254_sign(&acc.bn254_private_key.clone(), acc_str.as_bytes());

        // register nodes
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.register_node(
            "0.0.0.0:8080".to_string(),
            acc.bn254_public_key.to_compressed().unwrap(),
            sig.to_compressed().unwrap(),
        );
        // deposit into contract
        testing_env!(get_context_with_deposit(acc.clone()));
        contract.deposit(deposit_amount, acc.ed25519_public_key.clone().into());
    }
    println!("Created {} test accounts", num_of_nodes);
    test_accounts
}
