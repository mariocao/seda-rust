use super::*;

#[derive(Debug, Serialize)]
pub struct RegisterNodeArgs {
    pub multi_addr:       String,
    pub bn254_public_key: Vec<u8>,
    pub signature:        Vec<u8>,
}

impl ToString for RegisterNodeArgs {
    fn to_string(&self) -> String {
        let json = json!(self);
        json.to_string()
    }
}
