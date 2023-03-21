use super::Promise;
use crate::{p2p::MessageKind, P2PBroadcastAction, PromiseAction};

// TODO: data could be cleaned up to a generic that implements our ToBytes trait
// :)
pub fn p2p_broadcast_message(data: Vec<u8>, kind: MessageKind) -> Promise {
    Promise::new(PromiseAction::P2PBroadcast(P2PBroadcastAction { data, kind }))
}
