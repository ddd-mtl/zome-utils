#![allow(non_upper_case_globals)]
#![allow(unused_doc_comments)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

mod debug;
mod get;
mod links;
mod query;
mod utils;
mod relaxed;
mod call;
mod path_utils;

pub use debug::*;
pub use get::*;
pub use links::*;
pub use query::*;
pub use utils::*;
pub use relaxed::*;
pub use call::*;
pub use path_utils::*;

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


