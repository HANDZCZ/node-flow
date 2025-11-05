#![warn(
    missing_docs,
    clippy::tabs_in_doc_comments,
    clippy::suspicious_doc_comments,
    clippy::test_attr_in_doctest,
    clippy::empty_line_after_doc_comments,
    unused_doc_comments,
    clippy::empty_docs,
    clippy::pedantic,
    clippy::all,
    clippy::nursery
)]
#![forbid(
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    invalid_doc_attributes
)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

//! # Node Flow
//!
//! **Node Flow** is runtime-agnostic, composable, asynchronous node-based framework for building
//! structured and reusable data processing pipelines, workflows, or control flows.
//!
//! The core idea is that each **node** represents a self-contained asynchronous operation,
//! and **flows** define how multiple nodes are composed and executed.
//!
//! ## Key concepts
//!
//! - **[`Node`](crate::node::Node)** - the core building block, representing an async unit of work.
//! - **[`NodeOutput`](crate::node::NodeOutput)** - output type used by nodes to signal success or soft failure.
//! - **[`Flows`](crate::flows)** - structures that determine execution order and behavior.
//! - **[`Context system`](crate::context)** - flows and nodes can restrict the context to ensure that it can perform functions such as:
//!     - sharing a mutable state
//!     - context branching/joining
//!     - task spawning
//! - **[`Description`](crate::describe::Description)** - describes the structure of a flow, which can then be used for visualization.
//!
//! ## Examples
//! ```
//! use node_flow::node::{Node, NodeOutput};
//! use node_flow::flows::SequentialFlow;
//!
//! // Example node
//! #[derive(Clone)]
//! struct AddOne;
//!
//! struct ExampleCtx;
//!
//! impl<Ctx: Send> Node<u8, NodeOutput<u8>, (), Ctx> for AddOne {
//!     async fn run(&mut self, input: u8, _: &mut Ctx) -> Result<NodeOutput<u8>, ()> {
//!         Ok(NodeOutput::Ok(input + 1))
//!     }
//! }
//!
//! # tokio::runtime::Builder::new_current_thread()
//! #     .enable_all()
//! #     .build()
//! #     .unwrap()
//! #     .block_on(async {
//! async fn main() {
//!     let mut flow = SequentialFlow::<u8, u8, (), _>::builder()
//!         .add_node(AddOne)
//!         .add_node(AddOne)
//!         .add_node(AddOne)
//!         .build();
//!
//!     let mut ctx = ExampleCtx;
//!     let result = flow.run(5u8, &mut ctx).await;
//!     assert_eq!(result, Ok(NodeOutput::Ok(8)));
//! }
//! # main().await;
//! # });
//! ```
//!
//! ## When to use Node Flow
//!
//! Use this crate when you need:
//! - Composable async control flows (e.g., fallback chains, parallel processing).
//! - Declarative and type-safe node composition.
//! - Inspectable or visualizable flow structures.

pub mod context;
pub mod describe;
pub mod flows;
mod future_utils;
pub mod node;
