use super::Builder;
use super::chain_run::ChainRunParallel as ChainRun;
use crate::{
    flows::{NodeResult, parallel_flow::Joiner},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct ParallelFlow<
    Input,
    Output,
    Error,
    Context,
    ChainOutput = (),
    Joiner = (),
    NodeTypes = (),
    NodeIOETypes = (),
> {
    pub(super) _ioec: std::marker::PhantomData<(Input, Output, Error, Context)>,
    pub(super) _nodes_io: std::marker::PhantomData<NodeIOETypes>,
    pub(super) nodes: std::sync::Arc<NodeTypes>,
    pub(super) _joiner_input: std::marker::PhantomData<ChainOutput>,
    pub(super) joiner: Joiner,
}

impl<Input, Output, Error, Context> ParallelFlow<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send + Clone,
    Error: Send,
{
    #[must_use]
    pub fn builder() -> Builder<Input, Output, Error, Context> {
        Builder::new()
    }
}

impl<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes> Clone
    for ParallelFlow<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes>
where
    J: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _ioec: std::marker::PhantomData,
            _nodes_io: std::marker::PhantomData,
            nodes: self.nodes.clone(),
            _joiner_input: std::marker::PhantomData,
            joiner: self.joiner.clone(),
        }
    }
}

// workaround for https://github.com/rust-lang/rust/issues/100013
#[inline(always)]
#[allow(clippy::inline_always)]
fn call_joiner<'a, J, I, O, E, Ctx>(
    j: &J,
    i: I,
    s: &'a mut Ctx,
) -> impl Future<Output = NodeResult<O, E>>
where
    J: Joiner<'a, I, O, E, Ctx> + 'a,
{
    j.join(i, s)
}

impl<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes>
    Node<Input, NodeOutputStruct<Output>, Error, Context>
    for ParallelFlow<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes>
where
    Input: Send,
    Context: Send,
    for<'a> J: Joiner<'a, ChainRunOutput, Output, Error, Context>,
    NodeTypes: ChainRun<Input, Result<ChainRunOutput, Error>, Context, NodeIOETypes> + Send + Sync,
{
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
    ) -> impl Future<Output = NodeResult<Output, Error>> + Send {
        let nodes = self.nodes.as_ref();
        let joiner = &self.joiner;
        async move {
            let fut = nodes.run(input, context);
            let res = fut.await?;
            // workaround for https://github.com/rust-lang/rust/issues/100013
            call_joiner::<J, ChainRunOutput, Output, Error, Context>(joiner, res, context).await
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ChainRun, ParallelFlow as Flow};
    use crate::{
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode},
        node::{Node, NodeOutput},
        storage::{Storage, tests::MyVal},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(Passer::<u16, u64, ()>::new())
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build(async |input, context: &mut Storage| {
                context.insert(MyVal::default());
                assert_eq!(
                    input,
                    (
                        ((NodeOutput::Ok(0u64),), NodeOutput::SoftFail),
                        NodeOutput::Ok(0u32)
                    )
                );
                Ok(NodeOutput::Ok(120))
            });
        let res = flow.run(0, &mut st).await;

        assert_eq!(res, Result::Ok(NodeOutput::Ok(120)));
    }

    #[tokio::test]
    async fn test_chain() {
        let mut st = Storage::new();
        let node = (
            (
                (Passer::<u16, u64, ()>::new(),),
                SoftFailNode::<u16, u32, ()>::new(),
            ),
            Passer::<u16, u32, ()>::new(),
        );
        let res: Result<_, ()> = ChainRun::<u8, _, _, _>::run(&node, 0u8, &mut st).await;
        assert_eq!(
            res,
            Ok((
                ((NodeOutput::Ok(0u64),), NodeOutput::SoftFail),
                NodeOutput::Ok(0u32)
            ))
        );
    }

    #[tokio::test]
    async fn test_flow_storage() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), MyVal>::new())
            .add_node(Passer::<u16, u64, ()>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), MyVal>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), MyVal>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build(async |input, context: &mut Storage| {
                let merged_orig = context.insert(MyVal::default());
                assert_eq!(merged_orig, Some(MyVal("|||".to_owned())));
                assert_eq!(
                    input,
                    (
                        (
                            (
                                ((NodeOutput::SoftFail,), NodeOutput::Ok(5u64)),
                                NodeOutput::SoftFail
                            ),
                            NodeOutput::SoftFail
                        ),
                        NodeOutput::Ok(5u32)
                    )
                );
                Ok(NodeOutput::Ok(120))
            });

        let res = flow.run(5, &mut st).await;
        assert_eq!(res, Result::Ok(NodeOutput::Ok(120)));

        assert_eq!(st.remove::<MyVal>(), Some(MyVal::default()));
    }
}
