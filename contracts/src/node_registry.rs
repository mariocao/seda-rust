use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{U128, U64},
    log,
    near_bindgen,
    serde::{Deserialize, Serialize},
    Balance, PublicKey,
};

use crate::{manage_storage_deposit, MainchainContract, MainchainContractExt};

/// Node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct Node {
    pub multi_addr:       String,
    pub balance:          Balance,
    pub bn254_public_key: Vec<u8>,
}

/// Human-readable node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct HumanReadableNode {
    pub public_key:       PublicKey,
    pub multi_addr:       String,
    pub balance:          Balance,
    pub bn254_public_key: Vec<u8>,
}

/// Update node commands
#[derive(Deserialize, Serialize)]
pub enum UpdateNode {
    SetSocketAddress(String),
}

/// Contract private methods
impl MainchainContract {
    pub fn internal_get_node(&self, public_key: &PublicKey) -> Option<Node> {
        let active_node = self.active_nodes.get(public_key);
        if let Some(node) = active_node {
            return Some(node);
        }
        let inactive_node = self.inactive_nodes.get(public_key);
        if let Some(node) = inactive_node {
            return Some(node);
        }
        None
    }

    pub fn get_expect_node(&self, public_key: PublicKey) -> Node {
        self.internal_get_node(&public_key).expect("Node does not exist")
    }

    pub fn handle_node_balance_update(&mut self, public_key: &PublicKey, node: &Node) {
        // if minimum stake is reached, make sure node is active or set epoch when
        // eligible for committee selection
        if node.balance >= self.config.minimum_stake {
            // minimum stake is reached, if not already an active node, set the epoch when
            // eligible for committee selection
            if self.active_nodes.get(public_key).is_some() {
                // node is already active
                self.active_nodes.insert(public_key, node);
            } else {
                // node is not active, set epoch when eligible for committee selection
                let epoch_when_eligible = self.get_current_epoch() + self.config.epoch_delay_for_election;
                self.inactive_nodes.insert(public_key, node);
                self.pending_nodes.insert(public_key, &epoch_when_eligible);
            }
        } else {
            // minimum stake is not reached, check if node is active
            if self.active_nodes.get(public_key).is_some() {
                // node is active, remove from active nodes and add to inactive nodes
                self.active_nodes.remove(public_key);
                self.inactive_nodes.insert(public_key, node);
            } else {
                // node is not active, update inactive nodes
                self.inactive_nodes.insert(public_key, node);
            }
        }
    }

    pub fn internal_deposit(&mut self, amount: Balance) {
        manage_storage_deposit!(self, "require", {
            let account_id = env::signer_account_id();
            let public_key = env::signer_account_pk();

            // subtract from user balance and add to contract balance
            let new_user_balance = self.token.accounts.get(&account_id).unwrap() - amount;
            self.token.accounts.insert(&account_id, &new_user_balance);
            let mut node = self.get_expect_node(public_key.clone());
            node.balance += amount;
            self.handle_node_balance_update(&public_key, &node);

            // update the total balance of the contract
            self.last_total_balance += amount;

            env::log_str(format!("@{} deposited {}. New balance is {}", account_id, amount, node.balance).as_str());
        });
    }

    pub fn internal_withdraw(&mut self, amount: Balance) {
        // TODO: epoch delay for withdrawal
        manage_storage_deposit!(self, "require", {
            assert!(amount > 0, "Withdrawal amount should be positive");
            let public_key = env::signer_account_pk();
            let mut node = self.get_expect_node(public_key.clone());
            env::log_str(format!("{:?} balance is {}", public_key, node.balance).as_str());
            assert!(node.balance >= amount, "Not enough balance to withdraw");

            // subtract from contract balance and add to user balance
            let account_id = env::signer_account_id();
            let new_user_balance = self.token.accounts.get(&account_id).unwrap() + amount;
            self.token.accounts.insert(&account_id, &new_user_balance);
            node.balance -= amount;
            self.handle_node_balance_update(&public_key, &node);

            // update global balance
            self.last_total_balance -= amount;

            env::log_str(
                format!(
                    "@{:?} withdrawing {}. New balance is {}",
                    public_key, amount, node.balance
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
        let public_key = env::signer_account_pk();

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
        log!("{:?} registered node", public_key);
        let node = Node {
            multi_addr,
            balance: 0,
            bn254_public_key: bn254_public_key.clone(),
        };

        manage_storage_deposit!(self, "require", {
            // insert in inactive nodes
            self.inactive_nodes.insert(&public_key, &node);

            // insert in nodes_by_bn254_public_key
            self.nodes_by_bn254_public_key.insert(&bn254_public_key, &public_key);
        });
    }

    /// Updates one of the node's fields
    pub fn update_node(&mut self, command: UpdateNode) {
        let public_key = env::signer_account_pk();
        let mut node = self.get_expect_node(public_key.clone());

        match command {
            UpdateNode::SetSocketAddress(new_multi_addr) => {
                log!("{:?} updated node multi_addr to {}", public_key, new_multi_addr);
                node.multi_addr = new_multi_addr;
            }
        }

        manage_storage_deposit!(self, {
            if self.active_nodes.get(&public_key).is_some() {
                self.active_nodes.insert(&public_key, &node);
            } else {
                self.inactive_nodes.insert(&public_key, &node);
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
        let public_key = env::signer_account_pk();
        let account = self.get_expect_node(public_key);
        self.internal_withdraw(account.balance);
    }

    /*************** */
    /* View methods */
    /*************** */

    pub fn is_node_active(&self, public_key: PublicKey) -> bool {
        self.active_nodes.get(&public_key).is_some()
    }

    /// Returns the balance of the given account.
    pub fn get_node_balance(&self, public_key: PublicKey) -> U128 {
        U128(self.internal_get_node(&public_key).unwrap().balance)
    }

    pub fn get_node(&self, public_key: PublicKey) -> Option<HumanReadableNode> {
        let node = self.internal_get_node(&public_key);
        if let Some(node) = node {
            Some(HumanReadableNode {
                public_key:       public_key,
                multi_addr:       node.multi_addr,
                balance:          node.balance,
                bn254_public_key: node.bn254_public_key,
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
                    public_key:       node_id,
                    multi_addr:       node.multi_addr,
                    balance:          node.balance,
                    bn254_public_key: node.bn254_public_key,
                };
                nodes.push(human_readable_node);
            }
            index -= 1;
        }
        nodes
    }
}
