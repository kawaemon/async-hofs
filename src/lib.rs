#![cfg_attr(not(test), no_std)]

//! Various extention traits for providing asynchronous higher-order functions.
//! Currently [`map`] function is implemented to [`Result`], [`Option`],
//! [`Iterator`] and [`Stream`].
//!
//! [`map`]: core::option::Option::map
//!
//! [`Result`]: core::result::Result
//! [`Option`]: core::option::Option
//! [`Iterator`]: core::iter::Iterator
//! [`Stream`]: futures_core::Stream
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
//! # }
//! ```

mod async_util;
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
