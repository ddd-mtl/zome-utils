//! All helper functions calling `get()`

use hdk::prelude::*;
use std::convert::TryFrom;
use crate::*;

pub type TypedEntryAndHash<T> = (T, ActionHash, EntryHash);
pub type OptionTypedEntryAndHash<T> = Option<TypedEntryAndHash<T>>;


//// Get untyped entry from eh
pub fn get_entry_from_eh(eh: EntryHash) -> ExternResult<Entry> {
   match get(eh, GetOptions::content())? {
      None => error("get_entry_from_eh(): Entry not found"),
      Some(record) => match record.entry() {
         record::RecordEntry::Present(entry) =>  {
            Ok(entry.clone())
         }
         _ => error("No Entry at record"),
      }
   }
}

/// Get EntryType of an Entry
pub fn get_entry_type(entry: &Entry) -> ExternResult<EntryType> {
   let entry_type= match entry {
      Entry::CounterSign(_data, _bytes) => unreachable!("CounterSign"),
      Entry::Agent(_agent_hash) => EntryType::AgentPubKey,
      Entry::CapClaim(_claim) => EntryType::CapClaim,
      Entry::CapGrant(_grant) => EntryType::CapGrant,
      Entry::App(_entry_bytes) => {
         let eh = hash_entry(entry.clone())?;
         get_entry_type_from_eh(eh)?
      },
   };
   Ok(entry_type)
}

/// Get EntryType at address
pub fn get_entry_type_from_eh(eh: EntryHash) -> ExternResult<EntryType> {
   let maybe_record = get(eh, GetOptions::latest())?;
   if maybe_record.is_none() {
      return error("no record found for entry_hash");
   }
   let record = maybe_record.unwrap();
   let entry_type = record.action().entry_type().unwrap().clone();
   Ok(entry_type)
}

/// Get EntryHash from a ActionHash
pub fn get_eh(hh: ActionHash) -> ExternResult<EntryHash> {
   trace!("hh_to_eh() START - get...");
   let maybe_record = get(hh, GetOptions::content())?;
   if let None = maybe_record {
      warn!("hh_to_eh() END - Record not found");
      return error("hh_to_eh(): Record not found");
   }
   trace!("hh_to_eh() END - Record found");
   return el_to_eh(&maybe_record.unwrap());
}


/// Get EntryHash and typed Entry from a ActionHash
pub fn get_typed_from_hh<T: TryFrom<Entry>>(hash: ActionHash)
   -> ExternResult<(EntryHash, T)>
{
   match get(hash.clone(), GetOptions::content())? {
      Some(record) => {
         let eh = record.action().entry_hash().expect("Converting ActionHash which does not have an Entry");
         Ok((eh.clone(), get_typed_from_el(record)?))
      },
      None => error("get_typed_from_hh(): Entry not found"),
   }
}


/// Get EntryHash and typed Entry from an EntryHash
pub fn get_typed_from_eh<T: TryFrom<Entry>>(eh: EntryHash) -> ExternResult<T> {
   match get(eh, GetOptions::content())? {
      Some(record) => Ok(get_typed_from_el(record)?),
      None => error("get_typed_from_eh(): Entry not found"),
   }
}

/// Get typed Entry from Record
pub fn get_typed_from_el<T: TryFrom<Entry>>(record: Record) -> ExternResult<T> {
   match record.entry() {
      record::RecordEntry::Present(entry) =>  {
         let res = T::try_from(entry.clone());
         let err = error::<T>(&format!("get_typed_from_el() failed for: {:?}", entry)).err().unwrap();
         res.map_err(|_|err)
      },
      _ => error("Could not convert record"),
   }
}


/// Get typed Entry and its author from EntryHash
/// Must be a single author entry type
pub fn get_typed_and_author<T: TryFrom<Entry>>(ah: &AnyLinkableHash)
   -> ExternResult<(AgentPubKey, T)>
{
   let eh = ah.clone().into_entry_hash()
      .ok_or(wasm_error!(WasmErrorInner::Guest("Given address is not an entry hash".to_string())))?;

   let maybe_maybe_record = get(eh.clone(), GetOptions::latest());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting record: {}", err);
      return Err(err);
   }
   let maybe_record = maybe_maybe_record.unwrap();
   if maybe_record.is_none() {
      return error("no record found at address");
   }
   let record = maybe_record.unwrap();
   //assert!(entry_item.actions.len() > 0);
   //assert!(entry_item.actions[0].provenances().len() > 0);
   let author = record.action().author();
   let app_entry = get_typed_from_el::<T>(record.clone())?;
   Ok((author.clone(), app_entry))
}


///
pub fn get_latest_typed_from_eh<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
   entry_hash: EntryHash,
) -> ExternResult<OptionTypedEntryAndHash<T>> {
   /// First, make sure we DO have the latest action_hash address
   let maybe_latest_hh = match get_details(entry_hash.clone(), GetOptions::latest())? {
      Some(Details::Entry(details)) => match details.entry_dht_status {
         metadata::EntryDhtStatus::Live => match details.updates.len() {
            // pass out the action associated with this entry
            0 => Some(shh_to_hh(details.actions.first().unwrap().to_owned())),
            _ => {
               let mut sortlist = details.updates.to_vec();
               // unix timestamp should work for sorting
               sortlist.sort_by_key(|update| update.action().timestamp().as_micros());
               // sorts in ascending order, so take the last record
               let last = sortlist.last().unwrap().to_owned();
               Some(shh_to_hh(last))
            }
         },
         metadata::EntryDhtStatus::Dead => None,
         _ => None,
      },
      _ => None,
   };
   let latest_hh = return_none!(maybe_latest_hh);
   /// Second, go and get that record, and return its entry and action_address
   let maybe_latest_el = get(latest_hh, GetOptions::latest())?;
   let el = return_none!(maybe_latest_el);
   let maybe_typed_entry = el.entry().to_app_option::<T>()?;
   let typed_entry = return_none!(maybe_typed_entry);
   let hh = match el.action() {
      /// we DO want to return the action for the original instead of the updated
      Action::Update(update) => update.original_action_address.clone(),
      Action::Create(_) => el.action_address().clone(),
      _ => unreachable!("Can't have returned a action for a nonexistent entry"),
   };
   let eh =  el.action().entry_hash().unwrap().to_owned();
   /// Done
   Ok(Some((typed_entry, hh, eh)))
}