//! This module contains basic traits that should be implemented for node context.
//!
//! Traits in this module can be used by nodes to restrict the context type and ensure that it can perform certain functions.
//! It also contains definitions and implementations of different types of storages.
//!
//! For details, see the documentation of each trait.

mod traits;
pub use traits::*;
pub mod storage;
