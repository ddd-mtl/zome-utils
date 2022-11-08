//! All helper functions calling `get_links()`

use hdk::prelude::*;
use crate as zome_utils;

/// optimized get details by links
pub fn get_links_details(links: &mut Vec<Link>, option: GetOptions) -> ExternResult<Vec<(Option<Details>, Link)>> {
   let get_inputs: Vec<GetInput> = links
      .into_iter()
      .map(|link| GetInput::new(link.target.clone().into(), option.clone()))
      .collect();
   let details = HDK.with(|hdk| hdk.borrow().get_details(get_inputs))?;
   assert!(details.len() == links.len());
   let pairs = details.iter().map(|x|  (x.clone(), links.pop().unwrap())).collect();
   Ok(pairs)
}


///
pub fn get_typed_from_links<R: TryFrom<Entry>>(
   base: impl Into<AnyLinkableHash>,
   link_type: impl LinkTypeFilterExt,
   tag: Option<LinkTag>,
   //include_latest_updated_entry: bool,
) -> ExternResult<Vec<(R, Link)>> {
   let links = get_links(base, link_type, tag)?;
   debug!("get_links_and_load_type() links found: {}", links.len());
   let result_pairs = get_links_details(&mut links.clone(), GetOptions::default())?;
   debug!("get_links_and_load_type() result_pairs: {}", result_pairs.len());
   let mut typed_pairs: Vec<(R, Link)> = Vec::new();
   for pair in result_pairs {
      let Some(details) = pair.0.clone() else {
         continue;
      };
      let typed = match details {
         Details::Entry(EntryDetails { entry, .. }) => {
            let Ok(r) = R::try_from(entry.clone()) else {continue};
            r
         }
         Details::Record(RecordDetails { record, .. }) => {
            let Ok(r) = zome_utils::get_typed_from_record::<R>(record) else {continue};
            r
         }
      };
      typed_pairs.push((typed, pair.1.clone()));
   }
   debug!("get_links_and_load_type() typed_pairs: {}", typed_pairs.len());
   Ok(typed_pairs)
}