use super::errors::{DelegateCliError, Result};

/// Taken from https://docs.rs/near-sdk-sim/latest/near_sdk_sim/
/// Converts a string (nominated in full NEAR tokens) and converts it to
/// yoctoNEAR (smallest denominator in NEAR)
pub fn to_yocto(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

/// Retrieve Ed25519 keypair bytes from a string in the format of
/// `ed25519:base58-encoded-string`
pub(crate) fn get_signer_keypair_from_config(account_secret_key: &str) -> Result<Vec<u8>> {
    let signer_keypair_str = account_secret_key
        .strip_prefix("ed25519:")
        .ok_or(DelegateCliError::Config(
            "`account_secret_key` should contain prefix `ed25519:`".to_string(),
        ))?;

    Ok(bs58::decode(signer_keypair_str)
        .into_vec()
        .expect("Base58-encoded ed25519 keypair should be valid"))
}
