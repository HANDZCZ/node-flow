//#![warn(missing_docs)]
#![warn(unused_doc_comments)]
#![warn(clippy::empty_docs)]
#![warn(clippy::tabs_in_doc_comments)]
#![warn(clippy::suspicious_doc_comments)]
#![warn(clippy::test_attr_in_doctest)]
#![warn(rustdoc::private_intra_doc_links)]
#![warn(clippy::empty_line_after_doc_comments)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![forbid(rustdoc::broken_intra_doc_links)]
//#![forbid(missing_debug_implementations)]
#![forbid(invalid_doc_attributes)]
#![cfg_attr(all(doc, not(doctest)), feature(async_fn_in_trait))]
#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

pub use async_trait;

mod internal;
pub mod node;
pub mod storage;
