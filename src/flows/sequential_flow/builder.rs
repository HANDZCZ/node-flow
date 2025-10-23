use std::{marker::PhantomData, sync::Arc};

use super::SequentialFlow as Flow;
use crate::{
    flows::{ChainLink, NodeIOE, generic_defs::debug::impl_debug_for_builder},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct Builder<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    #[allow(clippy::type_complexity)]
    _ioec: PhantomData<fn() -> (Input, Output, Error, Context)>,
    _nodes_io: PhantomData<fn() -> NodeIOETypes>,
    nodes: NodeTypes,
}

impl_debug_for_builder!(
    "SequentialFlow",
    Builder,
    Input: Send,
    Error: Send
);

impl<Input, Output, Error, Context> Default for Builder<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
    Context: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error, Context> Builder<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
        self,
        node: NodeType,
    ) -> Builder<
        Input,
        Output,
        Error,
        Context,
        (NodeType,),
        ChainLink<(), NodeIOE<NodeInput, NodeOutput, NodeError>>,
    >
    where
        Input: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError, Context>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
        NodeInput: Send,
    {
        Builder {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (node,),
        }
    }
}

impl<
    Input,
    Output,
    Error,
    Context,
    NodeTypes,
    LastNodeInType,
    LastNodeOutType,
    LastNodeErrType,
    OtherNodeIOETypes,
>
    Builder<
        Input,
        Output,
        Error,
        Context,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
    >
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    #[allow(clippy::type_complexity)]
    pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
        self,
        node: NodeType,
    ) -> Builder<
        Input,
        Output,
        Error,
        Context,
        ChainLink<NodeTypes, NodeType>,
        ChainLink<
            ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
            NodeIOE<NodeInput, NodeOutput, NodeError>,
        >,
    >
    where
        LastNodeOutType: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError, Context>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
        NodeInput: Send,
    {
        Builder {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (self.nodes, node),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
    ) -> Flow<
        Input,
        Output,
        Error,
        Context,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
    >
    where
        LastNodeOutType: Into<Output>,
    {
        Flow {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
        }
    }
}
