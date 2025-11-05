//! This module contains all the different types of flows.
//!
//! For details on specific behavior, see the documentation of each flow.

mod chain_debug;
mod chain_describe;
mod generic_defs;

/// This module contains everything needed for constructing [`SequentialFlow`].
///
/// For detailed behavior and examples, see the documentation of [`SequentialFlow`] and [`Builder`](sequential_flow::Builder).
pub mod sequential_flow;
pub use sequential_flow::SequentialFlow;

/// This module contains everything needed for constructing [`OneOfSequentialFlow`].
///
/// For detailed behavior and examples, see the documentation of [`OneOfSequentialFlow`] and [`Builder`](one_of_sequential_flow::Builder).
pub mod one_of_sequential_flow;
pub use one_of_sequential_flow::OneOfSequentialFlow;

/// This module contains everything needed for constructing [`OneOfParallelFlow`].
///
/// For detailed behavior and examples, see the documentation of [`OneOfParallelFlow`] and [`Builder`](one_of_parallel_flow::Builder).
pub mod one_of_parallel_flow;
pub use one_of_parallel_flow::OneOfParallelFlow;

/// This module contains everything needed for constructing [`ParallelFlow`].
///
/// For detailed behavior and examples, see the documentation of [`ParallelFlow`], [`Builder`](parallel_flow::Builder) and [`Joiner`](parallel_flow::Joiner).
pub mod parallel_flow;
pub use parallel_flow::ParallelFlow;

/// This module contains everything needed for constructing [`FnFlow`].
///
/// For detailed behavior and examples, see the documentation of [`FnFlow`] and [`Runner`](fn_flow::Runner).
pub mod fn_flow;
pub use fn_flow::FnFlow;

/// This module contains everything needed for constructing [`Detached`].
///
/// For detailed behavior and examples, see the documentation of [`Detached`].
pub mod detached;
pub use detached::Detached;

use crate::node::NodeOutput;
type NodeIOE<Input, Output, Error> = (Input, NodeOutput<Output>, Error);
type ChainLink<Head, Tail> = (Head, Tail);
type NodeResult<Output, Error> = Result<NodeOutput<Output>, Error>;

#[cfg(test)]
#[doc(hidden)]
pub mod tests;
