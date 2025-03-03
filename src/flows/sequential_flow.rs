use std::{any::Any, marker::PhantomData, sync::Arc};

use async_trait::async_trait;

use crate::{
    internal::internal_node::{InternalNode, InternalNodeStruct},
    node::{Node, NodeOutput},
    storage::Storage,
};

pub struct SequentialFlow<Input, Output, Error> {
    _input: PhantomData<Input>,
    _output: PhantomData<Output>,
    last_node_output_converter: Arc<Box<dyn ConvertTo<Output>>>,
    #[cfg(not(all(doc, not(doctest))))]
    nodes: Arc<Vec<Box<dyn InternalNode<Error> + Sync>>>,
    #[cfg(all(doc, not(doctest)))]
    __: PhantomData<Error>,
}

impl<Input, Output, Error> Clone for SequentialFlow<Input, Output, Error> {
    fn clone(&self) -> Self {
        Self {
            _input: PhantomData,
            _output: PhantomData,
            last_node_output_converter: Arc::clone(&self.last_node_output_converter),
            nodes: Arc::clone(&self.nodes),
        }
    }
}

impl<Input, Output, Error> SequentialFlow<Input, Output, Error>
where
    Input: Send + 'static,
    Output: Send + 'static,
    Error: Send + 'static,
{
    /// Creates builder for [`SequentialFlow`].
    #[must_use]
    pub fn builder() -> SequentialFlowBuilder<Input, Output, Error, Input> {
        SequentialFlowBuilder::new()
    }
}

/// Builder for [`SequentialFlow`].
pub struct SequentialFlowBuilder<Input, Output, Error, NextNodeInput> {
    _input: PhantomData<Input>,
    _output: PhantomData<Output>,
    _next_node_input: PhantomData<NextNodeInput>,
    #[cfg(not(all(doc, not(doctest))))]
    nodes: Vec<Box<dyn InternalNode<Error> + Sync>>,
    #[cfg(all(doc, not(doctest)))]
    __: PhantomData<Error>,
}

#[allow(clippy::mismatching_type_param_order)]
impl<Input, Output, Error> SequentialFlowBuilder<Input, Output, Error, Input> {
    /// Creates new instance of [`SequentialFlowBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            nodes: Vec::new(),
        }
    }
}

#[allow(clippy::mismatching_type_param_order)]
impl<Input, Output, Error> Default for SequentialFlowBuilder<Input, Output, Error, Input> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error> Node<Input, NodeOutput<Output>, Error>
    for SequentialFlow<Input, Output, Error>
where
    Input: Send + 'static,
    Output: Send,
    Error: Send,
{
    async fn run_with_storage<'a>(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> Result<NodeOutput<Output>, Error> {
        let mut data: Box<dyn Any + Send> = Box::new(input);
        for mut node in self.nodes.iter().map(|node| node.duplicate()) {
            match node.run_with_storage(data, storage).await? {
                NodeOutput::Ok(output) => data = output,
                NodeOutput::SoftFail => return Ok(NodeOutput::SoftFail),
            }
        }
        let output = self
            .last_node_output_converter
            .convert(data)
            .expect("Converting data to sequence output type failed");
        return Ok(NodeOutput::Ok(output));
    }
}

impl<Input, Output, Error, LastNodeOutput>
    SequentialFlowBuilder<Input, Output, Error, LastNodeOutput>
where
    Input: Send + 'static,
    Output: Send + 'static,
    Error: Send + 'static,
{
    /// Adds node to the sequence.
    pub fn add_node<NodeType, NodeInput, NodeOutput_, NodeError>(
        mut self,
        node: NodeType,
    ) -> SequentialFlowBuilder<Input, Output, Error, NodeOutput_>
    where
        LastNodeOutput: Send + Sync + Into<NodeInput> + 'static,
        NodeInput: Send + Sync + 'static,
        NodeOutput_: Send + Sync + 'static,
        NodeError: Send + Sync + Into<Error> + 'static,
        NodeType:
            Node<NodeInput, NodeOutput<NodeOutput_>, NodeError> + Clone + Send + Sync + 'static,
    {
        self.nodes.push(Box::new(InternalNodeStruct::<
            NodeInput,
            NodeOutput_,
            NodeError,
            NodeType,
            LastNodeOutput,
        >::new(node)));
        SequentialFlowBuilder {
            _input: PhantomData,
            _output: PhantomData,
            _next_node_input: PhantomData,
            nodes: self.nodes,
        }
    }
}

impl<Input, Output, Error, LastNodeOutput>
    SequentialFlowBuilder<Input, Output, Error, LastNodeOutput>
where
    Output: Send + Sync + 'static,
    LastNodeOutput: Into<Output> + Send + Sync + 'static,
{
    /// Finalizes the sequence so nodes can't be added to it.
    #[must_use]
    pub fn build(self) -> SequentialFlow<Input, Output, Error> {
        SequentialFlow {
            _input: PhantomData,
            _output: PhantomData,
            last_node_output_converter: Arc::new(Box::new(DowncastConverter::<
                LastNodeOutput,
                Output,
            >::new())),
            nodes: Arc::new(self.nodes),
        }
    }
}

trait ConvertTo<T>: Send + Sync {
    fn convert(&self, data: Box<dyn Any>) -> Option<T>;
}

struct DowncastConverter<Input, Output>
where
    Input: Into<Output>,
{
    _node_output_type: PhantomData<Input>,
    _output_type: PhantomData<Output>,
}

impl<Input, Output> DowncastConverter<Input, Output>
where
    Input: Into<Output>,
{
    fn new() -> Self {
        Self {
            _node_output_type: PhantomData,
            _output_type: PhantomData,
        }
    }
}

impl<FromType, IntoType> ConvertTo<IntoType> for DowncastConverter<FromType, IntoType>
where
    FromType: Into<IntoType> + Send + Sync + 'static,
    IntoType: Send + Sync,
{
    fn convert(&self, data: Box<dyn Any>) -> Option<IntoType> {
        let box_from = data.downcast::<FromType>().ok()?;
        let from = *box_from;
        let into = from.into();
        Some(into)
    }
}
