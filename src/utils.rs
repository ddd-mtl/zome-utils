//! Other helpers

use hdk::prelude::*;
use crate::*;


///
pub fn create_entry_relaxed<T: EntryDefRegistration>(typed: T) -> ExternResult<HeaderHash>
   where
      hdk::prelude::Entry: TryFrom<T>,
      <hdk::prelude::Entry as TryFrom<T>>::Error: std::fmt::Debug,
{
   let create_input = CreateInput::new(
      T::entry_def_id(),
      Entry::try_from(typed).unwrap(),
      ChainTopOrdering::Relaxed,
   );
   return create_entry(create_input);
}


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


/// Return true if entryType is of a certain type from a zome in the DNA
pub fn is_type(type_candidat: EntryType, zome_name: &str, type_name: &str) -> ExternResult<bool> {
   //trace!("*** is_type() called: {:?} == {:?} ?", type_candidat, entry);
   let zome_name: ZomeName = zome_name.to_string().into();
   if let EntryType::App(app_entry_byte) = type_candidat {
      let zome_info = zome_info()?;
      if zome_info.name == zome_name {
         let index = zome_info.entry_defs
                              .entry_def_index_from_id(EntryDefId::App(type_name.to_string()))
                              .ok_or(WasmError::Guest(String::from("Entry type not found")))
            ?;
         return Ok(app_entry_byte.id() == index);
      } else {
         /// Get other Zome's entry defs
         let zome_names = dna_info()?.zome_names;
         if !zome_names.contains(&zome_name) {
            warn!("Requested zome not part of DNA");
            return Ok(false);
         }
         let res = call(
            CallTargetCell::Local,
            zome_name,
            "entry_defs".into(),
            None,
            (),
         )?;
         let entry_defs: EntryDefsCallbackResult = decode_response(res)?;
         let entry_defs: EntryDefs = match entry_defs { EntryDefsCallbackResult::Defs(defs) => defs };
         let index = entry_defs
            .entry_def_index_from_id(EntryDefId::App(type_name.to_string()))
            .ok_or(WasmError::Guest(String::from("Entry type not found")))
            ?;
         return Ok(app_entry_byte.id() == index);
      }
   }
   warn!("is_type() failed because candidat is not of app type");
   Ok(false)
}

