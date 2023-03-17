use bn254::{PrivateKey, PublicKey, Signature, ECDSA};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{json_types::U128, test_utils::VMContextBuilder, AccountId, Balance, VMContext};
use rand::{
    distributions::{Alphanumeric, DistString},
    Rng,
};

use crate::{
    consts::{DATA_IMAGE_SVG_ICON, INITIAL_SUPPLY},
    MainchainContract,
};

const TEST_DEPOSIT_AMOUNT: Balance = 618_720_000_000_000_000_000_000; // enough deposit to cover storage for all functions that require it

pub fn new_contract() -> MainchainContract {
    let mut rng = rand::thread_rng();
    let random_seed = rng.gen::<u64>();
    let committee_size = 2;
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
        random_seed,
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
        account_id: account_id.to_string().try_into().unwrap(),
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

pub fn bn254_sign(test_account: TestAccount, message: &[u8]) -> Signature {
    ECDSA::sign(message, &test_account.bn254_private_key).unwrap()
}
