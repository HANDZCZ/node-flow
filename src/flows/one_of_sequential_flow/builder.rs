use std::{marker::PhantomData, sync::Arc};

use crate::{
    flows::{ChainLink, NodeIOE, OneOfSequentialFlow},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct OneOfSequentialFlowBuilder<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    _ioe: PhantomData<(Input, Output, Error)>,
    _nodes_io: PhantomData<NodeIOETypes>,
    nodes: NodeTypes,
}

impl<Input, Output, Error> Default for OneOfSequentialFlowBuilder<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> OneOfSequentialFlowBuilder<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
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
    ) -> OneOfSequentialFlowBuilder<
        Input,
        Output,
        Error,
        (NodeType,),
        ChainLink<(), NodeIOE<NodeInput, NodeOutput, NodeError>>,
    >
    where
        Input: Into<NodeInput>,
        NodeOutput: Into<Output>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
    {
        OneOfSequentialFlowBuilder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (node,),
        }
    }
}

impl<Input, Output, Error, NodeTypes, LastNodeIOETypes, OtherNodeIOETypes>
    OneOfSequentialFlowBuilder<
        Input,
        Output,
        Error,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
    >
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    #[allow(clippy::type_complexity)]
    pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
        self,
        node: NodeType,
    ) -> OneOfSequentialFlowBuilder<
        Input,
        Output,
        Error,
        ChainLink<NodeTypes, NodeType>,
        ChainLink<
            ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            NodeIOE<NodeInput, NodeOutput, NodeError>,
        >,
    >
    where
        Input: Into<NodeInput>,
        NodeOutput: Into<Output>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send + Sync,
    {
        OneOfSequentialFlowBuilder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (self.nodes, node),
        }
    }

    pub fn build(
        self,
    ) -> OneOfSequentialFlow<
        Input,
        Output,
        Error,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
    > {
        OneOfSequentialFlow {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
        }
    }
}
