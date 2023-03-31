use std::fmt::Debug;

use crate::Result;

#[async_trait::async_trait]
pub trait ChainAdapterTrait: Debug + Send + Sync + 'static {
    /// The Client type for the adapter specific implementation.
    type Client: Send + Sync + 'static;
    /// The Config fields for the adapter specific implementation.
    type Config: Send + Sync;

    /// Returns an new instance of the client given the server address.
    fn new_client(config: &Self::Config) -> Result<Self::Client>;

    /// Returns a signed transaction given the necessary information.
    #[allow(clippy::too_many_arguments)]
    async fn construct_signed_tx(
        signer_account_id: Option<&str>,
        signer_keypair: &[u8],
        contract_id: &str,
        method_name: &str,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
        server_url: &str,
    ) -> Result<Vec<u8>>;

    async fn construct_transfer_tx(
        signer_account_id: Option<&str>,
        signer_keypair: &[u8],
        receiver_id: &str,
        amount: u128,
        server_url: &str,
    ) -> Result<Vec<u8>>;

    /// To send a transaction for the adapter specific implementation.
    async fn send_tx(client: Self::Client, signed_tx: &[u8]) -> Result<Vec<u8>>;
    /// To view for the adapter specific implementation.
    async fn view(client: Self::Client, contract_id: &str, method_name: &str, args: Vec<u8>) -> Result<Vec<u8>>;
}
