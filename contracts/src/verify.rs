use bn254::format_pairing_check_uncompressed_values;
use near_sdk::near_bindgen;
use near_sys::alt_bn128_pairing_check;

use crate::{MainchainContract, MainchainContractExt};

#[near_bindgen]
impl MainchainContract {
    pub fn bn254_verify(&mut self, message: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> bool {
        let vals = format_pairing_check_uncompressed_values(message, signature, public_key).unwrap();

        let res;
        unsafe {
            res = alt_bn128_pairing_check(core::mem::size_of_val(&vals) as u64, vals.as_ptr() as *const u64 as u64);
        }

        res == 1
    }
}
