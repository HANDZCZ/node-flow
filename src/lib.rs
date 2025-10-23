//#![warn(missing_docs)]
#![warn(unused_doc_comments)]
#![warn(clippy::empty_docs)]
#![warn(clippy::tabs_in_doc_comments)]
#![warn(clippy::suspicious_doc_comments)]
#![warn(clippy::test_attr_in_doctest)]
#![warn(rustdoc::private_intra_doc_links)]
#![warn(clippy::empty_line_after_doc_comments)]
#![warn(clippy::pedantic)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_debug_implementations)]
#![forbid(invalid_doc_attributes)]

pub mod context;
pub mod flows;
mod future_utils;
pub mod node;
