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
