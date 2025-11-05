//! This module contains all the different types of storages.
//!
//! For details on specific behavior, see the documentation of each storage.

/// This module defines and implements in-memory, **branch-local storage**.
///
/// It is used for managing node or flow state during execution on a **per-branch** basis,
/// where the items need to have unique values across branches.
///
/// For details and examples see the documentation of [`LocalStorage`].
pub mod local_storage;
pub use local_storage::LocalStorage;

/// This module defines and implements in-memory, **shared storage**.
///
/// It is used for managing node of flow state **shared across branches**,
/// where the items have only one instance thats is shared across all branches.
///
/// For details and examples see the documentation of [`SharedStorage`].
pub mod shared_storage;
pub use shared_storage::SharedStorage;
