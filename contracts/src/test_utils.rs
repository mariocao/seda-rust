use bn254::{PrivateKey, PublicKey, Signature, ECDSA};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{json_types::U128, test_utils::VMContextBuilder, AccountId, Balance, VMContext};

use crate::{
    consts::{DATA_IMAGE_SVG_ICON, INITIAL_SUPPLY},
    MainchainContract,
};

const TEST_DEPOSIT_AMOUNT: Balance = 9_500_000_000_000_000_000_000; // enough deposit to cover storage for all functions that require it

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
        2,
    )
}

pub struct TestAccount {
    pub account_id:         AccountId,
    pub ed25519_public_key: near_sdk::PublicKey,
    pub bn254_private_key:  PrivateKey,
    pub bn254_public_key:   PublicKey,
}

pub fn bob() -> TestAccount {
    let ed25519_public_key: near_sdk::PublicKey =
        "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap();
    let bn254_private_key_bytes =
        hex::decode("4b6d5965383445467931396d444a566e75507970664d4a6d744e477375493367".to_string()).unwrap();
    let bn254_private_key = PrivateKey::try_from(bn254_private_key_bytes.as_ref()).unwrap();
    let bn254_public_key = PublicKey::from_private_key(&bn254_private_key);
    return TestAccount {
        account_id: "bob_near".to_string().try_into().unwrap(),
        ed25519_public_key,
        bn254_private_key,
        bn254_public_key,
    };
}

pub fn alice() -> TestAccount {
    let ed25519_public_key: near_sdk::PublicKey =
        "ed25519:27ESUPfsjQtXpdV7iw6tosP6McmBEC8jq63g6qkZXJVf".parse().unwrap();
    let bn254_private_key_bytes =
        hex::decode("586177483953546d69394e414b7939726a4c7a31746b6f7671776f6865534c50".to_string()).unwrap();
    let bn254_private_key = PrivateKey::try_from(bn254_private_key_bytes.as_ref()).unwrap();
    let bn254_public_key = PublicKey::from_private_key(&bn254_private_key);
    return TestAccount {
        account_id: "alice_near".to_string().try_into().unwrap(),
        ed25519_public_key,
        bn254_private_key,
        bn254_public_key,
    };
}

pub fn carol() -> TestAccount {
    let ed25519_public_key: near_sdk::PublicKey =
        "ed25519:9xaHYd9VF6Me3gHTGku477KKws34XYtFfFiVV4c7CwNT".parse().unwrap();
    let bn254_private_key_bytes =
        hex::decode("484d5a7842376278505532797347457551744f6d34377152476368746d476154".to_string()).unwrap();
    let bn254_private_key = PrivateKey::try_from(bn254_private_key_bytes.as_ref()).unwrap();
    let bn254_public_key = PublicKey::from_private_key(&bn254_private_key);
    return TestAccount {
        account_id: "carol_near".to_string().try_into().unwrap(),
        ed25519_public_key,
        bn254_private_key,
        bn254_public_key,
    };
}

pub fn dao() -> TestAccount {
    let ed25519_public_key: near_sdk::PublicKey =
        "ed25519:4msyQstQ3Z7Gq1qrwE78HPTRYdLFtCmJ9dydrrbUtrer".parse().unwrap();
    let bn254_private_key_bytes =
        hex::decode("532797347457551744f484d5a78423762785056d34377152476368746d476154".to_string()).unwrap();
    let bn254_private_key = PrivateKey::try_from(bn254_private_key_bytes.as_ref()).unwrap();
    let bn254_public_key = PublicKey::from_private_key(&bn254_private_key);
    return TestAccount {
        account_id: "dao_near".to_string().try_into().unwrap(),
        ed25519_public_key,
        bn254_private_key,
        bn254_public_key,
    };
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
        .attached_deposit(TEST_DEPOSIT_AMOUNT) // required for post_data_request()
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
        .attached_deposit(TEST_DEPOSIT_AMOUNT) // required for post_data_request()
        .block_index(block_index)
        .build()
}

pub fn bn254_sign(test_account: TestAccount, message: &[u8]) -> Signature {
    ECDSA::sign(message, &test_account.bn254_private_key).unwrap()
}
