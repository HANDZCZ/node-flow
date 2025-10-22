use std::{marker::PhantomData, sync::Arc};

use super::ParallelFlow as Flow;
use crate::{
    flows::{
        ChainLink, NodeIOE,
        parallel_flow::{Joiner, chain_run::ChainRunParallel as ChainRun},
    },
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct Builder<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
{
    _ioec: PhantomData<(Input, Output, Error, Context)>,
    _nodes_io: PhantomData<NodeIOETypes>,
    nodes: NodeTypes,
}

impl<Input, Output, Error, Context> Default for Builder<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error, Context> Builder<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
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
        NodeType: Send + Sync + Clone,
        NodeOutput: Send,
    {
        Builder {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (node,),
        }
    }
}

impl<Input, Output, Error, Context, NodeTypes, OtherNodeIOETypes, LastNodeIOETypes>
    Builder<
        Input,
        Output,
        Error,
        Context,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
    >
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
    Context: Send,
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
            ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            NodeIOE<NodeInput, NodeOutput, NodeError>,
        >,
    >
    where
        Input: Into<NodeInput>,
        NodeError: Into<Error>,
        NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError, Context>,
        // Trait bounds for better and nicer errors
        NodeType: Send + Sync + Clone,
        NodeOutput: Send,
    {
        Builder {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (self.nodes, node),
        }
    }

    // TODO: mention signature issue in docs (&mut Context must be present and it needs to be async closure: async |_, _: &mut Context| {...})
    #[allow(clippy::type_complexity)]
    pub fn build<J, ChainRunOutput>(
        self,
        joiner: J,
    ) -> Flow<
        Input,
        Output,
        Error,
        Context,
        ChainRunOutput,
        J,
        NodeTypes,
        ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
    >
    where
        for<'a> J: Joiner<'a, ChainRunOutput, Output, Error, Context>,
        NodeTypes: ChainRun<
                Input,
                Result<ChainRunOutput, Error>,
                Context,
                ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            >,
    {
        Flow {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: Arc::new(self.nodes),
            _joiner_input: PhantomData,
            joiner,
        }
    }
}
