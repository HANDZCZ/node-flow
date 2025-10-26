#[cfg(feature = "shared_storage_impl")]
mod implementation;
#[cfg(feature = "shared_storage_impl")]
pub use implementation::*;
mod design;
pub use design::*;
