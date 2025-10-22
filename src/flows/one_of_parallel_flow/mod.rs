mod chain_run;

use crate::{
    flows::generic_defs::define_flow_and_ioe_conv_builder, node::NodeOutput as NodeOutputStruct,
};
use chain_run::ChainRunOneOfParallel as ChainRun;

type FutOutput<Output, Error, Context> = Result<(NodeOutputStruct<Output>, Context), Error>;

define_flow_and_ioe_conv_builder!(
    OneOfParallelFlow,
    ChainRun,
    >Input: Send + Clone,
    >Output: Send,
    >Error: Send,
    #NodeType: Send + Sync + Clone
    /// Docs :)
);

#[cfg(test)]
mod test {
    use super::{ChainRun, OneOfParallelFlow as Flow};
    use crate::{
        flows::tests::{InsertIntoStorageAssertWasNotInStorage, Passer, SoftFailNode},
        node::{Node, NodeOutput},
        storage::{Storage, tests::MyVal},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = Storage::new();
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
        let mut st = Storage::new();
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
        let mut st = Storage::new();
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
