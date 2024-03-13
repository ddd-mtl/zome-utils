//! All helper functions calling `get()`

use hdk::prelude::*;
use std::convert::TryFrom;
use crate::*;
use crate as zome_utils;

pub type TypedEntryAndHash<T> = (T, ActionHash, EntryHash);
pub type OptionTypedEntryAndHash<T> = Option<TypedEntryAndHash<T>>;


//// Get untyped entry from eh
pub fn get_entry_from_eh(eh: EntryHash) -> ExternResult<Entry> {
   match get(eh, GetOptions::content())? {
      None => zome_error!("get_entry_from_eh(): Entry not found"),
      Some(record) => match record.entry() {
         RecordEntry::Present(entry) =>  {
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
         get_entry_type_from_hash(eh.into())?
      },
   };
   Ok(entry_type)
}


/// Get EntryType at address
pub fn get_entry_type_from_hash(hash: AnyDhtHash) -> ExternResult<EntryType> {
   let maybe_record = get(hash, GetOptions::content())?;
   let Some(record) = maybe_record else {
      return zome_error!("no record found for entry_hash");
   };
   let entry_type = record.action().entry_type().ok_or(wasm_error!("No entry at given hash"))?.clone();
   Ok(entry_type)
}


/** */
pub fn get_data_type(hash: AnyLinkableHash) -> ExternResult<String> {
   let Some(dht) = hash.into_any_dht_hash() else {
      return Ok("External".to_owned());
   };
   let maybe_record = get(dht, GetOptions::content())?;
   let Some(record) = maybe_record else {
      return zome_error!("no record found for dht");
   };
   if let Some(entry_type) = record.action().entry_type() {
      let name = match entry_type {
         EntryType::AgentPubKey => "AgentPubKey",
         EntryType::CapClaim => "CapClaim",
         EntryType::CapGrant => "CapGrant",
         EntryType::App(_def) => "App",
      };
      return Ok(name.to_owned());
   }
   let name = match record.action().action_type() {
      ActionType::Dna => "Dna",
      ActionType::AgentValidationPkg=> "AgentValidationPkg",
      ActionType::InitZomesComplete=> "InitZomesComplete",
      ActionType::OpenChain=> "OpenChain",
      ActionType::CloseChain=> "CloseChain",
      ActionType::Create => "Create",
      ActionType::Update => "Update",
      ActionType::Delete => "Delete",
      ActionType::CreateLink => "CreateLink",
      ActionType::DeleteLink => "DeleteLink",
   };
   Ok(name.to_owned())
}


///
pub fn get_app_entry_name(dh: AnyDhtHash) -> ExternResult<(AppEntryName, Record)> {
   /// Grab Entry
   let maybe_maybe_record = get(dh.clone(), GetOptions::content());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let Some(record) = maybe_maybe_record.unwrap() else {
      return error("no Record found at address");
   };
   let RecordEntry::Present(entry) = record.entry() else {
      return error("no Entry found at address");
   };
   /// Grab Type
   let entry_type = get_entry_type(entry)?;
   let EntryType::App(app_entry_def) = entry_type else {
      return error("no AppEntry found at address");
   };
   let aen = get_app_entry_name_from_def(app_entry_def)?;

   Ok((aen, record))
}


///
pub fn get_app_entry_name_from_def(app_entry_def: AppEntryDef) -> ExternResult<AppEntryName> {
   /// Grab zome
   let dna = dna_info()?;
   let this_zome_info = zome_info()?;
   let mut entry_defs: EntryDefs = this_zome_info.entry_defs;
   /// Grab entry_def from different zome
   if this_zome_info.id != app_entry_def.zome_index {
      let zome_name = dna.zome_names[app_entry_def.zome_index.0 as usize].clone();
      let response = call(CallTargetCell::Local, zome_name, "entry_defs".into(), None, ())?;
      entry_defs  = decode_response(response)?;
   }
   /// Grab entry_def
   let entry_def: EntryDef = entry_defs.0[app_entry_def.entry_index.0 as usize].clone();
   let EntryDefId::App(name) = entry_def.id else {
      return error("Not an AppEntry");
   };
   /// Done
   Ok(name)
}


/// Get EntryHash from a ActionHash
pub fn get_eh(ah: ActionHash) -> ExternResult<EntryHash> {
   trace!("ah_to_eh() START - get...");
   let maybe_record = get(ah, GetOptions::content())?;
   let Some(record) = maybe_record else {
      warn!("ah_to_eh() END - Record not found");
      return zome_error!("ah_to_eh(): Record not found");
   };
   trace!("ah_to_eh() END - Record found");
   return record_to_eh(&record);
}


/// Get EntryHash and typed Entry from a ActionHash
pub fn get_typed_from_ah<T: TryFrom<Entry>>(hash: ActionHash) -> ExternResult<(EntryHash, T)> {
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
   let RecordEntry::Present(entry) = record.entry() else {
      return zome_error!("Could not convert record");
   };
   let res = T::try_from(entry.clone());
   let err = error::<T>(&format!("get_typed_from_record() failed for: {:?}", entry)).err().unwrap();
   res.map_err(|_|err)
}


/// Get author from AnyDhtHash
/// Must be a single author entry type
pub fn get_author(dht_hash: &AnyDhtHash) -> ExternResult<AgentPubKey> {
   let maybe_maybe_record = get(dht_hash.clone(), GetOptions::content());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let Some(record) = maybe_maybe_record.unwrap() else {
      return zome_error!("no Record found at address");
   };
   let author = record.action().author();
   Ok(author.to_owned())
}


/// Get typed Entry from AnyLinkableHash
/// Must be a single author entry type
pub fn get_typed_and_record<T: TryFrom<Entry>>(any_hash: &AnyLinkableHash)
   -> ExternResult<(Record, T)>
{
   let dh = into_dht_hash(any_hash.to_owned())?;
   let maybe_maybe_record = get(dh, GetOptions::content());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let Some(record) = maybe_maybe_record.unwrap() else {
      return zome_error!("no Record found at address");
   };
   let app_entry = get_typed_from_record::<T>(record.clone())?;
   Ok((record, app_entry))
}


/// Get typed Entry and its author from AnyLinkableHash
/// Must be a single author entry type
pub fn get_typed_and_author<T: TryFrom<Entry>>(any_hash: &AnyLinkableHash)
   -> ExternResult<(AgentPubKey, T)>
{
   let dh = into_dht_hash(any_hash.to_owned())?;
   let maybe_maybe_record = get(dh, GetOptions::content());
   if let Err(err) = maybe_maybe_record {
      warn!("Failed getting Record: {}", err);
      return Err(err);
   }
   let Some(record) = maybe_maybe_record.unwrap() else {
      return zome_error!("no Record found at address");
   };
   let author = record.action().author();
   let app_entry = get_typed_from_record::<T>(record.clone())?;
   Ok((author.clone(), app_entry))
}


///
pub fn get_latest_typed_from_eh<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
   entry_hash: EntryHash,
) -> ExternResult<OptionTypedEntryAndHash<T>> {
   /// First, make sure we DO have the latest action_hash address
   let maybe_maybe_details = get_details(entry_hash.clone(), GetOptions::latest())?;
   let Some(Details::Entry(details)) = maybe_maybe_details else {
      return Ok(None);
   };
   if details.entry_dht_status != metadata::EntryDhtStatus::Live {
      return Ok(None);
   }
   let latest_ah = match details.updates.len() {
      // pass out the action associated with this entry
      0 => sah_to_ah(details.actions.first().unwrap().to_owned()),
      _ => {
         let mut sortlist = details.updates.to_vec();
         // unix timestamp should work for sorting
         sortlist.sort_by_key(|update| update.action().timestamp().as_micros());
         // sorts in ascending order, so take the last Record
         let last = sortlist.last().unwrap().to_owned();
         sah_to_ah(last)
      }
   };
   /// Second, go and get that Record, and return its entry and action_address
   let Some(record) = get(latest_ah, GetOptions::latest())? else {
      return Ok(None);
   };
   let maybe_maybe_typed_entry = record.entry().to_app_option::<T>();
   if let Err(e) = maybe_maybe_typed_entry {
      return Err(wasm_error!(WasmErrorInner::Serialize(e)))
   }
   let Some(typed_entry) = maybe_maybe_typed_entry.unwrap() else {return Ok(None)};
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
   let Some(Details::Entry(EntryDetails { entry, updates, .. })) = details else {
      return Ok(None);
   };
   /// No updates, we are done
   if updates.is_empty() {
      return Ok(Some(entry));
   }
   /// Get the latest update via timestamp
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
   let Some(eh) = sah.action().entry_hash() else {
      unreachable!();
   };
   let record = get(eh.clone(), GetOptions::content())?.unwrap();
   Ok(record.entry.into_option())
}