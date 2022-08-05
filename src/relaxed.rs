use hdk::prelude::*;

///
pub fn create_entry_relaxed<I: EntryDefRegistration, E>(typed: I) -> ExternResult<ActionHash>
   where
      hdk::prelude::Entry: TryFrom<I, Error = E>,
      <hdk::prelude::Entry as TryFrom<I>>::Error: std::fmt::Debug,
      hdk::prelude::WasmError: From<E>,
{
   let create_input = CreateInput::new(
      I::entry_def_id(),
      Entry::try_from(typed).unwrap(),
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


