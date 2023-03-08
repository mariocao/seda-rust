use std::path::Path;

use bn254::ECDSA;
use ed25519_dalek::{Keypair as Ed25519DalekKeyPair, SecretKey, Signature, Signer};
use rand::rngs::OsRng;
use tokio::fs::write;

use crate::{derive_bn254_key_pair_from_fs, derive_ed25519_key_pair_from_fs, Bn254KeyPair, Ed25519KeyPair};

const TEST_SK_PATH: &str = "./seda_test_sk";
async fn generate_test_sk() {
    if !Path::new(&TEST_SK_PATH).exists() {
        let mut csprng = OsRng {};
        let sk = SecretKey::generate(&mut csprng);
        write(TEST_SK_PATH, hex::encode(&sk))
            .await
            .expect("Unable to write secret key");
    }
}

#[tokio::test]
async fn generate_bn254_pair() {
    generate_test_sk().await;
    let bn_pair: Bn254KeyPair = derive_bn254_key_pair_from_fs(TEST_SK_PATH, 1).expect("Couldn't derive bn254 key pair");
    let msg = "awesome-seda";
    let signature = ECDSA::sign(msg, &bn_pair.private_key).expect("couldnt sign msg");
    assert!(ECDSA::verify(msg, &signature, &bn_pair.public_key).is_ok())
}

#[tokio::test]
async fn generate_ed25519_pair() {
    generate_test_sk().await;
    let ed_pair: Ed25519KeyPair =
        derive_ed25519_key_pair_from_fs(TEST_SK_PATH, 1).expect("Couldn't derive ed25519 key pair");
    let dalek_pair =
        Ed25519DalekKeyPair::from_bytes(&[ed_pair.private_key.to_bytes(), ed_pair.public_key.to_bytes()].concat())
            .expect("Couldn't convert ed25519 keypair");
    let msg: &[u8] = b"awesome-seda";
    let signature: Signature = dalek_pair.sign(msg);
    assert!(dalek_pair.verify(msg, &signature).is_ok());
}
