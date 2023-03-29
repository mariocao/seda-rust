use super::Promise;
use crate::{P2PBroadcastAction, PromiseAction};

// TODO: data could be cleaned up to a generic that implements our ToBytes trait
pub fn p2p_broadcast_message(data: Vec<u8>) -> Promise {
    Promise::new(PromiseAction::P2PBroadcast(P2PBroadcastAction { data }))
}
