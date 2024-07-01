mod path_utils;
pub use path_utils::*;

mod call;
mod debug;
mod get;
mod links;
mod query;
mod relaxed;
mod utils;

pub use call::*;
pub use debug::*;
pub use get::*;
pub use links::*;
pub use query::*;
pub use relaxed::*;
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


// #[macro_export]
// macro_rules! else_none {
//         ( $e:expr ) => {
//             $e else {return Ok(None)};
//         }
// }


