#![allow(unused_doc_comments)]

mod debug;
mod get;
mod links;
mod query;
mod utils;
mod relaxed;


pub use debug::*;
pub use get::*;
pub use links::*;
pub use query::*;
pub use utils::*;
pub use relaxed::*;

//----------------------------------------------------------------------------------------

#[macro_export]
macro_rules! return_none {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return Ok(None),
        }
    }
}

