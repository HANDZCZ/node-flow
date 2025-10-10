use std::{marker::PhantomData, sync::Arc};

use crate::{
    flows::{ChainLink, NodeIOE, SequentialFlow},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct SequentialFlowBuilder<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()> {
    _ioe: PhantomData<(Input, Output, Error)>,
    _nodes_io: PhantomData<NodeIOETypes>,
    nodes: NodeTypes,
}

impl<Input, Output, Error> Default for SequentialFlowBuilder<Input, Output, Error> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> SequentialFlowBuilder<Input, Output, Error> {
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
    ) -> SequentialFlowBuilder<
        Input,
        Output,
        Error,
        (NodeType,),
        ChainLink<(), NodeIOE<NodeInput, NodeOutput, NodeError>>,
    >
    where
        Input: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError> + Clone,
    {
        SequentialFlowBuilder {
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
    SequentialFlowBuilder<
        Input,
        Output,
        Error,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
    >
{
    #[allow(clippy::type_complexity)]
    pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
        self,
        node: NodeType,
    ) -> SequentialFlowBuilder<
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
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError> + Clone,
    {
        SequentialFlowBuilder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (self.nodes, node),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
    ) -> SequentialFlow<
        Input,
        Output,
        Error,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, NodeIOE<LastNodeInType, LastNodeOutType, LastNodeErrType>>,
    >
    where
        LastNodeOutType: Into<Output>,
    {
        SequentialFlow {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
        }
    }
}
