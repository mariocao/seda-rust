use bn254::{PrivateKey, PublicKey, Signature, ECDSA};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{json_types::U128, test_utils::VMContextBuilder, VMContext};
use rand::distributions::{Alphanumeric, DistString};

use crate::{
    consts::{DATA_IMAGE_SVG_ICON, INITIAL_SUPPLY},
    MainchainContract,
};

// TODO: only compile this for tests

pub fn new_contract() -> MainchainContract {
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
    )
}

pub fn get_context_view() -> VMContext {
    VMContextBuilder::new().is_view(true).build()
}
pub fn get_context(signer_account_id: String) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(signer_account_id.parse().unwrap())
        .predecessor_account_id(signer_account_id.parse().unwrap())
        .is_view(false)
        .build()
}
pub fn get_context_for_post_signed_batch(signer_account_id: String) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(signer_account_id.parse().unwrap())
        .is_view(false)
        .attached_deposit(4_110_000_000_000_000_000_000)
        .block_index(100000000)
        .build()
}
pub fn get_context_with_deposit(signer_account_id: String) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(signer_account_id.parse().unwrap())
        .is_view(false)
        .attached_deposit(4_110_000_000_000_000_000_000) // required for post_data_request()
        .build()
}
pub fn get_context_for_ft_transfer(signer_account_id: String) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id(signer_account_id.parse().unwrap())
        .predecessor_account_id(signer_account_id.parse().unwrap())
        .is_view(false)
        .attached_deposit(1)
        .build()
}
pub fn get_context_at_block(block_index: u64) -> VMContext {
    VMContextBuilder::new().block_index(block_index).is_view(true).build()
}

pub fn generate_bn254_key() -> (PublicKey, PrivateKey) {
    let random_hex_string = hex::encode(Alphanumeric.sample_string(&mut rand::thread_rng(), 32));
    let private_key_bytes = hex::decode(random_hex_string).unwrap();

    let private_key = PrivateKey::try_from(private_key_bytes.as_ref()).unwrap();
    let public_key = PublicKey::from_private_key(&private_key);

    (public_key, private_key)
}

pub fn bn254_sign(private_key: &PrivateKey, message: &[u8]) -> Signature {
    ECDSA::sign(message, private_key).unwrap()
}
