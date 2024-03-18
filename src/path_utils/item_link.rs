use hdk::prelude::*;

/// Struct holding info about the link between an Item and its LeafAnchor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemLink {
  pub item_hash: AnyLinkableHash,
  pub tag: Vec<u8>, // LinkTag ; TODO
  /// Flattened ScopedLinkType
  pub zome_index: u8,
  pub link_index: u8,
}


impl ItemLink {
  pub fn from(link: Link) -> ItemLink {
    ItemLink {
      item_hash: link.target,
      tag: link.tag.0,
      zome_index: link.zome_index.0,
      link_index: link.link_type.0,
    }
  }
}


/// Replacement of `get_links()` that converts all results to ItemLinks
pub fn get_itemlinks(path: Path, link_filter: LinkTypeFilter, link_tag: Option<LinkTag>) -> ExternResult<Vec<ItemLink>> {
  /// Grab Links
  let links = get_links( GetLinksInput {
    base_address: AnyLinkableHash::from(path.path_entry_hash()?),
    link_type: link_filter,
    get_options: GetOptions::network(),
    tag_prefix: link_tag,
    before: None, after: None, author: None,
  }
  )?;
  /// Convert to ItemLinks
  let res = links.into_iter().map(|link| ItemLink::from(link)).collect();
  /// Done
  Ok(res)
}
