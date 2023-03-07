use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{U128, U64},
    log,
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
    Balance,
};

use crate::{manage_storage_deposit, MainchainContract, MainchainContractExt};

/// Node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct Node {
    pub multi_addr:          String,
    pub balance:             Balance,
    pub bn254_public_key:    Vec<u8>,
}

/// Human-readable node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct HumanReadableNode {
    pub account_id:          AccountId,
    pub multi_addr:          String,
    pub balance:             Balance,
    pub bn254_public_key:    Vec<u8>,
}

/// Update node commands
#[derive(Deserialize, Serialize)]
pub enum UpdateNode {
    SetSocketAddress(String),
}

/// Contract private methods
impl MainchainContract {
    pub fn internal_get_node(&self, account_id: &AccountId) -> Option<Node> {
        let active_node = self.active_nodes.get(&account_id);
        if active_node.is_some() {
            return Some(active_node.unwrap());
        }
        let inactive_node = self.inactive_nodes.get(&account_id);
        if inactive_node.is_some() {
            return Some(inactive_node.unwrap());
        }
        None
    }

    pub fn get_expect_node(&self, node_id: AccountId) -> Node {
        self.internal_get_node(&node_id).expect("Node does not exist")
    }

    pub fn handle_node_balance_update(&mut self, account_id: &AccountId, node: &Node) {
        // if minimum stake is reached, make sure node is active or set epoch when eligible for committee selection
        if node.balance >= self.config.minimum_stake {
            // minimum stake is reached, if not already an active node, set the epoch when eligible for committee selection
            if self.active_nodes.get(&account_id).is_some() {
                // node is already active
                self.active_nodes.insert(&account_id, &node);
            } else {
                // node is not active, set epoch when eligible for committee selection
                let epoch_when_eligible = self.get_current_epoch() + self.config.epoch_delay_for_election;
                self.inactive_nodes.insert(&account_id, &node);
                self.pending_nodes.insert(&account_id, &epoch_when_eligible);
            }
        } else {
            // minimum stake is not reached, check if node is active
            if self.active_nodes.get(&account_id).is_some() {
                // node is active, remove from active nodes and add to inactive nodes
                self.active_nodes.remove(&account_id);
                self.inactive_nodes.insert(&account_id, &node);
            } else {
                // node is not active, update inactive nodes
                self.inactive_nodes.insert(&account_id, &node);
            }
        }
    }

    pub fn internal_deposit(&mut self, amount: Balance) {
        manage_storage_deposit!(self, "require", {
            let account_id = env::signer_account_id();

            // subtract from user balance and add to contract balance
            let new_user_balance = self.token.accounts.get(&account_id).unwrap() - amount;
            self.token.accounts.insert(&account_id, &new_user_balance);
            let mut node = self.get_expect_node(account_id.clone());
            node.balance += amount;
            self.handle_node_balance_update(&account_id, &node);

            // update the total balance of the contract
            self.last_total_balance += amount;

            env::log_str(format!("@{} deposited {}. New balance is {}", account_id, amount, node.balance).as_str());
        });
    }

    pub fn internal_withdraw(&mut self, amount: Balance) {
        // TODO: epoch delay for withdrawal
        manage_storage_deposit!(self, "require", {
            assert!(amount > 0, "Withdrawal amount should be positive");
            let account_id = env::predecessor_account_id();
            let mut node = self.get_expect_node(account_id.clone());
            env::log_str(format!("{} balance is {}", account_id, node.balance).as_str());
            assert!(node.balance >= amount, "Not enough balance to withdraw");

            // subtract from contract balance and add to user balance
            let new_user_balance = self.token.accounts.get(&account_id).unwrap() + amount;
            self.token.accounts.insert(&account_id, &new_user_balance);
            node.balance -= amount;
            self.handle_node_balance_update(&account_id, &node);

            // update global balance
            self.last_total_balance -= amount;

            env::log_str(
                format!(
                    "@{} withdrawing {}. New balance is {}",
                    account_id, amount, node.balance
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

        // assert unique bn254_public_key
        assert!(
            !self.nodes_by_bn254_public_key.contains_key(&bn254_public_key.clone()),
            "bn254_public_key already exists"
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
            bn254_public_key: bn254_public_key.clone(),
        };

        manage_storage_deposit!(self, "require", {
            // insert in inactive nodes
            self.inactive_nodes.insert(&account_id, &node);

            // insert in nodes_by_bn254_public_key
            self.nodes_by_bn254_public_key.insert(&bn254_public_key, &account_id);
        });
    }

    /// Updates one of the node's fields
    pub fn update_node(&mut self, command: UpdateNode) {
        let account_id = env::signer_account_id();
        let mut node = self.get_expect_node(account_id.clone());

        match command {
            UpdateNode::SetSocketAddress(new_multi_addr) => {
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

    pub fn deposit(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_deposit(amount);
    }

    /// Withdraws the balance for given account.
    pub fn withdraw(&mut self, amount: U128) {
        let amount: Balance = amount.into();
        self.internal_withdraw(amount);
    }

    /// Withdraws the entire balance from the predecessor account.
    pub fn withdraw_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let account = self.get_expect_node(account_id.clone());
        self.internal_withdraw(account.balance);
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

    pub fn get_node(&self, node_id: AccountId) -> Option<HumanReadableNode> {
        let node = self.internal_get_node(&node_id);
        if let Some(node) = node {
            Some(HumanReadableNode {
                account_id:          node_id,
                multi_addr:          node.multi_addr,
                balance:             node.balance,
                bn254_public_key:    node.bn254_public_key,
            })
        } else {
            None
        }
    }

    pub fn get_nodes(&self, limit: U64, offset: U64) -> Vec<HumanReadableNode> {
        let mut nodes = Vec::new();
        let mut index = self.active_nodes.len() - u64::from(offset);
        let limit = u64::from(limit);
        while index > 0 && nodes.len() < limit.try_into().unwrap() {
            if let Some(node_id) = self.active_nodes.keys().nth(index as usize - 1) {
                let node = self.active_nodes.get(&node_id).unwrap();
                let human_readable_node = HumanReadableNode {
                    account_id:          node_id,
                    multi_addr:          node.multi_addr,
                    balance:             node.balance,
                    bn254_public_key:    node.bn254_public_key,
                };
                nodes.push(human_readable_node);
            }
            index -= 1;
        }
        nodes
    }
}
