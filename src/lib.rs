#![cfg_attr(not(test), no_std)]

//! Various extention traits for providing asynchronous higher-order functions.
//!
//! # Examples
//!
//! ```
//! # #[tokio::main]
//! # async fn main() {
//! // This won't make any name conflicts since all imports inside prelude are anonymous.
//! use async_hofs::prelude::*;
//!
//! assert_eq!(
//!     Some(1).async_map(|x| async move { x + 2 }).await,
//!     Some(3),
//! );
//!
//! type Result = core::result::Result<i32, i32>;
//!
//! assert_eq!(
//!     Result::Ok(1).async_and_then(|_| async move { Err(77) }).await,
//!     Result::Err(77)
//! );
//! # }
//! ```

mod async_util;
mod foo;
pub mod iter;
pub mod option;
pub mod result;
pub mod stream;

pub mod prelude {
    pub use crate::iter::AsyncMapExt as _;
    pub use crate::option::AsyncMapExt as _;
    pub use crate::result::AsyncMapExt as _;
    pub use crate::stream::AsyncMapExt as _;
}
