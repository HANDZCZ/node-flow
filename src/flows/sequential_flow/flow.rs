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

impl<Input, Output, Error> SequentialFlow<Input, Output, Error>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Error: Send,
{
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
    use std::task::Poll;

    use crate::{
        flows::{
            SequentialFlow,
            sequential_flow::chain_run::ChainRunSequential,
            tests::{Passer, poll_once},
        },
        node::{Node, NodeOutput},
        storage::Storage,
    };

    #[test]
    fn test_flow() {
        let mut st = Storage::new();
        let mut flow = SequentialFlow::<bool, u128, ()>::builder()
            .add_node(Passer::<u8, u16, ()>::new())
            .add_node(Passer::<u32, u64, ()>::new())
            .build();
        let fut = flow.run_with_storage(true, &mut st);

        let res = poll_once(fut);
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
        let fut = ChainRunSequential::<_, Result<NodeOutput<u128>, ()>, _>::run_with_storage(
            &node, true, &mut st,
        );
        let res = poll_once(fut);
        assert_eq!(res, Poll::Ready(Result::Ok(NodeOutput::Ok(1))));
    }
}
