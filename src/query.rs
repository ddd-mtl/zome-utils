//! All helper functions calling `query()`

use hdk::prelude::*;
use crate::*;
use crate as zome_utils;

/// Return vec of typed entries of given entry type found in local source chain
pub fn get_all_typed_local<R: TryFrom<Entry>>(entry_type: EntryType)
   -> ExternResult<Vec<(ActionHash, Create, R)>>
{
   /// Query type
   let query_args = ChainQueryFilter::default()
      .include_entries(true)
      .action_type(ActionType::Create)
      .entry_type(entry_type);
   let records = query(query_args)?;
   /// Get typed for all results
   let mut typeds = Vec::new();
   for record in records {
      let typed: R = get_typed_from_record(record.clone())?;
      let Action::Create(create) = record.action()
         else { panic!("Should be a create Action")};
      typeds.push((record.action_address().to_owned(), create.clone(), typed))
   }
   /// Done
   Ok(typeds)
}


/// Get Record at address using query()
pub fn get_local_from_eh(eh: EntryHash) -> ExternResult<Record> {
   let mut set = HashSet::with_capacity(1);
   set.insert(eh);
   let query_args = ChainQueryFilter::default()
      .include_entries(true)
      .entry_hashes(set);
   let vec = query(query_args)?;
   if vec.len() != 1 {
      return zome_error!("Record not found at given EntryHash");
   }
   Ok(vec[0].clone())
}


/// Get Record at address using query()
pub fn get_local_from_ah(ah: ActionHash) -> ExternResult<Record> {
   let query_args = ChainQueryFilter::default()
      .include_entries(true);
   let maybe_vec = query(query_args);
   if let Err(err) = maybe_vec {
      return zome_error!("{:?}",err);
   }
   let vec = maybe_vec.unwrap();
   for record in vec {
      if record.action_address() == &ah {
         return Ok(record.clone());
      }
   }
   return zome_error!("Record not found at given ActionHash");
}
