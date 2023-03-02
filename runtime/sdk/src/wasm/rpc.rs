use super::Promise;
use crate::{promises::RpcAction, PromiseAction, Result, SDKError, ToUrl};

pub fn rpc_call<U: ToUrl, M: ToString>(url: U, method: M, args: Vec<String>) -> Result<Promise> {
    let url = url.to_url()?;

    if url.scheme() != "ws" {
        return Err(SDKError::InvalidUrlScheme(url.scheme().into()));
    }

    Ok(Promise::new(PromiseAction::Rpc(RpcAction {
        url,
        method: method.to_string(),
        args,
    })))
}
