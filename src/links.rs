//! All helper functions calling `get_links()`

use hdk::prelude::*;
use hdk::prelude::holo_hash::AnyLinkableHashPrimitive;
use hdk::prelude::holo_hash::hash_type::AnyLinkable;
use crate as zome_utils;


///-------------------------------------------------------------------------------------------------
///  impl GetLinksInput
///-------------------------------------------------------------------------------------------------

///
pub fn link_input_full(
   base_address: AnyLinkableHash,
   link_type: LinkTypeFilter,
   get_options: GetOptions,
   tag_prefix: Option<LinkTag>,
   after: Option<Timestamp>,
   before: Option<Timestamp>,
   author: Option<AgentPubKey>,
) -> GetLinksInput {
   GetLinksInput {
      base_address,
      link_type,
      get_options,
      tag_prefix,
      after,
      before,
      author,
   }
}


///
pub fn link_input(base: AnyLinkableHash, link_type: LinkTypeFilter) -> GetLinksInput {
   GetLinksInput {
      base_address: base,
      link_type,
      get_options: GetOptions::network(),
      tag_prefix: None,
      after: None,
      before: None,
      author: None,
   }
}


///-------------------------------------------------------------------------------------------------

///
pub fn into_dht_hash(yh: AnyLinkableHash) -> ExternResult<AnyDhtHash> {
   match yh.into_primitive() {
      AnyLinkableHashPrimitive::Entry(eh) => Ok(AnyDhtHash::from(eh)),
      AnyLinkableHashPrimitive::Action(ah) => Ok(AnyDhtHash::from(ah)),
      AnyLinkableHashPrimitive::External(_xh) => zome_utils::zome_error!("AnyDhtHash is of an external link type"),
   }
}


#[allow(non_snake_case)]
fn links_to_GetInputs(links: Vec<Link>, maybe_filter: Option<AnyLinkable>) -> Vec<(GetInput, Link)> {
   let mut get_inputs: Vec<(GetInput, Link)> = Vec::new();
   for link in links.into_iter() {
      let input = match link.target.hash_type() {
         AnyLinkable::Entry => {
            if let Some(AnyLinkable::Action) = maybe_filter {
               continue;
            }            
            GetInput::new(link.target.clone().into_entry_hash().unwrap().into(), GetOptions::network())
         },
         AnyLinkable::Action => {
            if let Some(AnyLinkable::Entry) = maybe_filter {
               continue;
            }
            GetInput::new(link.target.clone().into_action_hash().unwrap().into(), GetOptions::network())
         }
         AnyLinkable::External => continue,
      };
      get_inputs.push((input, link));
   }
   get_inputs
}


// /// optimized get details by links
// pub fn get_links_details(links: &mut Vec<Link>) -> ExternResult<Vec<(Option<Details>, Link)>> {
//    let get_inputs = links_to_GetInputs(links)?;
//    debug!("get_links_details() get_inputs: {:?}", get_inputs);
//    let details = HDK.with(|hdk| hdk.borrow().get_details(get_inputs))?;
//    let pairs = details.into_iter().map(|x|  (x, links.pop().unwrap())).collect();
//    debug!("get_links_details() pairs: {:?}", pairs);
//    Ok(pairs)
// }


///
pub fn get_typed_from_links<R: TryFrom<Entry>>(input: GetLinksInput) -> ExternResult<Vec<(R, Link)>> {
   let links = get_links(input)?;
   //debug!("get_typed_from_links() links found: {}", links.len());
   let input_pairs = links_to_GetInputs(links, None);
   //debug!("get_typed_from_links() input_pairs: {}", input_pairs.len());
   let mut typed_pairs: Vec<(R, Link)> = Vec::new();
   for pair in input_pairs.into_iter() {
      let Some(record) = get(pair.0.any_dht_hash, pair.0.get_options)? 
         else {continue};
      //debug!("get_typed_from_links() record: {:?}", record);
      let Ok(r) = zome_utils::get_typed_from_record::<R>(record) 
         else {continue};
      typed_pairs.push((r, pair.1.clone()));
   }
   //debug!("get_typed_from_links() typed_pairs: {}", typed_pairs.len());
   Ok(typed_pairs)
}


/// Returns Vec of: CreateLinkHash, LinkTarget, LinkAuthor, TypedEntry
pub fn get_typed_from_actions_links<T: TryFrom<Entry>>(
   input: GetLinksInput,
) -> ExternResult<Vec<(ActionHash, AnyLinkableHash, AgentPubKey, T)>> {
   let links = get_links(input)?;
   //debug!("get_typed_from_actions_links() links found: {}", links.len());
   let input_pairs = links_to_GetInputs(links, Some(AnyLinkable::Action));
   //debug!("get_typed_from_actions_links() input_pairs: {}", input_pairs.len());
   let mut tuples: Vec<(ActionHash, AnyLinkableHash, AgentPubKey, T)> = Vec::new();
   for (_input, link) in input_pairs.into_iter() {
      let Ok(p) = zome_utils::get_typed_and_author::<T>(&link.target)
         else {continue};
      tuples.push((link.create_link_hash, link.target, p.0, p.1));
   }
   //debug!("get_typed_from_actions_links() typed_pairs: {}", typed_pairs.len());
   Ok(tuples)
}