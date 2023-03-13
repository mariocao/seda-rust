pub mod batch;
pub mod committee;
pub mod consts;
pub mod dao;
pub mod data_request;
pub mod epoch;
pub mod fungible_token;
pub mod merkle;
pub mod node_registry;
pub mod slot;
pub mod storage;
pub mod verify;
use merkle::CryptoHash;
use near_contract_standards::fungible_token::{metadata::FungibleTokenMetadata, FungibleToken};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap, Vector},
    env,
    json_types::U128,
    near_bindgen,
    AccountId,
    Balance,
    BorshStorageKey,
    PanicOnDefault,
};

use crate::{
    batch::{Batch, BatchHeight, BatchId},
    epoch::EpochHeight,
    node_registry::Node,
};

/// Collection keys
#[derive(BorshStorageKey, BorshSerialize)]
enum MainchainStorageKeys {
    FungibleToken,
    FungibleTokenMetadata,
    ActiveNodes,
    PendingNodes,
    InactiveNodes,
    DataRequestAccumulator,
    BatchIdsByHeight,
    BatchById,
    NodesByBn254PublicKey,
}

/// Contract global state
#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct MainchainContract {
    // Fungible token used for staking
    token:    FungibleToken,
    // Fungible token metadata
    metadata: LazyOption<FungibleTokenMetadata>,
    // DAO account with admin privileges
    dao:      AccountId,
    // Mainchain configuration, changeable by the DAO
    config:   dao::Config,

    // TODO: do all of these need to be UnorderedMaps?
    // Nodes that are eligible to participate in the current epoch
    active_nodes:              UnorderedMap<AccountId, Node>,
    // Nodes that are not eligible to participate in the current epoch
    inactive_nodes:            UnorderedMap<AccountId, Node>,
    // Sub-set of inactive nodes that are waiting to be activated
    pending_nodes:             UnorderedMap<AccountId, EpochHeight>,
    // Sub-set of active nodes that are part of the committee of the current epoch
    // committees[EPOCH_COMMITTEES_LOOKAHEAD + 1][SLOTS_PER_EPOCH]
    committees:                Vec<Vec<AccountId>>,
    data_request_accumulator:  Vector<String>,
    num_batches:               BatchHeight,
    batch_ids_by_height:       LookupMap<BatchHeight, BatchId>,
    batch_by_id:               LookupMap<BatchId, Batch>,
    last_total_balance:        Balance,
    nodes_by_bn254_public_key: LookupMap<Vec<u8>, AccountId>,
    random_seed:               CryptoHash,
    bootstrapping_phase:       bool,
    last_processed_epoch:      EpochHeight,
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    #[init]
    pub fn new(dao: AccountId, initial_supply: U128, metadata: FungibleTokenMetadata, committee_size: u64) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(
            env::is_valid_account_id(dao.as_bytes()),
            "The DAO account ID is invalid"
        );
        let config = dao::Config {
            committee_size,
            ..Default::default()
        };
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(MainchainStorageKeys::FungibleToken),
            metadata: LazyOption::new(MainchainStorageKeys::FungibleTokenMetadata, Some(&metadata)),
            dao: dao.clone(),
            config,
            active_nodes: UnorderedMap::new(MainchainStorageKeys::ActiveNodes),
            inactive_nodes: UnorderedMap::new(MainchainStorageKeys::InactiveNodes),
            pending_nodes: UnorderedMap::new(MainchainStorageKeys::PendingNodes),
            committees: Vec::new(),
            data_request_accumulator: Vector::<String>::new(MainchainStorageKeys::DataRequestAccumulator),
            num_batches: 0,
            batch_ids_by_height: LookupMap::new(MainchainStorageKeys::BatchIdsByHeight),
            batch_by_id: LookupMap::new(MainchainStorageKeys::BatchById),
            last_total_balance: 0,
            nodes_by_bn254_public_key: LookupMap::new(MainchainStorageKeys::NodesByBn254PublicKey),
            random_seed: CryptoHash::default(),
            bootstrapping_phase: true,
            last_processed_epoch: 0,
        };
        this.token.internal_register_account(&dao);
        this.token.internal_deposit(&dao, initial_supply.into());
        this
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
#[path = ""]
mod tests {
    mod batch_test;
    mod committee_selection_test;
    mod dao_test;
    mod data_request_test;
    mod fungible_token_test;
    mod node_registry_test;
    mod slot_test;
    mod test_utils;
    mod verify_test;
}
