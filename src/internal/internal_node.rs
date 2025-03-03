use std::{any::Any, marker::PhantomData};

use async_trait::async_trait;

use crate::{
    node::{Node, NodeOutput},
    storage::Storage,
};

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
pub(crate) trait InternalNode<Error>:
    Node<Box<dyn Any + Send>, NodeOutput<Box<dyn Any + Send>>, Error> + Send
where
    Error: Send,
{
    #[cfg(not(all(doc, not(doctest))))]
    fn duplicate(&self) -> Box<dyn InternalNode<Error>>;
}

pub(crate) struct InternalNodeStruct<Input, Output, Error, NodeType, PreviousNodeOutputType> {
    _node_types: PhantomData<(Input, Output, Error)>,
    _previous_node_output_type: PhantomData<PreviousNodeOutputType>,
    node: NodeType,
}

impl<NodeType, PreviousNodeOutputType, Input, Output, Error>
    InternalNodeStruct<Input, Output, Error, NodeType, PreviousNodeOutputType>
where
    PreviousNodeOutputType: Into<Input> + 'static,
{
    pub fn new(node: NodeType) -> Self {
        Self {
            _node_types: PhantomData,
            _previous_node_output_type: PhantomData,
            node,
        }
    }

    #[inline]
    fn get_input(input: Box<dyn Any + Send>) -> Option<Input> {
        let input = input.downcast::<PreviousNodeOutputType>().ok()?;
        Some((*input).into())
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<NodeType, Error, PreviousNodeOutputType, Input, Output, NodeError>
    Node<Box<dyn Any + Send>, NodeOutput<Box<dyn Any + Send>>, Error>
    for InternalNodeStruct<Input, Output, NodeError, NodeType, PreviousNodeOutputType>
where
    Input: Send,
    Output: Send + 'static,
    Error: Send,
    NodeError: Send,
    NodeType: Node<Input, NodeOutput<Output>, NodeError> + Send,
    NodeError: Into<Error>,
    PreviousNodeOutputType: Into<Input> + Send + 'static,
{
    async fn run_with_storage<'input>(
        &mut self,
        input: Box<dyn Any + Send>,
        storage: &mut Storage,
    ) -> Result<NodeOutput<Box<dyn Any + Send>>, Error> {
        let Some(input) = Self::get_input(input) else {
            unreachable!(
                "Type safety for the win!\n\tIf you reach this something went seriously wrong."
            );
        };
        let output = self
            .node
            .run_with_storage(input, storage)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutput::SoftFail => NodeOutput::SoftFail,
            NodeOutput::Ok(output) => NodeOutput::Ok(Box::new(output)),
        })
    }
}

impl<Input, Output, NodeError, Error, NodeType, PreviousNodeOutputType> InternalNode<Error>
    for InternalNodeStruct<Input, Output, NodeError, NodeType, PreviousNodeOutputType>
where
    NodeType: Node<Input, NodeOutput<Output>, NodeError> + Send + Clone + 'static,
    NodeError: Into<Error>,
    Error: Send,
    Input: Send + 'static,
    Output: Send + 'static,
    NodeError: Send + 'static,
    PreviousNodeOutputType: Into<Input> + Send + 'static,
{
    #[cfg(not(all(doc, not(doctest))))]
    fn duplicate(&self) -> Box<dyn InternalNode<Error>> {
        Box::new(Self {
            _node_types: PhantomData,
            _previous_node_output_type: PhantomData,
            node: self.node.clone(),
        })
    }
}
