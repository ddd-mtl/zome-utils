//! All helper functions calling `get_links()`

use hdk::prelude::*;
use crate::*;

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
   base: EntryHash,
   link_type: LinkType,
   tag: Option<LinkTag>,
   //include_latest_updated_entry: bool,
) -> ExternResult<Vec<(R, Link)>> {
   let links = get_links(AnyLinkableHash::from(base), link_type, tag)?;
   //debug!("get_links_and_load_type() links found: {}", links.len());
   let result_pairs = get_links_details(&mut links.clone(), GetOptions::default())?;
   let typed_pairs = result_pairs
      .iter()
      .flat_map(|pair| match pair.0.clone() {
         Some(Details::Entry(EntryDetails { entry, .. })) => {
            match R::try_from(entry.clone()) {
               Ok(r) => Ok((r, pair.1.clone())),
               Err(_) => error(
                  "Could not convert get_links result to requested type",
               ),
            }
         }
         _ => error("get_links did not return an app entry"),
      })
      .collect();
   Ok(typed_pairs)
}