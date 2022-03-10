//! All helper functions calling `get()`

use hdk::prelude::*;
use std::convert::TryFrom;
use crate::*;

pub type TypedEntryAndHash<T> = (T, HeaderHash, EntryHash);
pub type OptionTypedEntryAndHash<T> = Option<TypedEntryAndHash<T>>;

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
   let maybe_element = get(eh, GetOptions::latest())?;
   if maybe_element.is_none() {
      return error("no element found for entry_hash");
   }
   let element = maybe_element.unwrap();
   let entry_type = element.header().entry_type().unwrap().clone();
   Ok(entry_type)
}

/// Get EntryHash from a HeaderHash
pub fn get_eh(hh: HeaderHash) -> ExternResult<EntryHash> {
   trace!("hh_to_eh() START - get...");
   let maybe_element = get(hh, GetOptions::content())?;
   if let None = maybe_element {
      warn!("hh_to_eh() END - Element not found");
      return error("hh_to_eh(): Element not found");
   }
   trace!("hh_to_eh() END - Element found");
   return el_to_eh(&maybe_element.unwrap());
}


/// Get EntryHash and typed Entry from a HeaderHash
pub fn get_typed_from_hh<T: TryFrom<Entry>>(hash: HeaderHash)
   -> ExternResult<(EntryHash, T)>
{
   match get(hash.clone(), GetOptions::content())? {
      Some(element) => {
         let eh = element.header().entry_hash().expect("Converting HeaderHash which does not have an Entry");
         Ok((eh.clone(), get_typed_from_el(element)?))
      },
      None => error("get_typed_from_hh(): Entry not found"),
   }
}


/// Get EntryHash and typed Entry from an EntryHash
pub fn get_typed_from_eh<T: TryFrom<Entry>>(eh: EntryHash) -> ExternResult<T> {
   match get(eh, GetOptions::content())? {
      Some(element) => Ok(get_typed_from_el(element)?),
      None => error("get_typed_from_eh(): Entry not found"),
   }
}

/// Get typed Entry from Element
pub fn get_typed_from_el<T: TryFrom<Entry>>(element: Element) -> ExternResult<T> {
   match element.entry() {
      element::ElementEntry::Present(entry) =>  {
         let res = T::try_from(entry.clone());
         let err = error::<T>(&format!("get_typed_from_el() failed for: {:?}", entry)).err().unwrap();
         res.map_err(|_|err)
      },
      _ => error("Could not convert element"),
   }
}


/// Get typed Entry and its author from EntryHash
/// Must be a single author entry type
pub fn get_typed_and_author<T: TryFrom<Entry>>(eh: &EntryHash)
   -> ExternResult<(AgentPubKey, T)>
{
   let maybe_maybe_element = get(eh.clone(), GetOptions::latest());
   if let Err(err) = maybe_maybe_element {
      warn!("Failed getting element: {}", err);
      return Err(err);
   }
   let maybe_element = maybe_maybe_element.unwrap();
   if maybe_element.is_none() {
      return error("no element found at address");
   }
   let element = maybe_element.unwrap();
   //assert!(entry_item.headers.len() > 0);
   //assert!(entry_item.headers[0].provenances().len() > 0);
   let author = element.header().author();
   let app_entry = get_typed_from_el::<T>(element.clone())?;
   Ok((author.clone(), app_entry))
}


///
pub fn get_latest_typed_from_eh<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
   entry_hash: EntryHash,
) -> ExternResult<OptionTypedEntryAndHash<T>> {
   /// First, make sure we DO have the latest header_hash address
   let maybe_latest_hh = match get_details(entry_hash.clone(), GetOptions::latest())? {
      Some(Details::Entry(details)) => match details.entry_dht_status {
         metadata::EntryDhtStatus::Live => match details.updates.len() {
            // pass out the header associated with this entry
            0 => Some(shh_to_hh(details.headers.first().unwrap().to_owned())),
            _ => {
               let mut sortlist = details.updates.to_vec();
               // unix timestamp should work for sorting
               sortlist.sort_by_key(|update| update.header().timestamp().as_micros());
               // sorts in ascending order, so take the last element
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
   /// Second, go and get that element, and return its entry and header_address
   let maybe_latest_el = get(latest_hh, GetOptions::latest())?;
   let el = return_none!(maybe_latest_el);
   let maybe_typed_entry = el.entry().to_app_option::<T>()?;
   let typed_entry = return_none!(maybe_typed_entry);
   let hh = match el.header() {
      /// we DO want to return the header for the original instead of the updated
      Header::Update(update) => update.original_header_address.clone(),
      Header::Create(_) => el.header_address().clone(),
      _ => unreachable!("Can't have returned a header for a nonexistent entry"),
   };
   let eh =  el.header().entry_hash().unwrap().to_owned();
   /// Done
   Ok(Some((typed_entry, hh, eh)))
}