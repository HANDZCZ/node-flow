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
