mod chain_run;

use crate::flows::generic_defs::define_flow_and_ioe_conv_builder;
use chain_run::ChainRunOneOfSequential as ChainRun;

define_flow_and_ioe_conv_builder!(
    OneOfSequentialFlow,
    ChainRun,
    >Input: Send + Clone,
    #NodeType: Send + Sync + Clone
    /// Docs :)
);

#[cfg(test)]
mod test {
    use super::{ChainRun, OneOfSequentialFlow as Flow};
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

        assert_eq!(res, Ok(NodeOutput::Ok(5)));
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

        assert_eq!(res, Ok(NodeOutput::Ok(5)));
    }
}
