//! Other helpers

use hdk::prelude::*;
use crate::*;
use crate as zome_utils;


/// Returns number of seconds since UNIX_EPOCH
pub fn now() -> u64 {
   let now = sys_time().expect("sys_time() should always work");
   now.as_seconds_and_nanos().0 as u64
}


///
pub fn get_zome_index(candidat: ZomeName) -> ExternResult<u8> {
   let mut i = 0;
   for zome_name in dna_info()?.zome_names {
      //debug!("get_variant_index() variant = {:?}", variant);
      if zome_name == candidat {
         return Ok(i);
      }
      i += 1;
   }
   return zome_error!("Unknown Zome");
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
   return zome_error!("Unknown variant");
}


///
pub fn get_variant<T: UnitEnum>(entry_index: EntryDefIndex) -> ExternResult<T::Unit> {
   let mut i = 0;
   for variant in T::unit_iter() {
      if i == entry_index.0 {
         return Ok(variant);
      }
      i += 1;
   }
   return zome_error!("Unknown EntryDefIndex: {}", entry_index.0);
}


//
// /// Return true if entryType is of a certain entry type from a zome in the DNA
// pub fn is_type(candidat: EntryType, zome_name: &str, type_name: &str) -> ExternResult<bool> {
//    //trace!("*** is_type() called: {:?} == {:?} ?", type_candidat, entry);
//    let EntryType::App(app_entry_def) = candidat else {
//       warn!("is_type() failed because candidat is not of app type");
//       return Ok(false);
//    };
//    let zome_name: ZomeName = zome_name.to_string().into();
//    let zome_info = zome_info()?;
//    /// Already in the right zome
//    if zome_info.name == zome_name {
//       let index = zome_info.entry_defs
//                            .entry_def_index_from_id(EntryDefId::App(type_name.to_string()));
//                            //.ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Entry type not found"))))
//          //?;
//       return Ok(app_entry_def.id() == index);
//    } else {
//       /// Get other Zome's entry defs
//       let zome_names = dna_info()?.zome_names;
//       if !zome_names.contains(&zome_name) {
//          warn!("Requested zome not part of DNA");
//          return Ok(false);
//       }
//       let res = call(
//          CallTargetCell::Local,
//          zome_name,
//          "entry_defs".into(),
//          None,
//          (),
//       )?;
//       let entry_defs: EntryDefsCallbackResult = decode_response(res)?;
//       let entry_defs: EntryDefs = match entry_defs { EntryDefsCallbackResult::Defs(defs) => defs };
//       let index = entry_defs
//          .entry_def_index_from_id(EntryDefId::App(type_name.to_string()))
//          .ok_or(wasm_error!(WasmErrorInner::Guest(String::from("Entry type not found"))))
//          ?;
//       return Ok(app_entry_def.id() == index);
//    }
// }

