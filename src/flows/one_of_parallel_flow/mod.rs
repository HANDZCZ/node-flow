mod chain_run;

use crate::{
    context::{Fork, Update},
    describe::{Description, Edge, remove_generics_from_name},
    flows::{chain_describe::ChainDescribe, generic_defs::define_flow_and_ioe_conv_builder},
    node::NodeOutput as NodeOutputStruct,
};
use chain_run::ChainRunOneOfParallel as ChainRun;

type FutOutput<Output, Error, Context> = Result<(NodeOutputStruct<Output>, Context), Error>;

define_flow_and_ioe_conv_builder!(
    OneOfParallelFlow,
    ChainRun,
    |self| {
        let node_count = <NodeTypes as ChainDescribe<Context, NodeIOETypes>>::COUNT;
        let mut node_descriptions = Vec::with_capacity(node_count);
        self.nodes.describe(&mut node_descriptions);
        let edges = (0..node_count)
            .flat_map(|i| [Edge::flow_to_node(i), Edge::node_to_flow(i)])
            .collect::<Vec<_>>();

        Description::new_flow(self, node_descriptions, edges).modify_name(remove_generics_from_name)
    },
    >Input: Send + Clone,
    >Output: Send,
    >Error: Send,
    >Context: Fork + Update + Send,
    #NodeType: Send + Sync + Clone
    /// `OneOfParallelFlow` executes nodes (branches) **in parallel**, returning when one succeeds or fails.
    ///
    /// Nodes (branches) are executed concurrently.
    /// The flow completes when **any** node succeeds or "hard" fails.
    /// - If a node returns [`NodeOutput::Ok`](crate::node::NodeOutput::Ok), that value is returned.
    /// - If a node returns [`NodeOutput::SoftFail`](crate::node::NodeOutput::SoftFail),
    ///   that result is ignored and the flow continues waiting for other nodes (branches).
    /// - If a node returns an **error**, then that error is returned.
    ///
    /// If all nodes (branches) soft-fail, the flow itself returns [`NodeOutput::SoftFail`](crate::node::NodeOutput::SoftFail).
    ///
    /// This flow allows defining race-style execution, where multiple branches are ran in parallel
    /// and the first one to complete determines the returned value.
    ///
    /// # Type Parameters
    /// - `Input`: The type of data accepted by this flow.
    /// - `Output`: The type of data produced by this flow.
    /// - `Error`: The type of error emitted by this flow.
    /// - `Context`: The type of context used during execution.
    ///
    /// # Examples
    /// ```
    /// use node_flow::node::{Node, NodeOutput};
    /// use node_flow::flows::OneOfParallelFlow;
    /// use node_flow::context::{Fork, Update};
    ///
    /// // Example nodes
    /// #[derive(Clone)]
    /// struct A;
    /// #[derive(Clone)]
    /// struct B;
    ///
    /// struct ExampleCtx;
    /// impl Fork for ExampleCtx // ...
    /// # { fn fork(&self) -> Self { Self } }
    /// impl Update for ExampleCtx // ...
    /// # { fn update_from(&mut self, other: Self) {} }
    ///
    /// impl<Ctx: Send> Node<(), NodeOutput<i32>, (), Ctx> for A {
    ///     async fn run(&mut self, _: (), _: &mut Ctx) -> Result<NodeOutput<i32>, ()> {
    ///         Ok(NodeOutput::SoftFail) // Ignored
    ///     }
    /// }
    ///
    /// impl<Ctx: Send> Node<(), NodeOutput<i32>, (), Ctx> for B {
    ///     async fn run(&mut self, _: (), _: &mut Ctx) -> Result<NodeOutput<i32>, ()> {
    ///         Ok(NodeOutput::Ok(5)) // Wins the race
    ///     }
    /// }
    ///
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// async fn main() {
    ///     let mut flow = OneOfParallelFlow::<(), i32, (), _>::builder()
    ///         .add_node(A)
    ///         .add_node(B)
    ///         .build();
    ///
    ///     let mut ctx = ExampleCtx;
    ///     let result = flow.run((), &mut ctx).await;
    ///     assert_eq!(result, Ok(NodeOutput::Ok(5)));
    /// }
    /// # main().await;
    /// # });
    /// ```
);

#[cfg(test)]
mod test {
    use super::{ChainRun, OneOfParallelFlow as Flow};
    use crate::{
        context::storage::local_storage::{LocalStorageImpl, tests::MyVal},
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode},
        node::{Node, NodeOutput},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = LocalStorageImpl::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(SoftFailNode::<u8, u16, ()>::new())
            .add_node(SoftFailNode::<u32, u64, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let res = flow.run(5, &mut st).await;

        assert_eq!(res, Result::Ok(NodeOutput::Ok(5)));
    }

    #[tokio::test]
    async fn test_chain() {
        let mut st = LocalStorageImpl::new();
        let node = (
            (
                (SoftFailNode::<u16, u32, ()>::new(),),
                SoftFailNode::<u16, u32, ()>::new(),
            ),
            Passer::<u16, u32, ()>::new(),
        );
        let res = ChainRun::<_, Result<NodeOutput<u64>, ()>, _, _>::run(&node, 5u8, &mut st).await;
        assert_eq!(res, Ok(NodeOutput::Ok(5)));
    }

    #[tokio::test]
    async fn test_flow_storage() {
        let mut st = LocalStorageImpl::new();
        let mut flow = Flow::<u8, u64, (), _>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), MyVal>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), MyVal>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), MyVal>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let res = flow.run(5, &mut st).await;

        assert_eq!(res, Result::Ok(NodeOutput::Ok(5)));
    }
}
