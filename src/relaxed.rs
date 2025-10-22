use hdk::prelude::*;

/// Note: same implementation as update_entry() but with Relaxed chain ordering
pub fn update_entry_relaxed<I, E>(hash: ActionHash, input: I) -> ExternResult<ActionHash>
where
  Entry: TryFrom<I, Error = E>,
  WasmError: From<E>,
{
   let input = UpdateInput {
      original_action_address: hash,
      entry: input.try_into()?,
      chain_top_ordering: ChainTopOrdering::Relaxed,
   };
   update(input)
}


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
pub fn create_link_relaxed<T, E>(
   base_address: impl Into<AnyLinkableHash>,
   target_address: impl Into<AnyLinkableHash>,
   link_type: T,
   tag: impl Into<LinkTag>,
) -> ExternResult<ActionHash>
   where
      ScopedLinkType: TryFrom<T, Error = E>,
      WasmError: From<E>,
{
   let ScopedLinkType {
      zome_index,
      zome_type: link_type,
   } = link_type.try_into()?;
   HDK.with(|h| {
      h.borrow().create_link(CreateLinkInput::new(
         base_address.into(),
         target_address.into(),
         zome_index,
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
       .delete_link(DeleteLinkInput::new(address, GetOptions::default(), ChainTopOrdering::Relaxed))
   })
}

///
pub fn delete_entry_relaxed(address: ActionHash) -> ExternResult<ActionHash> {
   HDK.with(|h| {
      h.borrow()
        .delete(DeleteInput::new(address, ChainTopOrdering::Relaxed))
   })
}
