#![warn(
    // missing_docs,
    clippy::tabs_in_doc_comments,
    clippy::suspicious_doc_comments,
    clippy::test_attr_in_doctest,
    clippy::empty_line_after_doc_comments,
    unused_doc_comments,
    clippy::empty_docs,
    clippy::pedantic,
    clippy::all,
)]
#![forbid(
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    invalid_doc_attributes
)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

pub mod context;
pub mod describe;
pub mod flows;
mod future_utils;
pub mod node;
