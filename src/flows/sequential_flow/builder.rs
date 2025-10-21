use std::{marker::PhantomData, sync::Arc};

use super::SequentialFlow as Flow;
use crate::{
    flows::{ChainLink, NodeIOE},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct Builder<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    _ioe: PhantomData<(Input, Output, Error)>,
    _nodes_io: PhantomData<NodeIOETypes>,
    nodes: NodeTypes,
}

impl<Input, Output, Error> Default for Builder<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> Builder<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            _ioe: PhantomData,
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
        (NodeType,),
        ChainLink<(), NodeIOE<NodeInput, NodeOutput, NodeError>>,
    >
    where
        Input: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
        NodeInput: Send,
    {
        Builder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (node,),
        }
    }
}

impl<
    Input,
    Output,
    Error,
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
        ChainLink<NodeTypes, NodeType>,
        ChainLink<
            ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
            NodeIOE<NodeInput, NodeOutput, NodeError>,
        >,
    >
    where
        LastNodeOutType: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
        NodeInput: Send,
    {
        Builder {
            _ioe: PhantomData,
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
        NodeTypes,
        ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
    >
    where
        LastNodeOutType: Into<Output>,
    {
        Flow {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
        }
    }
}
