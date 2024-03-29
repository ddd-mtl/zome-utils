//! All helper functions calling `get_links()`

use hdk::prelude::*;
use hdk::prelude::holo_hash::AnyLinkableHashPrimitive;
use hdk::prelude::holo_hash::hash_type::AnyLinkable;
use crate as zome_utils;


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
            GetInput::new(link.target.clone().into_entry_hash().unwrap().into(), GetOptions::content())
         },
         AnyLinkable::Action => {
            if let Some(AnyLinkable::Entry) = maybe_filter {
               continue;
            }
            GetInput::new(link.target.clone().into_action_hash().unwrap().into(), GetOptions::content())
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
pub fn get_typed_from_links<R: TryFrom<Entry>>(
   base: impl Into<AnyLinkableHash>,
   link_type: impl LinkTypeFilterExt,
   tag: Option<LinkTag>,
   //include_latest_updated_entry: bool,
) -> ExternResult<Vec<(R, Link)>> {
   let links = get_links(base, link_type, tag)?;
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


///
pub fn get_typed_from_actions_links<T: TryFrom<Entry>>(
   base: impl Into<AnyLinkableHash>,
   link_type: impl LinkTypeFilterExt,
   tag: Option<LinkTag>,
   //include_latest_updated_entry: bool,
) -> ExternResult<Vec<(AgentPubKey, T)>> {
   let links = get_links(base, link_type, tag)?;
   //debug!("get_typed_from_actions_links() links found: {}", links.len());
   let input_pairs = links_to_GetInputs(links, Some(AnyLinkable::Action));
   //debug!("get_typed_from_actions_links() input_pairs: {}", input_pairs.len());
   let mut typed_pairs: Vec<(AgentPubKey, T)> = Vec::new();
   for pair in input_pairs.into_iter() {
      let Ok(p) = zome_utils::get_typed_and_author::<T>(&pair.1.target) 
         else {continue};
      typed_pairs.push(p);
   }
   //debug!("get_typed_from_actions_links() typed_pairs: {}", typed_pairs.len());
   Ok(typed_pairs)
}