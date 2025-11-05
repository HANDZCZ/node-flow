//! This module contains all the necessary components for describing the structure of a flow.
//!
//! It also contains a [`D2Describer`] for formatting [`Description`] into [D2](https://d2lang.com/) graph syntax.
//!
//! For details, see the documentation of [`Description`].

mod design;
pub use design::*;

#[cfg(feature = "d2describer")]
mod d2;
#[cfg(feature = "d2describer")]
pub use d2::*;

pub(crate) fn remove_generics_from_name(orig_name: &mut String) {
    let generic_start_idx = orig_name.find('<').unwrap_or(orig_name.len());
    orig_name.truncate(generic_start_idx);
}
