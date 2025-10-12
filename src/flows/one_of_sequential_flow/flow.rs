use super::{Builder, chain_run::ChainRunOneOfSequential as ChainRun};
use crate::{
    flows::NodeResult,
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};
use std::{marker::PhantomData, sync::Arc};

pub struct OneOfSequentialFlow<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()> {
    pub(super) _ioe: PhantomData<(Input, Output, Error)>,
    pub(super) _nodes_io: PhantomData<NodeIOETypes>,
    pub(super) nodes: Arc<NodeTypes>,
}

impl<Input, Output, Error> OneOfSequentialFlow<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
{
    #[must_use]
    pub fn builder() -> Builder<Input, Output, Error> {
        Builder::new()
    }
}

impl<Input, Output, Error, NodeTypes, NodeIOETypes> Node<Input, NodeOutputStruct<Output>, Error>
    for OneOfSequentialFlow<Input, Output, Error, NodeTypes, NodeIOETypes>
where
    NodeTypes: ChainRun<Input, NodeResult<Output, Error>, NodeIOETypes>,
{
    fn run_with_storage(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> impl Future<Output = NodeResult<Output, Error>> + Send {
        self.nodes.run_with_storage(input, storage)
    }
}

#[cfg(test)]
mod test {
    use std::task::Poll;

    use super::{ChainRun, OneOfSequentialFlow as Flow};
    use crate::{
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode, poll_once},
        node::{Node, NodeOutput},
        storage::Storage,
    };

    #[test]
    fn test_flow() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, ()>::builder()
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(SoftFailNode::<u8, u16, ()>::new())
            .add_node(SoftFailNode::<u32, u64, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let fut = flow.run_with_storage(5, &mut st);

        let res = poll_once(fut);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(5))));
    }

    #[test]
    fn test_chain() {
        let mut st = Storage::new();
        let node = (
            (
                (SoftFailNode::<u16, u32, ()>::new(),),
                SoftFailNode::<u16, u32, ()>::new(),
            ),
            Passer::<u16, u32, ()>::new(),
        );
        let fut =
            ChainRun::<_, Result<NodeOutput<u64>, ()>, _>::run_with_storage(&node, 5u8, &mut st);
        let res = poll_once(fut);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(5))));
    }

    #[test]
    fn test_flow_storage() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, ()>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), String>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let fut = flow.run_with_storage(5, &mut st);

        let res = poll_once(fut);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(5))));
    }
}
