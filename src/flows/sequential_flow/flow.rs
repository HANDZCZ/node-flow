use crate::{
    flows::{NodeResult, SequentialFlowBuilder, sequential_flow::chain_run::ChainRunSequential},
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};
use std::{marker::PhantomData, sync::Arc};

pub struct SequentialFlow<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()> {
    pub(super) _ioe: PhantomData<(Input, Output, Error)>,
    pub(super) _nodes_io: PhantomData<NodeIOETypes>,
    pub(super) nodes: Arc<NodeTypes>,
}

impl<Input, Output, Error> SequentialFlow<Input, Output, Error> {
    #[must_use]
    pub fn builder() -> SequentialFlowBuilder<Input, Output, Error> {
        SequentialFlowBuilder::new()
    }
}

impl<Input, Output, Error, NodeTypes, NodeIOETypes> Node<Input, NodeOutputStruct<Output>, Error>
    for SequentialFlow<Input, Output, Error, NodeTypes, NodeIOETypes>
where
    NodeTypes: ChainRunSequential<Input, NodeResult<Output, Error>, NodeIOETypes>,
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
        flows::{SequentialFlow, sequential_flow::chain_run::ChainRunSequential},
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

    #[test]
    fn test_flow() {
        let mut st = Storage::new();
        let mut flow = SequentialFlow::<bool, u128, ()>::builder()
            .add_node(Passer::<u8, u16, ()>::new())
            .add_node(Passer::<u32, u64, ()>::new())
            .build();
        let fut = flow.run_with_storage(true, &mut st);

        let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
        let res = Future::poll(Box::pin(fut).as_mut(), &mut ctx);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(1))));
    }

    #[test]
    fn test_chain() {
        let mut st = Storage::new();
        let node = (
            (
                (Passer::<bool, u8, ()>::new(),),
                Passer::<u16, u32, ()>::new(),
            ),
            Passer::<u64, u128, ()>::new(),
        );
        let mut fut: Pin<Box<dyn Future<Output = Result<NodeOutput<u128>, ()>>>> =
            Box::pin(node.run_with_storage(true, &mut st));
        let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
        let res = Future::poll(fut.as_mut(), &mut ctx);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(1))));
    }
}
