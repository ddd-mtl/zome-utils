//! Other helpers

use hdk::prelude::*;
use crate::*;


/// Return HeaderHash from SignedHeaderHashed
pub fn shh_to_hh(shh: element::SignedHeaderHashed) -> HeaderHash {
   shh.header_hashed().as_hash().to_owned()
}


/// Return EntryHash for Element
pub fn el_to_eh(element: &Element) -> ExternResult<EntryHash> {
   let maybe_eh = element.header().entry_hash();
   if let None = maybe_eh {
      warn!("el_to_eh(): entry_hash not found");
      return error("el_to_eh(): entry_hash not found");
   }
   Ok(maybe_eh.unwrap().clone())
}


/// Returns number of seconds since UNIX_EPOCH
pub fn now() -> u64 {
   let now = sys_time().expect("sys_time() should always work");
   now.as_seconds_and_nanos().0 as u64
}


/// Remote call to self
pub fn call_self<I>(fn_name: &str, payload: I) -> ExternResult<ZomeCallResponse>
   where
      I: serde::Serialize + std::fmt::Debug
{
   call_remote(
      agent_info()?.agent_latest_pubkey,
      zome_info()?.name,
      fn_name.to_string().into(),
      None,
      payload,
   )
}

