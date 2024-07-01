//! All helper functions calling `get()`

use hdk::prelude::*;
use std::convert::TryFrom;
use crate::*;
use crate as zome_utils;

pub type TypedEntryAndHash<T> = (T, ActionHash, EntryHash);
pub type OptionTypedEntryAndHash<T> = Option<TypedEntryAndHash<T>>;


/// Get Record from AnyDhtHash
pub fn get_record(dh: AnyDhtHash) -> ExternResult<Record> {
   let maybe_record = get(dh, GetOptions::network())?;
   let Some(record) = maybe_record else {
      return zome_error!("no Record found at given hash");
   };
   Ok(record)
}

/// Get untyped entry from eh
pub fn get_entry(dh: AnyDhtHash) -> ExternResult<Entry> {
   let record = get_record(dh)?;
   let RecordEntry::Present(entry) = record.entry()
       else { return  zome_error!("Record does not hold an Entry"); };
   Ok(entry.to_owned())
}


/// Get untyped entry from eh
pub fn get_entry_from_eh(eh: EntryHash) -> ExternResult<Entry> {
   let record = get_record(AnyDhtHash::from(eh))?;
   let RecordEntry::Present(entry) = record.entry()
      else { return  zome_error!("Record does not hold an Entry"); };
   Ok(entry.clone())
}


/// Get EntryHash from a ActionHash
pub fn get_eh(ah: ActionHash) -> ExternResult<EntryHash> {
   let record = get_record(AnyDhtHash::from(ah))?;
   let  Some(eh) = record.action().entry_hash() else {
      return zome_error!("Record does not hold an EntryHash");
   };
   Ok(eh.to_owned())
}


/// Get author from AnyDhtHash
/// Must be a single author entry type
pub fn get_author(dh: AnyDhtHash) -> ExternResult<AgentPubKey> {
   let record = get_record(dh)?;
   let author = record.action().author();
   Ok(author.to_owned())
}


/// Get typed Entry from Record
pub fn get_typed_from_record<T: TryFrom<Entry>>(record: Record) -> ExternResult<T> {
   let RecordEntry::Present(entry) = record.entry()
       else { return zome_error!("Record does not hold an Entry"); };
   let res = T::try_from(entry.clone());
   let err = error::<T>(&format!("Converting Entry to type failed for entry: {:?}", entry)).err().unwrap();
   return res.map_err(|_|err);
}


/// Get EntryHash and typed Entry from an EntryHash
pub fn get_typed_from_eh<T: TryFrom<Entry>>(eh: EntryHash) -> ExternResult<T> {
   let record = get_record(AnyDhtHash::from(eh))?;
   Ok(get_typed_from_record(record)?)
}

/// Get typed Entry from an ActionHash
pub fn get_typed_from_ah<T: TryFrom<Entry>>(ah: ActionHash) -> ExternResult<(EntryHash, T)> {
   let record = get_record(AnyDhtHash::from(ah))?;
   let  Some(eh) = record.action().entry_hash() else {
      return zome_error!("Record does not hold an EntryHash");
   };
   Ok((eh.to_owned(), get_typed_from_record(record)?))
}


/// Get typed Entry from AnyLinkableHash
/// Must be a single author entry type
pub fn get_typed_and_record<T: TryFrom<Entry>>(lh: AnyLinkableHash) -> ExternResult<(Record, T)>
{
   let dh = into_dht_hash(lh)?;
   let record = get_record(dh)?;
   let typed = get_typed_from_record::<T>(record.clone())?;
   Ok((record, typed))
}


/// Get typed Entry and its author from AnyLinkableHash
/// Must be a single author entry type
pub fn get_typed_and_author<T: TryFrom<Entry>>(lh: AnyLinkableHash) -> ExternResult<(AgentPubKey, T)>
{
   let dh = into_dht_hash(lh)?;
   let record = get_record(dh)?;
   let author = record.action().author();
   let app_entry = get_typed_from_record::<T>(record.clone())?;
   Ok((author.clone(), app_entry))
}


///
pub fn get_app_entry_name(dh: AnyDhtHash, cell_target: CallTargetCell) -> ExternResult<(AppEntryName, Entry)>
{
   /// Grab Entry
   let entry = get_entry(dh)?;
   /// Grab Type
   let entry_type = get_entry_type(&entry)?;
   let EntryType::App(app_entry_def) = entry_type else {
      return zome_error!("no AppEntry found at given hash");
   };
   let aen = get_app_entry_name_from_def(app_entry_def, cell_target)?;
   ///
   Ok((aen, entry))
}


///
pub fn get_app_entry_name_from_def(app_entry_def: AppEntryDef, cell_target: CallTargetCell) -> ExternResult<AppEntryName>
{
   /// Grab zome
   let dna = dna_info()?;
   let this_zome_info = zome_info()?;
   let mut entry_defs: EntryDefs = this_zome_info.entry_defs;
   /// Grab entry_def from different zome
   if this_zome_info.id != app_entry_def.zome_index {
      let zome_name = dna.zome_names[app_entry_def.zome_index.0 as usize].clone();
      let response = call(cell_target, zome_name, "entry_defs".into(), None, ())?;
      entry_defs  = decode_response(response)?;
   }
   /// Grab entry_def
   let entry_def: EntryDef = entry_defs.0[app_entry_def.entry_index.0 as usize].clone();
   let EntryDefId::App(name) = entry_def.id else {
      return zome_error!("Not an AppEntry");
   };
   /// Done
   Ok(name)
}


///
pub fn get_latest_typed_from_eh<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
   entry_hash: EntryHash,
) -> ExternResult<OptionTypedEntryAndHash<T>> {
   /// First, make sure we DO have the latest action_hash address
   let maybe_maybe_details = get_details(entry_hash.clone(), GetOptions::network())?;
   let Some(Details::Entry(details)) = maybe_maybe_details else {
      return Ok(None);
   };
   if details.entry_dht_status != EntryDhtStatus::Live {
      return Ok(None);
   }
   let latest_ah = match details.updates.len() {
      // pass out the action associated with this entry
      0 => details.actions.first().unwrap().as_hash().to_owned(),
      _ => {
         let mut sortlist = details.updates.to_vec();
         // unix timestamp should work for sorting
         sortlist.sort_by_key(|update| update.action().timestamp().as_micros());
         // sorts in ascending order, so take the last Record
         let last = sortlist.last().unwrap().to_owned();
         last.as_hash().to_owned()
      }
   };
   /// Second, go and get that Record, and return its entry and action_address
   let Some(record) = get(latest_ah, GetOptions::network())? else {
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
   let record = get(eh.clone(), GetOptions::network())?.unwrap();
   Ok(record.entry.into_option())
}


/// Recursively call get_details() until no updates are found
/// If multiple updates are found. It will take the last one in the list.
pub fn get_latest_record(action_hash: ActionHash) -> ExternResult<Record> {
   let Some(details) = get_details(action_hash, GetOptions::default())?
       else { return zome_error!("Record not found")};
   match details {
      Details::Entry(_) => zome_error!("Malformed details"),
      Details::Record(element_details) => {
         match element_details.updates.last() {
            Some(update) => get_latest_record(update.action_address().clone()),
            None => Ok(element_details.record),
         }
      },
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
         get_entry_type_at(eh.into())?
      },
   };
   Ok(entry_type)
}


/// Get EntryType at address
pub fn get_entry_type_at(dh: AnyDhtHash) -> ExternResult<EntryType> {
   let record = get_record(dh)?;
   let Some(entry_type) = record.action().entry_type() else {
      return zome_error!("No Entry at given hash");
   };
   ///
   Ok(entry_type.to_owned())
}


///
pub fn get_linkable_type(hash: AnyLinkableHash) -> ExternResult<String> {
   let Some(dht) = hash.into_any_dht_hash() else {
      return Ok("External".to_owned());
   };
   let maybe_record = get(dht, GetOptions::network())?;
   let Some(record) = maybe_record else {
      return zome_error!("no Record found at given hash");
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