#![allow(unused_doc_comments)]

//#![allow(non_upper_case_globals)]
//#![allow(non_camel_case_types)]
//#![allow(non_snake_case)]
//#![allow(unused_attributes)]

mod debug;
mod get;
mod links;
mod query;
mod system;
mod utils;


pub use debug::*;
pub use get::*;
pub use links::*;
pub use query::*;
pub use system::*;
pub use utils::*;

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

