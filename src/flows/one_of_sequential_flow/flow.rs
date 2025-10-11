use crate::{
    flows::{
        NodeResult, OneOfSequentialFlowBuilder,
        one_of_sequential_flow::chain_run::ChainRunOneOfSequential,
    },
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};
use std::{marker::PhantomData, sync::Arc};

pub struct OneOfSequentialFlow<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()> {
    pub(super) _ioe: PhantomData<(Input, Output, Error)>,
    pub(super) _nodes_io: PhantomData<NodeIOETypes>,
    pub(super) nodes: Arc<NodeTypes>,
}

impl<Input, Output, Error> OneOfSequentialFlow<Input, Output, Error> {
    #[must_use]
    pub fn builder() -> OneOfSequentialFlowBuilder<Input, Output, Error> {
        OneOfSequentialFlowBuilder::new()
    }
}

impl<Input, Output, Error, NodeTypes, NodeIOETypes> Node<Input, NodeOutputStruct<Output>, Error>
    for OneOfSequentialFlow<Input, Output, Error, NodeTypes, NodeIOETypes>
where
    NodeTypes: ChainRunOneOfSequential<Input, NodeResult<Output, Error>, NodeIOETypes>,
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
    use std::{marker::PhantomData, pin::Pin, task::Poll};

    use crate::{
        flows::{OneOfSequentialFlow, one_of_sequential_flow::chain_run::ChainRunOneOfSequential},
        node::{Node, NodeOutput},
        storage::Storage,
    };

    #[derive(Clone)]
    struct Passer<I, O, E>(PhantomData<(I, O, E)>);

    impl<I, O, E> Passer<I, O, E> {
        fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<I, O, E> Node<I, NodeOutput<O>, E> for Passer<I, O, E>
    where
        I: Into<O> + Send,
        O: Send,
        E: Send,
    {
        async fn run_with_storage(
            &mut self,
            input: I,
            _storage: &mut Storage,
        ) -> Result<NodeOutput<O>, E> {
            Ok(NodeOutput::Ok(input.into()))
        }
    }

    #[derive(Clone)]
    struct SoftFailNode<I, O, E>(PhantomData<(I, O, E)>);

    impl<I, O, E> SoftFailNode<I, O, E> {
        fn new() -> Self {
            Self(PhantomData)
        }
    }
    impl<I, O, E> Node<I, NodeOutput<O>, E> for SoftFailNode<I, O, E>
    where
        I: Into<O> + Send,
        O: Send,
        E: Send,
    {
        async fn run_with_storage(
            &mut self,
            _input: I,
            _storage: &mut Storage,
        ) -> Result<NodeOutput<O>, E> {
            Ok(NodeOutput::SoftFail)
        }
    }

    #[test]
    fn test_flow() {
        let mut st = Storage::new();
        let mut flow = OneOfSequentialFlow::<u8, u64, ()>::builder()
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(SoftFailNode::<u8, u16, ()>::new())
            .add_node(SoftFailNode::<u32, u64, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let fut = flow.run_with_storage(5, &mut st);

        let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
        let res = Future::poll(Box::pin(fut).as_mut(), &mut ctx);
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
        let mut fut: Pin<Box<dyn Future<Output = Result<NodeOutput<u64>, ()>>>> =
            Box::pin(node.run_with_storage(5u8, &mut st));
        let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
        let res = Future::poll(fut.as_mut(), &mut ctx);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(5))));
    }

    #[derive(Clone)]
    struct InsertIntoStorageAssertWasNotInStorage<I, O, E, T>(PhantomData<(I, O, E, T)>);

    impl<I, O, E, T> InsertIntoStorageAssertWasNotInStorage<I, O, E, T> {
        fn new() -> Self {
            Self(PhantomData)
        }
    }
    impl<I, O, E, T> Node<I, NodeOutput<O>, E> for InsertIntoStorageAssertWasNotInStorage<I, O, E, T>
    where
        I: Into<O> + Send,
        O: Send,
        E: Send,
        T: Default + Clone + Send + 'static,
    {
        async fn run_with_storage(
            &mut self,
            _input: I,
            storage: &mut Storage,
        ) -> Result<NodeOutput<O>, E> {
            assert!(
                storage.insert(T::default()).is_none(),
                "{} was in storage",
                std::any::type_name::<T>()
            );
            Ok(NodeOutput::SoftFail)
        }
    }

    #[test]
    fn test_flow_storage() {
        let mut st = Storage::new();
        let mut flow = OneOfSequentialFlow::<u8, u64, ()>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), String>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let fut = flow.run_with_storage(5, &mut st);

        let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
        let res = Future::poll(Box::pin(fut).as_mut(), &mut ctx);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(5))));
    }
}
