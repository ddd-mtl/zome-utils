//! All helper functions calling `get()`

use hdk::prelude::*;
use std::convert::TryFrom;
use crate::*;

pub type TypedEntryAndHash<T> = (T, ActionHash, EntryHash);
pub type OptionTypedEntryAndHash<T> = Option<TypedEntryAndHash<T>>;


//// Get untyped entry from eh
pub fn get_entry_from_eh(eh: EntryHash) -> ExternResult<Entry> {
   match get(eh, GetOptions::content())? {
      None => zome_error!("get_entry_from_eh(): Entry not found"),
      Some(record) => match record.entry() {
         record::RecordEntry::Present(entry) =>  {
            Ok(entry.clone())
         }
         _ => zome_error!("No Entry at Record"),
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
      return zome_error!("no record found for entry_hash");
   }
   let record = maybe_record.unwrap();
   let entry_type = record.action().entry_type().unwrap().clone();
   Ok(entry_type)
}

/// Get EntryHash from a ActionHash
pub fn get_eh(ah: ActionHash) -> ExternResult<EntryHash> {
   trace!("ah_to_eh() START - get...");
   let maybe_record = get(ah, GetOptions::content())?;
   if let None = maybe_record {
      warn!("ah_to_eh() END - Record not found");
      return zome_error!("ah_to_eh(): Record not found");
   }
   trace!("ah_to_eh() END - Record found");
   return record_to_eh(&maybe_record.unwrap());
}


/// Get EntryHash and typed Entry from a ActionHash
pub fn get_typed_from_ah<T: TryFrom<Entry>>(hash: ActionHash)
   -> ExternResult<(EntryHash, T)>
{
   match get(hash.clone(), GetOptions::content())? {
      Some(record) => {
         let eh = record.action().entry_hash().expect("Converting ActionHash which does not have an Entry");
         Ok((eh.clone(), get_typed_from_record(record)?))
      }
      None => zome_error!("get_typed_from_ah(): Entry not found"),
   }
}


/// Get EntryHash and typed Entry from an EntryHash
pub fn get_typed_from_eh<T: TryFrom<Entry>>(eh: EntryHash) -> ExternResult<T> {
   match get(eh, GetOptions::content())? {
      Some(record) => Ok(get_typed_from_record(record)?),
      None => zome_error!("get_typed_from_eh(): Entry not found"),
   }
}

/// Get typed Entry from Record
pub fn get_typed_from_record<T: TryFrom<Entry>>(record: Record) -> ExternResult<T> {
   match record.entry() {
      record::RecordEntry::Present(entry) =>  {
         let res = T::try_from(entry.clone());
         let err = error::<T>(&format!("get_typed_from_record() failed for: {:?}", entry)).err().unwrap();
         res.map_err(|_|err)
      },
      _ => zome_error!("Could not convert record"),
   }
}


/// Get author from AnyLinkableHash
/// Must be a single author entry type
pub fn get_author(ah: &AnyLinkableHash)
   -> ExternResult<AgentPubKey>
{
   let maybe_maybe_record = get(ah.clone(), GetOptions::content());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let maybe_record = maybe_maybe_record.unwrap();
   if maybe_record.is_none() {
      return zome_error!("no Record found at address");
   }
   let record = maybe_record.unwrap();
   let author = record.action().author();
   Ok(author.to_owned())
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
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let maybe_record = maybe_maybe_record.unwrap();
   if maybe_record.is_none() {
      return zome_error!("no Record found at address");
   }
   let record = maybe_record.unwrap();
   let author = record.action().author();
   let app_entry = get_typed_from_record::<T>(record.clone())?;
   Ok((author.clone(), app_entry))
}


///
pub fn get_latest_typed_from_eh<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
   entry_hash: EntryHash,
) -> ExternResult<OptionTypedEntryAndHash<T>> {
   /// First, make sure we DO have the latest action_hash address
   let maybe_latest_ah = match get_details(entry_hash.clone(), GetOptions::latest())? {
      Some(Details::Entry(details)) => match details.entry_dht_status {
         metadata::EntryDhtStatus::Live => match details.updates.len() {
            // pass out the action associated with this entry
            0 => Some(sah_to_ah(details.actions.first().unwrap().to_owned())),
            _ => {
               let mut sortlist = details.updates.to_vec();
               // unix timestamp should work for sorting
               sortlist.sort_by_key(|update| update.action().timestamp().as_micros());
               // sorts in ascending order, so take the last Record
               let last = sortlist.last().unwrap().to_owned();
               Some(sah_to_ah(last))
            }
         },
         metadata::EntryDhtStatus::Dead => None,
         _ => None,
      },
      _ => None,
   };
   let latest_ah = return_none!(maybe_latest_ah);
   /// Second, go and get that Record, and return its entry and action_address
   let maybe_latest_record = get(latest_ah, GetOptions::latest())?;
   let record = return_none!(maybe_latest_record);
   let maybe_maybe_typed_entry = record.entry().to_app_option::<T>();
   if let Err(e) = maybe_maybe_typed_entry {
      return Err(wasm_error!(WasmErrorInner::Serialize(e)))
   }
   let typed_entry = return_none!(maybe_maybe_typed_entry.unwrap());
   let ah = match record.action() {
      /// we DO want to return the action for the original instead of the updated
      Action::Update(update) => update.original_action_address.clone(),
      Action::Create(_) => record.action_address().clone(),
      _ => unreachable!("Can't have returned a action for a nonexistent entry"),
   };
   let eh =  record.action().entry_hash().unwrap().to_owned();
   /// Done
   Ok(Some((typed_entry, ah, eh)))
}


///
pub fn get_latest_entry(target: EntryHash, option: GetOptions) -> ExternResult<Option<Entry>> {
   let details = get_details(target, option.clone())?;
   match details {
      Some(Details::Entry(EntryDetails { entry, updates, .. })) => {
         // No updates, we are done
         if updates.is_empty() {
            return Ok(Some(entry));
         }
         // Get the latest update via timestamp
         let sah = updates
            .into_iter()
            .fold(
               None,
               |latest: Option<SignedActionHashed>, update| match latest {
                  Some(latest) => {
                     if update.action().timestamp() > latest.action().timestamp() {
                        Some(update)
                     } else {
                        Some(latest)
                     }
                  }
                  None => Some(update),
               },
            )
            .expect("Updates are not empty");
         match sah.action().entry_hash() {
            Some(eh) => {
               let record = get(eh.clone(), GetOptions::content())?.unwrap();
               return Ok(record.entry.into_option());
            },
            None => unreachable!(),
         }
      }
      _ => Ok(None),
   }
}