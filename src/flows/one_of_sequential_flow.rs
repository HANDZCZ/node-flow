use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    internal::internal_node2::{InternalNode2, InternalNodeStruct2},
    node::{Node, NodeOutput},
    storage::Storage,
};

pub struct OneOfSequentialFlow<Input, Output, Error> {
    #[allow(clippy::type_complexity)]
    #[cfg(not(all(doc, not(doctest))))]
    nodes: Arc<Vec<Box<dyn InternalNode2<Input, Output, Error> + Sync>>>,
    #[cfg(all(doc, not(doctest)))]
    __: std::marker::PhantomData<(Input, Output, Error)>,
}

impl<Input, Output, Error> OneOfSequentialFlow<Input, Output, Error> {
    /// Creates builder for [`OneOfSequentialFlow`].
    #[must_use]
    pub fn builder() -> OneOfSequentialFlowBuilder<Input, Output, Error> {
        OneOfSequentialFlowBuilder::new()
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<Input, Output, Error> Node<Input, NodeOutput<Output>, Error>
    for OneOfSequentialFlow<Input, Output, Error>
where
    Input: Clone + Send,
{
    async fn run_with_storage(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> Result<NodeOutput<Output>, Error> {
        for mut node in self.nodes.iter().map(|node| node.duplicate()) {
            let res = node.run_with_storage(input.clone(), storage).await?;
            match res {
                NodeOutput::SoftFail => {}
                NodeOutput::Ok(output) => return Ok(NodeOutput::Ok(output)),
            }
        }
        Ok(NodeOutput::SoftFail)
    }
}

#[derive(Default)]
pub struct OneOfSequentialFlowBuilder<Input, Output, Error> {
    #[cfg(not(all(doc, not(doctest))))]
    nodes: Vec<Box<dyn InternalNode2<Input, Output, Error> + Sync>>,
    #[cfg(all(doc, not(doctest)))]
    __: std::marker::PhantomData<(Input, Output, Error)>,
}

impl<Input, Output, Error> OneOfSequentialFlowBuilder<Input, Output, Error> {
    /// Creates a new instance of [`OneOfSequentialFlowBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a node to the [`OneOfSequentialFlowBuilder`].
    pub fn add_node<NodeType, NodeInput, NodeOutput_, NodeError>(&mut self, node: NodeType)
    where
        NodeType:
            Node<NodeInput, NodeOutput<NodeOutput_>, NodeError> + Clone + Send + Sync + 'static,
        Input: Into<NodeInput> + Send + Sync + 'static,
        Output: Send + Sync + 'static,
        Error: Send + Sync + 'static,
        NodeInput: Send + Sync + 'static,
        NodeOutput_: Into<Output> + Send + Sync + 'static,
        NodeError: Into<Error> + Send + Sync + 'static,
    {
        self.nodes.push(Box::new(InternalNodeStruct2::new(node)));
    }

    #[must_use]
    pub fn build(self) -> OneOfSequentialFlow<Input, Output, Error> {
        OneOfSequentialFlow {
            nodes: Arc::new(self.nodes),
        }
    }
}
