pub fn get_oracle_contract_id() -> String {
    std::env::var("ORACLE_CONTRACT_ID").expect("Env 'ORACLE_CONTRACT_ID' does not exist")
}

pub fn get_local_bn254_public_key() -> String {
    std::env::var("BN254_PUBLIC_KEY").expect("Env 'BN254_PUBLIC_KEY' does not exist")
}

pub fn get_local_ed25519_public_key() -> String {
    std::env::var("ED25519_PUBLIC_KEY").expect("Env 'ED25519_PUBLIC_KEY' does not exist")
}
