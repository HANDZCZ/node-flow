mod builder;
pub use builder::*;
mod chain_run;

use crate::flows::generic_defs::flow::define_flow;
use chain_run::ChainRunSequential as ChainRun;

define_flow!(
    SequentialFlow,
    ChainRun,
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
        storage::Storage,
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = Storage::new();
        let mut flow = Flow::<bool, u128, ()>::builder()
            .add_node(Passer::<u8, u16, ()>::new())
            .add_node(Passer::<u32, u64, ()>::new())
            .build();
        let res = flow.run_with_storage(true, &mut st).await;

        assert_eq!(res, Ok(NodeOutput::Ok(1)));
    }

    #[tokio::test]
    async fn test_chain() {
        let mut st = Storage::new();
        let node = (
            (
                (Passer::<bool, u8, ()>::new(),),
                Passer::<u16, u32, ()>::new(),
            ),
            Passer::<u64, u128, ()>::new(),
        );
        let res =
            ChainRun::<_, Result<NodeOutput<u128>, ()>, _>::run_with_storage(&node, true, &mut st)
                .await;
        assert_eq!(res, Ok(NodeOutput::Ok(1)));
    }
}
