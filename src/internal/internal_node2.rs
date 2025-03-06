use std::marker::PhantomData;

use async_trait::async_trait;

use crate::{
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
pub(crate) trait InternalNode2<Input, Output, Error>:
    Node<Input, NodeOutputStruct<Output>, Error> + Send
{
    #[cfg(not(all(doc, not(doctest))))]
    fn duplicate(&self) -> Box<dyn InternalNode2<Input, Output, Error>>;
}

pub(crate) struct InternalNodeStruct2<
    NodeInput,
    NodeOutput,
    NodeError,
    NodeType,
    Input,
    Output,
    Error,
> {
    _in_types: PhantomData<(NodeInput, NodeOutput, NodeError)>,
    _out_types: PhantomData<(Input, Output, Error)>,
    node: NodeType,
}

impl<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
    InternalNodeStruct2<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
{
    pub fn new(node: NodeType) -> Self {
        Self {
            _in_types: PhantomData,
            _out_types: PhantomData,
            node,
        }
    }
}

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
impl<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
    Node<Input, NodeOutputStruct<Output>, Error>
    for InternalNodeStruct2<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
where
    NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError> + Send,
    Input: Into<NodeInput> + Send,
    NodeOutput: Into<Output> + Send,
    NodeError: Into<Error> + Send,
    NodeInput: Send,
    Output: Send,
    Error: Send,
{
    async fn run_with_storage(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> Result<NodeOutputStruct<Output>, Error> {
        let res = self
            .node
            .run_with_storage(input.into(), storage)
            .await
            .map_err(Into::into)?;
        let res = match res {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
        };
        Ok(res)
    }
}

impl<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
    InternalNode2<Input, Output, Error>
    for InternalNodeStruct2<NodeInput, NodeOutput, NodeError, NodeType, Input, Output, Error>
where
    NodeType: Node<NodeInput, NodeOutputStruct<NodeOutput>, NodeError> + Clone + Send + 'static,
    Input: Into<NodeInput> + Send + 'static,
    Output: Send + 'static,
    Error: Send + 'static,
    NodeInput: Send + 'static,
    NodeOutput: Into<Output> + Send + 'static,
    NodeError: Into<Error> + Send + 'static,
{
    #[cfg(not(all(doc, not(doctest))))]
    fn duplicate(&self) -> Box<dyn InternalNode2<Input, Output, Error>> {
        Box::new(Self {
            _in_types: PhantomData,
            _out_types: PhantomData,
            node: self.node.clone(),
        })
    }
}
