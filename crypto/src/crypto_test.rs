use std::{fs, path::Path};

use bn254::ECDSA;
use ed25519_dalek::{Keypair, SecretKey, Signature, Signer};

use super::*;
use crate::{CryptoError, MasterKey};

const TEST_SK_PATH: &str = "./seda_test_sk";
fn generate_test_sk() {
    if !Path::new(&TEST_SK_PATH).exists() {
        let sk = SecretKey::generate(&mut ed_rand::rngs::OsRng);
        fs::write(TEST_SK_PATH, hex::encode(sk.to_bytes())).expect("Unable to write secret key");
    }
}

#[test]
fn generate_bn254_pair() {
    generate_test_sk();
    let bn_pair = KeyPair::derive_bn254(TEST_SK_PATH, 1).expect("Couldn't derive bn254 key pair");
    let msg = "awesome-seda";
    let signature = ECDSA::sign(msg, &bn_pair.private_key).expect("couldnt sign msg");
    assert!(ECDSA::verify(msg, &signature, &bn_pair.public_key).is_ok())
}

#[test]
fn generate_ed25519_pair() {
    generate_test_sk();
    let ed_pair = KeyPair::derive_ed25519(TEST_SK_PATH, 1).expect("Couldn't derive ed25519 key pair");
    let dalek_pair = Keypair::from_bytes(&[ed_pair.private_key.to_bytes(), ed_pair.public_key.to_bytes()].concat())
        .expect("Couldn't convert ed25519 keypair");
    let msg: &[u8] = b"awesome-seda";
    let signature: Signature = dalek_pair.sign(msg);
    assert!(dalek_pair.verify(msg, &signature).is_ok());
}

#[test]
fn master_key_from_hex_1() {
    let mk_random = MasterKey::random();
    let mk_string: String = mk_random.into();
    let mk_from_string = MasterKey::try_from(mk_string);
    assert!(mk_from_string.is_ok());
}

#[test]
fn master_key_from_hex_2() {
    let mk_string: String = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let master_key = MasterKey::try_from(mk_string);
    assert!(master_key.is_ok());
}

#[test]
fn master_key_from_hex_error_length() {
    let mk_string: String = "1234".to_string();
    let master_key = MasterKey::try_from(mk_string);
    assert!(matches!(master_key, Err(CryptoError::InvalidMasterKeyLength(_))));
}

#[test]
fn master_key_from_hex_error_invalid() {
    let mk_string: String = "potato".to_string();
    let master_key = MasterKey::try_from(mk_string);
    assert!(matches!(master_key, Err(CryptoError::FromHex(_))));
}
