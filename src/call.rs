//! All helper functions calling `call()` or `call_remote()`

use hdk::prelude::*;
use crate as zome_utils;
use crate::zome_error;


/// Remote call to self
pub fn call_self<I>(fn_name: &str, payload: I) -> ExternResult<ZomeCallResponse>
   where
      I: serde::Serialize + std::fmt::Debug
{
   // TODO check fn_name exists?
   call_remote(
      agent_info()?.agent_latest_pubkey,
      zome_info()?.name,
      fn_name.to_string().into(),
      None,
      payload,
   )
}


///
pub fn call_self_cell<I, O>(zome_name: &str, fn_name: &str, payload: I) -> ExternResult<O>
   where
      I: serde::Serialize + std::fmt::Debug,
      O: serde::de::DeserializeOwned + std::fmt::Debug
{
   //debug!("call_self_cell() - {}()", fn_name);
   // TODO check fn_name exists?
   let res = call(
      CallTargetCell::Local,
      zome_name,
      fn_name.to_string().into(),
      None,
      payload,
   )?;
   //debug!("call_self_cell() response for {fn_name}(): {:?}", res);
   let output: ExternResult<O> = zome_utils::decode_response(res);
   //debug!("call_self_cell() decoded response for {fn_name}(): {:?}", output);
   if let Err(e) = output {
      return zome_error!("Decoding zome call response from {fn_name}() failed: {}", e);
   }
   output
}