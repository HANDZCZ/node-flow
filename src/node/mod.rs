//! This module contains the [`Node`] trait and everything related to it.
//!
//! For details, see the documentation of each item.

mod base;
pub use base::*;
mod output;
pub use output::*;
#[cfg(feature = "boxed_node")]
mod boxed;
mod macros;
#[cfg(feature = "boxed_node")]
pub use boxed::*;
