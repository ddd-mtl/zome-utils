#![allow(unused_doc_comments)]

mod debug;
mod get;
mod links;
mod query;
mod utils;
mod relaxed;
mod call;


pub use debug::*;
pub use get::*;
pub use links::*;
pub use query::*;
pub use utils::*;
pub use relaxed::*;
pub use call::*;


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


// #[macro_export]
// macro_rules! else_none {
//         ( $e:expr ) => {
//             $e else {return Ok(None)};
//         }
// }


