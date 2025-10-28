use std::fmt::Debug;

use super::Builder;
use super::chain_run::ChainRunParallel as ChainRun;
use crate::{
    describe::{Description, DescriptionBase, Edge, Type, remove_generics_from_name},
    flows::{
        NodeResult, chain_debug::ChainDebug, chain_describe::ChainDescribe, parallel_flow::Joiner,
    },
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
    #[expect(clippy::type_complexity)]
    pub(super) _ioec: std::marker::PhantomData<fn() -> (Input, Output, Error, Context)>,
    pub(super) _nodes_io: std::marker::PhantomData<fn() -> NodeIOETypes>,
    pub(super) nodes: std::sync::Arc<NodeTypes>,
    pub(super) _joiner_input: std::marker::PhantomData<fn() -> ChainOutput>,
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

impl<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes> Debug
    for ParallelFlow<Input, Output, Error, Context, ChainRunOutput, J, NodeTypes, NodeIOETypes>
where
    NodeTypes: ChainDebug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelFlow")
            .field("nodes", &self.nodes.as_list())
            .finish_non_exhaustive()
    }
}

// workaround for https://github.com/rust-lang/rust/issues/100013
#[inline(always)]
#[expect(clippy::inline_always)]
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
    NodeTypes: ChainRun<Input, Result<ChainRunOutput, Error>, Context, NodeIOETypes>
        + ChainDescribe<Context, NodeIOETypes>
        + Send
        + Sync,
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

    fn describe(&self) -> Description {
        let node_count = <NodeTypes as ChainDescribe<Context, NodeIOETypes>>::COUNT;
        let mut node_descriptions = Vec::with_capacity(node_count + 1);
        self.nodes.describe(&mut node_descriptions);

        node_descriptions.push(Description::Node {
            base: DescriptionBase {
                r#type: Type {
                    name: "Joiner".to_owned(),
                },
                input: Type {
                    name: String::new(),
                },
                output: Type {
                    name: String::new(),
                },
                error: Type {
                    name: String::new(),
                },
                context: Type {
                    name: String::new(),
                },
                description: None,
                externals: None,
            },
        });

        let mut edges = Vec::with_capacity(node_count * 2 + 1);
        for i in 0..node_count {
            edges.push(Edge::flow_to_node(i));
            edges.push(Edge::node_to_node(i, node_count));
        }
        edges.push(Edge::node_to_flow(node_count));

        Description::new_flow(self, node_descriptions, edges).modify_name(remove_generics_from_name)
    }
}

#[cfg(test)]
mod test {
    use super::{ChainRun, ParallelFlow as Flow};
    use crate::{
        context::storage::local_storage::{LocalStorage, LocalStorageImpl, tests::MyVal},
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode},
        node::{Node, NodeOutput},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = LocalStorageImpl::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(Passer::<u16, u64, ()>::new())
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build(async |input, context: &mut LocalStorageImpl| {
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
        let mut st = LocalStorageImpl::new();
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
        let mut st = LocalStorageImpl::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), MyVal>::new())
            .add_node(Passer::<u16, u64, ()>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), MyVal>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), MyVal>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build(async |input, context: &mut LocalStorageImpl| {
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
