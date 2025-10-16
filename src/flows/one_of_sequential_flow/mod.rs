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
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode},
        node::{Node, NodeOutput},
        storage::Storage,
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, ()>::builder()
            .add_node(SoftFailNode::<u16, u32, ()>::new())
            .add_node(SoftFailNode::<u8, u16, ()>::new())
            .add_node(SoftFailNode::<u32, u64, ()>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let res = flow.run_with_storage(5, &mut st).await;

        assert_eq!(res, Ok(NodeOutput::Ok(5)));
    }

    #[tokio::test]
    async fn test_chain() {
        let mut st = Storage::new();
        let node = (
            (
                (SoftFailNode::<u16, u32, ()>::new(),),
                SoftFailNode::<u16, u32, ()>::new(),
            ),
            Passer::<u16, u32, ()>::new(),
        );
        let res =
            ChainRun::<_, Result<NodeOutput<u64>, ()>, _>::run_with_storage(&node, 5u8, &mut st)
                .await;
        assert_eq!(res, Ok(NodeOutput::Ok(5)));
    }

    #[tokio::test]
    async fn test_flow_storage() {
        let mut st = Storage::new();
        let mut flow = Flow::<u8, u64, ()>::builder()
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u16, u32, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u8, u16, (), String>::new())
            .add_node(InsertIntoStorageAssertWasNotInStorage::<u32, u64, (), String>::new())
            .add_node(Passer::<u16, u32, ()>::new())
            .build();
        let res = flow.run_with_storage(5, &mut st).await;

        assert_eq!(res, Ok(NodeOutput::Ok(5)));
    }
}
