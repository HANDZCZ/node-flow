mod sequential_flow;
pub use sequential_flow::*;
mod one_of_sequential_flow;
pub use one_of_sequential_flow::*;

use crate::node::NodeOutput;
type NodeIOE<Input, Output, Error> = (Input, NodeOutput<Output>, Error);
type ChainLink<Head, Tail> = (Head, Tail);
type NodeResult<Output, Error> = Result<NodeOutput<Output>, Error>;
