use near_sdk::{
    collections::{LookupMap, UnorderedMap},
    env,
    json_types::{U128, U64},
    log,
    near_bindgen,
    AccountId,
    Balance,
};
use seda_common::{DepositInfo, Node, NodeInfo, UpdateNode};

use crate::{manage_storage_deposit, MainchainContract, MainchainContractExt, MainchainStorageKeys};

/// Contract private methods
impl MainchainContract {
    pub fn internal_get_node(&self, account_id: &AccountId) -> Option<Node> {
        let active_node = self.active_nodes.get(account_id);
        if let Some(node) = active_node {
            return Some(node);
        }
        let inactive_node = self.inactive_nodes.get(account_id);
        if let Some(node) = inactive_node {
            return Some(node);
        }
        None
    }

    pub fn get_expect_node(&self, account_id: AccountId) -> Node {
        self.internal_get_node(&account_id)
            .unwrap_or_else(|| panic!("{}", format!("Node {account_id} does not exist")))
    }

    pub fn get_expect_node_by_ed25519_public_key(&self, ed25519_public_key: Vec<u8>) -> Node {
        let account_id = self
            .nodes_by_ed25519_public_key
            .get(&ed25519_public_key)
            .unwrap_or_else(|| panic!("Node {:?} does not exist", ed25519_public_key));
        self.internal_get_node(&account_id)
            .unwrap_or_else(|| panic!("{}", format!("Node {account_id} does not exist")))
    }

    pub fn handle_node_balance_update(&mut self, account_id: &AccountId, node: &Node) {
        // if minimum stake is reached, make sure node is active or set epoch when
        // eligible for committee selection
        if node.balance >= self.config.minimum_stake {
            // minimum stake is reached, if not already an active node, set the epoch when
            // eligible for committee selection
            if self.active_nodes.get(account_id).is_some() {
                // node is already active
                self.active_nodes.insert(account_id, node);
            } else {
                // node is not active, set epoch when eligible for committee selection
                let epoch_when_eligible = self.get_current_epoch() + self.config.epoch_delay_for_election;
                self.inactive_nodes.insert(account_id, node);
                self.pending_nodes.insert(account_id, &epoch_when_eligible);
            }
        } else {
            // minimum stake is not reached, check if node is active
            if self.active_nodes.get(account_id).is_some() {
                // node is active, remove from active nodes and add to inactive nodes
                self.active_nodes.remove(account_id);
                self.inactive_nodes.insert(account_id, node);
            } else {
                // node is not active, update inactive nodes
                self.inactive_nodes.insert(account_id, node);
            }
        }
    }

    pub fn internal_deposit(&mut self, amount: Balance, ed25519_public_key: Vec<u8>) {
        manage_storage_deposit!(self, "require", {
            let depositor_account_id = env::signer_account_id();

            // subtract from user balance and add to contract balance
            self.burn(&depositor_account_id, amount);
            let mut node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            node.balance += amount;
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            self.handle_node_balance_update(&node_account_id, &node);

            // update info for depositor
            let mut depositor = self.depositors.get(&depositor_account_id).unwrap_or_else(|| {
                UnorderedMap::new(MainchainStorageKeys::Depositor {
                    account_hash: env::sha256_array(depositor_account_id.as_bytes()),
                })
            });
            depositor.insert(&ed25519_public_key, &amount);
            self.depositors.insert(&depositor_account_id, &depositor);

            // update the total balance of the contract
            self.last_total_balance += amount;

            env::log_str(
                format!(
                    "@{} deposited {} into {}'s node. New balance is {}",
                    depositor_account_id, amount, node_account_id, node.balance
                )
                .as_str(),
            );
        });
    }

    pub fn internal_withdraw(&mut self, amount: Balance, ed25519_public_key: Vec<u8>) {
        manage_storage_deposit!(self, {
            assert!(amount > 0, "Withdrawal amount should be positive");
            let mut node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            assert!(node.balance >= amount, "Not enough balance to withdraw");

            // find depositor info for this node
            let depositor_account_id = env::signer_account_id();
            let mut depositor = self
                .depositors
                .get(&depositor_account_id)
                .expect("No deposit info found for this account");
            let deposited = depositor
                .get(&ed25519_public_key)
                .expect("No deposit info found for this node");
            assert!(deposited >= amount, "Not enough balance to withdraw");

            // find withdraw request
            let mut node_withdraw_requests = self.withdraw_requests.get(&ed25519_public_key).unwrap_or_else(|| {
                LookupMap::new(MainchainStorageKeys::WithdrawRequest {
                    account_hash: env::sha256_array(ed25519_public_key.as_slice()),
                })
            });
            // assert there is a pending withdrawal for this depositor
            let withdraw_request = node_withdraw_requests
                .get(&depositor_account_id)
                .expect("No pending withdrawal request found for this account");
            // check that the epoch is valid
            let current_epoch = self.get_current_epoch();
            assert!(
                withdraw_request.epoch <= current_epoch,
                "{} epochs remain until withdrawal is allowed",
                withdraw_request.epoch - current_epoch
            );

            // subtract from contract balance and add to user balance
            self.mint(&depositor_account_id, amount);
            node.balance -= amount;
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            self.handle_node_balance_update(&node_account_id, &node);
            depositor.insert(&ed25519_public_key, &(deposited - amount));
            self.depositors.insert(&depositor_account_id, &depositor);

            // update global balance
            self.last_total_balance -= amount;

            // remove the withdraw request
            node_withdraw_requests.remove(&depositor_account_id);
            self.withdraw_requests
                .insert(&ed25519_public_key, &node_withdraw_requests);

            env::log_str(
                format!(
                    "@{} withdrawing {} from {}'s node. New balance is {}",
                    depositor_account_id, node_account_id, amount, node.balance
                )
                .as_str(),
            );
        });
    }
}

/// Contract public methods
#[near_bindgen]
impl MainchainContract {
    /// Registers a new node while charging for storage usage
    #[payable]
    pub fn register_node(&mut self, multi_addr: String, bn254_public_key: Vec<u8>, signature: Vec<u8>) {
        let account_id = env::signer_account_id();
        let ed25519_public_key = env::signer_account_pk().into_bytes().to_vec();

        // assert unique bn254_public_key and ed25519_public_key
        assert!(
            !self.nodes_by_bn254_public_key.contains_key(&bn254_public_key.clone()),
            "bn254_public_key already exists"
        );
        assert!(
            !self.nodes_by_ed25519_public_key.contains_key(&ed25519_public_key),
            "ed25519_public_key already exists"
        );

        // verify the signature
        assert!(
            self.bn254_verify(account_id.as_bytes().to_vec(), signature, bn254_public_key.clone()),
            "Invalid signature"
        );

        // create a new node
        log!("{} registered node", account_id);
        let node = Node {
            multi_addr,
            balance: 0,
            ed25519_public_key,
            bn254_public_key: bn254_public_key.clone(),
        };

        manage_storage_deposit!(self, "require", {
            // insert in inactive nodes
            self.inactive_nodes.insert(&account_id, &node);

            // insert in nodes_by_bn254_public_key and nodes_by_ed25519_public_key
            self.nodes_by_bn254_public_key.insert(&bn254_public_key, &account_id);
            self.nodes_by_ed25519_public_key
                .insert(&node.ed25519_public_key, &account_id);
        });
    }

    pub fn unregister_node(&mut self, ed25519_public_key: Vec<u8>) {
        manage_storage_deposit!(self, "refund", {
            // assert the signer_account_pk matches the ed25519_public_key
            assert!(
                env::signer_account_pk().into_bytes().to_vec() == ed25519_public_key,
                "Invalid ed25519_public_key"
            );

            // assert the node balance is zero
            let node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            assert!(node.balance == 0, "Node balance is not zero");

            // remove the node
            let account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            self.inactive_nodes.remove(&account_id);
        });
    }

    /// Updates one of the node's fields
    #[payable]
    pub fn update_node(&mut self, command: UpdateNode) {
        let account_id = env::signer_account_id();
        let mut node = self.get_expect_node(account_id.clone());

        match command {
            UpdateNode::SetSocketAddress { new_multi_addr } => {
                log!("{} updated node multi_addr to {}", account_id, new_multi_addr);
                node.multi_addr = new_multi_addr;
            }
        }

        manage_storage_deposit!(self, {
            if self.active_nodes.get(&account_id).is_some() {
                self.active_nodes.insert(&account_id, &node);
            } else {
                self.inactive_nodes.insert(&account_id, &node);
            }
        });
    }

    /// Deposits the given amount to the given account.
    #[payable]
    pub fn deposit(&mut self, amount: U128, ed25519_public_key: Vec<u8>) {
        let amount: Balance = amount.into();
        self.internal_deposit(amount, ed25519_public_key);
    }

    #[payable]
    pub fn request_withdraw(&mut self, amount: U128, ed25519_public_key: Vec<u8>) {
        manage_storage_deposit!(self, "require", {
            let amount: Balance = amount.into();
            assert!(amount > 0, "Withdrawal amount should be positive");
            let node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            assert!(node.balance >= amount, "Not enough balance to withdraw");

            // find depositor info for this node
            let depositor_account_id = env::signer_account_id();
            let depositor = self
                .depositors
                .get(&depositor_account_id)
                .expect("No deposit info found for this account");
            let deposited = depositor
                .get(&ed25519_public_key)
                .expect("No deposit info found for this node");
            assert!(deposited >= amount, "Not enough balance to withdraw");

            // create a new pending withdrawal
            let mut node_withdraw_requests = self.withdraw_requests.get(&ed25519_public_key).unwrap_or_else(|| {
                LookupMap::new(MainchainStorageKeys::WithdrawRequest {
                    account_hash: env::sha256_array(ed25519_public_key.as_slice()),
                })
            });
            // assert there is no pending withdrawal for this depositor
            assert!(
                node_withdraw_requests.get(&depositor_account_id).is_none(),
                "There is already a pending withdrawal for this account"
            );
            let pending_withdraw = WithdrawRequest {
                amount,
                epoch: self.get_current_epoch() + self.config.withdraw_delay,
            };
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            log!(
                "{} requested withdrawal of {} from {}'s node. Will be available at epoch {}",
                depositor_account_id,
                amount,
                node_account_id,
                self.config.withdraw_delay + self.get_current_epoch()
            );
            node_withdraw_requests.insert(&depositor_account_id, &pending_withdraw);
            self.withdraw_requests
                .insert(&ed25519_public_key, &node_withdraw_requests);
        });
    }

    #[payable]
    pub fn cancel_withdraw_request(&mut self, ed25519_public_key: Vec<u8>) {
        manage_storage_deposit!(self, {
            let depositor_account_id = env::signer_account_id();
            let mut node_withdraw_requests = self.withdraw_requests.get(&ed25519_public_key).unwrap_or_else(|| {
                LookupMap::new(MainchainStorageKeys::WithdrawRequest {
                    account_hash: env::sha256_array(ed25519_public_key.as_slice()),
                })
            });
            node_withdraw_requests.remove(&depositor_account_id);
            self.withdraw_requests
                .insert(&ed25519_public_key, &node_withdraw_requests);
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            log!(
                "{} cancelled withdrawal request for {}",
                depositor_account_id,
                node_account_id
            );
        });
    }

    /// Withdraws the balance for given account.
    #[payable]
    pub fn withdraw(&mut self, amount: U128, ed25519_public_key: Vec<u8>) {
        let amount: Balance = amount.into();
        self.internal_withdraw(amount, ed25519_public_key);
    }

    /*************** */
    /* View methods */
    /*************** */

    pub fn is_node_active(&self, account_id: AccountId) -> bool {
        self.active_nodes.get(&account_id).is_some()
    }

    /// Returns the balance of the given account.
    pub fn get_node_balance(&self, account_id: AccountId) -> U128 {
        U128(self.internal_get_node(&account_id).unwrap().balance)
    }

    pub fn get_node(&self, account_id: AccountId) -> Option<NodeInfo> {
        let node = self.internal_get_node(&account_id);
        if let Some(node) = node {
            Some(NodeInfo {
                account_id,
                multi_addr: node.multi_addr,
                balance: node.balance,
                ed25519_public_key: node.ed25519_public_key,
                bn254_public_key: node.bn254_public_key,
            })
        } else {
            None
        }
    }

    pub fn get_nodes(&self, limit: U64, offset: U64) -> Vec<NodeInfo> {
        let mut nodes = Vec::new();
        let mut index = self.active_nodes.len() - u64::from(offset);
        let limit = u64::from(limit);
        while index > 0 && nodes.len() < limit.try_into().unwrap() {
            if let Some(node_id) = self.active_nodes.keys().nth(index as usize - 1) {
                let node = self.active_nodes.get(&node_id).unwrap();
                let human_readable_node = NodeInfo {
                    account_id:         node_id,
                    multi_addr:         node.multi_addr,
                    balance:            node.balance,
                    ed25519_public_key: node.ed25519_public_key,
                    bn254_public_key:   node.bn254_public_key,
                };
                nodes.push(human_readable_node);
            }
            index -= 1;
        }
        nodes
    }

    pub fn get_deposits(&self, account_id: AccountId) -> Vec<DepositInfo> {
        let depositor = self.depositors.get(&account_id).unwrap();
        let mut deposits = Vec::new();
        for deposit in depositor.iter() {
            deposits.push(DepositInfo {
                amount:                  deposit.1,
                node_ed25519_public_key: deposit.0,
            });
        }
        deposits
    }

    pub fn get_node_by_ed25519_public_key(&self, ed25519_public_key: Vec<u8>) -> NodeInfo {
        let account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
        let node = self.internal_get_node(&account_id).unwrap();
        NodeInfo {
            account_id,
            multi_addr: node.multi_addr,
            balance: node.balance,
            ed25519_public_key: node.ed25519_public_key,
            bn254_public_key: node.bn254_public_key,
        }
    }

    pub fn get_node_by_bn254_public_key(&self, bn254_public_key: Vec<u8>) -> NodeInfo {
        let account_id = self.nodes_by_bn254_public_key.get(&bn254_public_key).unwrap();
        let node = self.internal_get_node(&account_id).unwrap();
        NodeInfo {
            account_id,
            multi_addr: node.multi_addr,
            balance: node.balance,
            ed25519_public_key: node.ed25519_public_key,
            bn254_public_key: node.bn254_public_key,
        }
    }
}
