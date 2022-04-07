use hdk::prelude::*;

///
pub fn create_entry_relaxed<T: EntryDefRegistration>(typed: T) -> ExternResult<HeaderHash>
   where
      hdk::prelude::Entry: TryFrom<T>,
      <hdk::prelude::Entry as TryFrom<T>>::Error: std::fmt::Debug,
{
   let create_input = CreateInput::new(
      T::entry_def_id(),
      Entry::try_from(typed).unwrap(),
      ChainTopOrdering::Relaxed,
   );
   return create_entry(create_input);
}


///
pub fn create_link_relaxed<T: Into<LinkTag>>(
   base_address: EntryHash,
   target_address: EntryHash,
   tag: T,
) -> ExternResult<HeaderHash> {
   HDK.with(|h| {
      h.borrow().create_link(CreateLinkInput::new(
         base_address,
         target_address,
         tag.into(),
         ChainTopOrdering::Relaxed,
      ))
   })
}


///
pub fn delete_link_relaxed(address: HeaderHash) -> ExternResult<HeaderHash> {
   HDK.with(|h| {
      h.borrow()
       .delete_link(DeleteLinkInput::new(address, ChainTopOrdering::Relaxed))
   })
}


