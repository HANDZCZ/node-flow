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
    /// Docs :)
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
