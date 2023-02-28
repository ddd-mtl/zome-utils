//! Other helpers

use hdk::prelude::*;
use crate::*;
use crate as zome_utils;

/// Return ActionHash from SignedActionHashed
pub fn sah_to_ah(sah: SignedActionHashed) -> ActionHash {
   sah.as_hash().to_owned()
}


/// Return EntryHash for Record
pub fn record_to_eh(record: &Record) -> ExternResult<EntryHash> {
   let maybe_eh = record.action().entry_hash();
   if maybe_eh.is_none() {
      warn!("record_to_eh(): entry_hash not found");
      return zome_error!("record_to_eh(): entry_hash not found");
   }
   Ok(maybe_eh.unwrap().clone())
}


/// Returns number of seconds since UNIX_EPOCH
pub fn now() -> u64 {
   let now = sys_time().expect("sys_time() should always work");
   now.as_seconds_and_nanos().0 as u64
}


/// Get EntryDefIndex from a unit_enum
pub fn get_variant_index<T: UnitEnum>(unknown: T::Unit) -> ExternResult<u8> {
   let mut i = 0;
   for variant in T::unit_iter() {
      //debug!("get_variant_index() variant = {:?}", variant);
      if variant == unknown {
         return Ok(i);
      }
      i += 1;
   }
   return Err(wasm_error!(WasmErrorInner::Guest("Unknown variant".to_string())));
}



// pub fn app_type_to_location(app_type: AppEntryType) -> AppEntryLocation {
//
// }


//
// /// Return true if entryType is of a certain type from a zome in the DNA
// pub fn is_type(type_candidat: EntryType, zome_name: &str, type_name: &str) -> ExternResult<bool> {
//    //trace!("*** is_type() called: {:?} == {:?} ?", type_candidat, entry);
//    let zome_name: ZomeName = zome_name.to_string().into();
//    if let EntryType::App(app_entry_byte) = type_candidat {
//       let zome_info = zome_info()?;
//       if zome_info.name == zome_name {
//          let index = zome_info.entry_defs
//                               .entry_def_index_from_id(EntryDefId::App(type_name.to_string()))
//                               .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Entry type not found"))))
//             ?;
//          return Ok(app_entry_byte.id() == index);
//       } else {
//          /// Get other Zome's entry defs
//          let zome_names = dna_info()?.zome_names;
//          if !zome_names.contains(&zome_name) {
//             warn!("Requested zome not part of DNA");
//             return Ok(false);
//          }
//          let res = call(
//             CallTargetCell::Local,
//             zome_name,
//             "entry_defs".into(),
//             None,
//             (),
//          )?;
//          let entry_defs: EntryDefsCallbackResult = decode_response(res)?;
//          let entry_defs: EntryDefs = match entry_defs { EntryDefsCallbackResult::Defs(defs) => defs };
//          let index = entry_defs
//             .entry_def_index_from_id(EntryDefId::App(type_name.to_string()))
//             .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Entry type not found"))))
//             ?;
//          return Ok(app_entry_byte.id() == index);
//       }
//    }
//    warn!("is_type() failed because candidat is not of app type");
//    Ok(false)
// }

