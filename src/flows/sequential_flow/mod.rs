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
    /// Docs :)
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
