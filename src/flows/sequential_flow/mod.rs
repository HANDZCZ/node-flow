mod builder;
pub use builder::*;
mod chain_run;

use crate::{
    describe::{Description, Edge, remove_generics_from_name},
    flows::{chain_describe::ChainDescribe, generic_defs::flow::define_flow},
};
use chain_run::ChainRunSequential as ChainRun;

define_flow!(
    SequentialFlow,
    ChainRun,
    |self| {
        let node_count = <NodeTypes as ChainDescribe<Context, NodeIOETypes>>::COUNT;
        let mut node_descriptions = Vec::with_capacity(node_count);
        self.nodes.describe(&mut node_descriptions);

        let mut edges = Vec::with_capacity(node_count + 1);
        edges.push(Edge::flow_to_node(0));
        for i in 0..node_count - 1 {
            edges.push(Edge::node_to_node(i, i + 1));
        }
        edges.push(Edge::node_to_flow(node_count - 1));

        Description::new_flow(self, node_descriptions, edges).modify_name(remove_generics_from_name)
    },
    Input: Send,
    Error: Send,
    Context: Send,
    /// `SequentialFlow` executes nodes **sequentially**, like a pipeline.
    ///
    /// Nodes are executed sequentially like a pipeline where
    /// the output of one node is used as in input of the next node.
    /// Nodes are executed in order of insertion until **all** succeed or **any** node "hard" fails.
    ///
    /// - If a node returns [`NodeOutput::Ok`](crate::node::NodeOutput::Ok), that value is then fed into the next node.
    /// - If a node returns [`NodeOutput::SoftFail`](crate::node::NodeOutput::SoftFail), the flow soft-fails.
    /// - If a node returns an **error**, then that error is returned.
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
    /// use node_flow::flows::SequentialFlow;
    ///
    /// // Example node
    /// #[derive(Clone)]
    /// struct AddOne;
    ///
    /// struct ExampleCtx;
    ///
    /// impl<Ctx: Send> Node<u8, NodeOutput<u8>, (), Ctx> for AddOne {
    ///     async fn run(&mut self, input: u8, _: &mut Ctx) -> Result<NodeOutput<u8>, ()> {
    ///         Ok(NodeOutput::Ok(input + 1))
    ///     }
    /// }
    ///
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// async fn main() {
    ///     let mut flow = SequentialFlow::<u8, u8, (), _>::builder()
    ///         .add_node(AddOne)
    ///         .add_node(AddOne)
    ///         .add_node(AddOne)
    ///         .build();
    ///
    ///     let mut ctx = ExampleCtx;
    ///     let result = flow.run(5u8, &mut ctx).await;
    ///     assert_eq!(result, Ok(NodeOutput::Ok(8)));
    /// }
    /// # main().await;
    /// # });
    /// ```
);

#[cfg(test)]
mod test {
    use super::{ChainRun, SequentialFlow as Flow};
    use crate::{
        flows::tests::Passer,
        node::{Node, NodeOutput},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut flow = Flow::<bool, u128, (), ()>::builder()
            .add_node(Passer::<u8, u16, ()>::new())
            .add_node(Passer::<u32, u64, ()>::new())
            .build();
        let res = flow.run(true, &mut ()).await;

        assert_eq!(res, Ok(NodeOutput::Ok(1)));
    }

    #[tokio::test]
    async fn test_chain() {
        let node = (
            (
                (Passer::<bool, u8, ()>::new(),),
                Passer::<u16, u32, ()>::new(),
            ),
            Passer::<u64, u128, ()>::new(),
        );
        let res =
            ChainRun::<_, Result<NodeOutput<u128>, ()>, (), _>::run(&node, true, &mut ()).await;
        assert_eq!(res, Ok(NodeOutput::Ok(1)));
    }
}
