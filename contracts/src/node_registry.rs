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

/// Deposit info for one account to a node
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct DepositInfo {
    pub node_ed25519_public_key: Vec<u8>,
    pub amount:                  Balance,
}

/// Node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct Node {
    pub multi_addr:         String,
    pub balance:            Balance,
    pub ed25519_public_key: Vec<u8>,
    pub bn254_public_key:   Vec<u8>,
}

/// Human-readable node information
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct HumanReadableNode {
    pub account_id:         AccountId,
    pub multi_addr:         String,
    pub balance:            Balance,
    pub ed25519_public_key: Vec<u8>,
    pub bn254_public_key:   Vec<u8>,
}

/// Update node commands
#[derive(Deserialize, Serialize)]
pub enum UpdateNode {
    SetSocketAddress(String),
}

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
        self.internal_get_node(&account_id).expect("Node does not exist")
    }

    pub fn get_expect_node_by_ed25519_public_key(&self, ed25519_public_key: Vec<u8>) -> Node {
        let account_id = self
            .nodes_by_ed25519_public_key
            .get(&ed25519_public_key)
            .expect("Node does not exist");
        self.internal_get_node(&account_id).expect("Node does not exist")
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
            let new_user_balance = self.token.accounts.get(&depositor_account_id).unwrap() - amount;
            self.token.accounts.insert(&depositor_account_id, &new_user_balance);
            let mut node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            node.balance += amount;
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            self.handle_node_balance_update(&node_account_id, &node);

            // update info for depositor
            let depositor = self.depositors.get(&depositor_account_id);
            if depositor.is_none() {
                let deposit_info = vec![DepositInfo {
                    node_ed25519_public_key: ed25519_public_key,
                    amount,
                }];
                self.depositors.insert(&depositor_account_id, &deposit_info);
            } else {
                // find the deposit info for this node or create a new one
                let mut deposit_info = depositor.unwrap();
                if let Some(deposit_info) = deposit_info
                    .iter_mut()
                    .find(|x| x.node_ed25519_public_key == ed25519_public_key)
                {
                    deposit_info.amount += amount;
                } else {
                    deposit_info.push(DepositInfo {
                        node_ed25519_public_key: ed25519_public_key,
                        amount,
                    });
                    self.depositors.insert(&depositor_account_id, &deposit_info);
                }
            }

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
        // TODO: epoch delay for withdrawal
        manage_storage_deposit!(self, "require", {
            assert!(amount > 0, "Withdrawal amount should be positive");
            let mut node = self.get_expect_node_by_ed25519_public_key(ed25519_public_key.clone());
            assert!(node.balance >= amount, "Not enough balance to withdraw");

            // find depositor info for this node
            let depositor_account_id = env::signer_account_id();
            let depositor = self.depositors.get(&depositor_account_id);
            assert!(depositor.is_some(), "No deposit info found for this account");
            let mut depositor_vec = depositor.clone().unwrap();
            let deposit_info = depositor_vec
                .iter_mut()
                .find(|x| x.node_ed25519_public_key == ed25519_public_key)
                .expect("No deposit info found for this node");
            assert!(deposit_info.amount >= amount, "Not enough balance to withdraw");

            // subtract from contract balance and add to user balance
            let new_user_balance = self.token.accounts.get(&depositor_account_id).unwrap() + amount;
            self.token.accounts.insert(&depositor_account_id, &new_user_balance);
            node.balance -= amount;
            let node_account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
            self.handle_node_balance_update(&node_account_id, &node);

            // TODO: this is a f**king mess, we need to use a map from NEAR collection types
            // remove old deposit info from depositor_vec
            let mut depositor_vec = depositor.unwrap();
            depositor_vec.retain(|x| x.node_ed25519_public_key != ed25519_public_key);
            // update deposit info
            deposit_info.amount -= amount;
            // add back to depositor_vec
            depositor_vec.push(deposit_info.clone());
            // update depositors
            self.depositors.insert(&depositor_account_id, &depositor_vec);

            // update global balance
            self.last_total_balance -= amount;

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

    /// Updates one of the node's fields
    #[payable]
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

    /// Deposits the given amount to the given account.
    #[payable]
    pub fn deposit(&mut self, amount: U128, ed25519_public_key: Vec<u8>) {
        let amount: Balance = amount.into();
        self.internal_deposit(amount, ed25519_public_key);
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

    pub fn get_node(&self, account_id: AccountId) -> Option<HumanReadableNode> {
        let node = self.internal_get_node(&account_id);
        if let Some(node) = node {
            Some(HumanReadableNode {
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

    pub fn get_nodes(&self, limit: U64, offset: U64) -> Vec<HumanReadableNode> {
        let mut nodes = Vec::new();
        let mut index = self.active_nodes.len() - u64::from(offset);
        let limit = u64::from(limit);
        while index > 0 && nodes.len() < limit.try_into().unwrap() {
            if let Some(node_id) = self.active_nodes.keys().nth(index as usize - 1) {
                let node = self.active_nodes.get(&node_id).unwrap();
                let human_readable_node = HumanReadableNode {
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

    pub fn get_deposits(&self, account_id: AccountId) -> Option<Vec<DepositInfo>> {
        self.depositors.get(&account_id)
    }

    pub fn get_node_by_ed25519_public_key(&self, ed25519_public_key: Vec<u8>) -> HumanReadableNode {
        let account_id = self.nodes_by_ed25519_public_key.get(&ed25519_public_key).unwrap();
        let node = self.internal_get_node(&account_id).unwrap();
        HumanReadableNode {
            account_id,
            multi_addr: node.multi_addr,
            balance: node.balance,
            ed25519_public_key: node.ed25519_public_key,
            bn254_public_key: node.bn254_public_key,
        }
    }

    pub fn get_node_by_bn254_public_key(&self, bn254_public_key: Vec<u8>) -> HumanReadableNode {
        let account_id = self.nodes_by_bn254_public_key.get(&bn254_public_key).unwrap();
        let node = self.internal_get_node(&account_id).unwrap();
        HumanReadableNode {
            account_id,
            multi_addr: node.multi_addr,
            balance: node.balance,
            ed25519_public_key: node.ed25519_public_key,
            bn254_public_key: node.bn254_public_key,
        }
    }
}
