use hdk::prelude::*;

/// Note: same implementation as create_entry() but with Relaxed chain ordering
pub fn create_entry_relaxed<I, E, E2>(typed: I) -> ExternResult<ActionHash>
   where
      ScopedEntryDefIndex: for<'a> TryFrom<&'a I, Error = E2>,
      EntryVisibility: for<'a> From<&'a I>,
      Entry: TryFrom<I, Error = E>,
      WasmError: From<E>,
      WasmError: From<E2>,
{
   // wtf
   let ScopedEntryDefIndex {
      zome_index,
      zome_type: entry_def_index,
   } = (&typed).try_into()?;

   let create_input = CreateInput::new(
      EntryDefLocation::app(zome_index, entry_def_index),
      EntryVisibility::from(&typed),
      typed.try_into()?, //entry,
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
         zome_info()?.id,
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


