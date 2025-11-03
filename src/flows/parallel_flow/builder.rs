use std::{marker::PhantomData, sync::Arc};

use super::ParallelFlow as Flow;
use crate::{
    context::{Fork, Join},
    flows::{
        ChainLink, NodeIOE,
        generic_defs::debug::impl_debug_for_builder,
        parallel_flow::{Joiner, chain_run::ChainRunParallel as ChainRun},
    },
    node::{Node, NodeOutput as NodeOutputStruct},
};

/// Builder for [`ParallelFlow`](Flow).
///
/// This builder ensures:
/// - `Input` into the flow can be converted into the input of all nodes
/// - error of all nodes can be converted into the `Error` of the flow
/// - `Joiner` returns `Result<Output, Error>`
///
/// See also [`ParallelFlow`](Flow).
pub struct Builder<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
    Context: Fork + Join + Send,
{
    #[expect(clippy::type_complexity)]
    _ioec: PhantomData<fn() -> (Input, Output, Error, Context)>,
    _nodes_io: PhantomData<fn() -> NodeIOETypes>,
    nodes: NodeTypes,
}

impl_debug_for_builder!(
    "ParallelFlow",
    Builder,
    Input: Send + Clone,
    Error: Send,
    Context: Fork + Join + Send
);

impl<Input, Output, Error, Context> Default for Builder<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
    Context: Fork + Join + Send,
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
    Context: Fork + Join + Send,
{
    /// Creates a new empty builder for [`ParallelFlow`](Flow).
    #[must_use]
    pub fn new() -> Self {
        Self {
            _ioec: PhantomData,
            _nodes_io: PhantomData,
            nodes: (),
        }
    }

    /// Adds a new node.
    ///
    /// The new node must satisfy:
    /// - `Self`: `Node<NodeInputType, NodeOutput<NodeOutputType>, NodeErrorType, _>`
    /// - `Input`: `Into<NodeInputType>`,
    /// - `NodeErrorType`: `Into<Error>`,
    ///
    /// # Returns
    /// A new [`Builder`] with the added node.
    #[expect(clippy::type_complexity, clippy::type_repetition_in_bounds)]
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
    Context: Fork + Join + Send,
{
    /// Adds a new node.
    ///
    /// The new node must satisfy:
    /// - `Self`: `Node<NodeInputType, NodeOutput<NodeOutputType>, NodeErrorType, _>`
    /// - `Input`: `Into<NodeInputType>`,
    /// - `NodeErrorType`: `Into<Error>`,
    ///
    /// # Returns
    /// A new [`Builder`] with the added node.
    #[expect(clippy::type_complexity, clippy::type_repetition_in_bounds)]
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

    /// Finalizes the builder and produces a [`ParallelFlow`](Flow) instance.
    ///
    /// The joiner must satisfy:
    /// - `Self`: `Joiner<'_, NodesOutputs, Output, Error, _>`
    ///
    /// When using closure as a joiner it always needs:
    /// - to be an **async closure** - because of lifetimes
    /// - *for context to*:
    ///     - have the **type of a context** specified when **using** context - because it cannot infer the type\
    ///       *or*
    ///     - have the context specified as `_: &mut _` when **not using** context - because it cannot satisfy that `Joiner` is implemented
    ///
    /// # Examples
    /// ```
    /// # use node_flow::node::{Node, NodeOutput};
    /// # use node_flow::flows::ParallelFlow;
    /// # use node_flow::context::{Fork, Join};
    /// # #[derive(Clone)]
    /// # struct A;
    /// # struct Context;
    /// # impl Fork for Context { fn fork(&self) -> Self { Self } }
    /// # impl Join for Context { fn join(&mut self, others: Box<[Self]>) {} }
    /// # impl<Ctx: Send> Node<(), NodeOutput<i32>, (), Ctx> for A {
    /// #     async fn run(&mut self, _: (), _: &mut Ctx) -> Result<NodeOutput<i32>, ()> { todo!() }
    /// # }
    /// # let flow = ParallelFlow::<(), i32, (), Context>::builder()
    /// #     .add_node(A)
    /// // ...
    /// .build(async |_, _: &mut _| {
    ///     Ok(NodeOutput::Ok(120))
    /// });
    /// # let flow = ParallelFlow::<(), i32, (), Context>::builder()
    /// #     .add_node(A)
    /// // ...
    /// .build(async |_, ctx: &mut Context| {
    ///     let _forked_ctx = ctx.fork();
    ///     Ok(NodeOutput::Ok(120))
    /// });
    /// ```
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
