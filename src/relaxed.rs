use hdk::prelude::*;

/// Note: same implementation as create_entry() but with Relaxed chain ordering
pub fn create_entry_relaxed<I: EntryDefRegistration + Clone, E, E2>(input: I) -> ExternResult<ActionHash>
   where
      EntryDefIndex: for<'a> TryFrom<&'a I, Error = E2>,
      EntryVisibility: for<'a> From<&'a I>,
      Entry: TryFrom<I, Error = E>,
      WasmError: From<E>,
      WasmError: From<E2>,
{
   let entry_def_index = EntryDefIndex::try_from(&input)?;
   let visibility = EntryVisibility::from(&input);
   let create_input = CreateInput::new(
      EntryDefLocation::app(entry_def_index),
      visibility,
      input.try_into()?,
      ChainTopOrdering::Relaxed,
   );
   return create(create_input);
}


///
pub fn create_link_relaxed<T: Into<LinkTag>>(
   base_address: EntryHash,
   target_address: EntryHash,
   link_type: LinkType,
   tag: T,
) -> ExternResult<ActionHash> {
   HDK.with(|h| {
      h.borrow().create_link(CreateLinkInput::new(
         base_address.into(),
         target_address.into(),
         link_type,
         tag.into(),
         ChainTopOrdering::Relaxed,
      ))
   })
}


///
pub fn delete_link_relaxed(address: ActionHash) -> ExternResult<ActionHash> {
   HDK.with(|h| {
      h.borrow()
       .delete_link(DeleteLinkInput::new(address, ChainTopOrdering::Relaxed))
   })
}


