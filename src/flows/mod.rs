mod chain_debug;
mod chain_describe;
mod generic_defs;
pub mod sequential_flow;
pub use sequential_flow::SequentialFlow;
pub mod one_of_sequential_flow;
pub use one_of_sequential_flow::OneOfSequentialFlow;
pub mod one_of_parallel_flow;
pub use one_of_parallel_flow::OneOfParallelFlow;
pub mod parallel_flow;
pub use parallel_flow::ParallelFlow;

use crate::node::NodeOutput;
type NodeIOE<Input, Output, Error> = (Input, NodeOutput<Output>, Error);
type ChainLink<Head, Tail> = (Head, Tail);
type NodeResult<Output, Error> = Result<NodeOutput<Output>, Error>;

#[cfg(test)]
mod tests;
