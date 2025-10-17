use std::{marker::PhantomData, sync::Arc};

use super::ParallelFlow as Flow;
use crate::{
    flows::{
        ChainLink, NodeIOE,
        parallel_flow::{Joiner, chain_run::ChainRunParallel as ChainRun},
    },
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct Builder<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    _ioe: PhantomData<(Input, Output, Error)>,
    _nodes_io: PhantomData<NodeIOETypes>,
    nodes: NodeTypes,
}

impl<Input, Output, Error> Default for Builder<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> Builder<Input, Output, Error>
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
        NodeType: Send + Sync + Clone,
    {
        Builder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (node,),
        }
    }
}

impl<Input, Output, Error, NodeTypes, OtherNodeIOETypes, LastNodeIOETypes>
    Builder<Input, Output, Error, NodeTypes, ChainLink<OtherNodeIOETypes, LastNodeIOETypes>>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
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
            ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            NodeIOE<NodeInput, NodeOutput, NodeError>,
        >,
    >
    where
        Input: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError>,
        // Trait bounds for better and nicer errors
        NodeType: Send + Sync + Clone,
    {
        Builder {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: (self.nodes, node),
        }
    }

    // TODO: mention signature issue in docs (&mut Storage must be present and it needs to be async closure: async |_, _: &mut Storage| {...})
    #[allow(clippy::type_complexity)]
    pub fn build<J, ChainRunOutput>(
        self,
        joiner: J,
    ) -> Flow<
        Input,
        Output,
        Error,
        ChainRunOutput,
        J,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
    >
    where
        for<'a> J: Joiner<'a, ChainRunOutput, Output, Error>,
        NodeTypes: ChainRun<
                Input,
                Result<ChainRunOutput, Error>,
                ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            >,
    {
        Flow {
            _ioe: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
            _joiner_input: PhantomData,
            joiner,
        }
    }
}
