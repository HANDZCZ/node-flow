use std::{future::poll_fn, pin::pin, task::Poll};

use crate::{
    flows::{
        NodeResult,
        one_of_parallel_flow::{
            FutOutput,
            chain_run::{poll::ChainPollOneOfParallel, spawn::ChainSpawn},
        },
    },
    future_utils::SoftFailPoll,
    node::NodeOutput as NodeOutputStruct,
    storage::Storage,
};

pub trait ChainRunOneOfParallel<Input, Output, T> {
    fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> impl Future<Output = Output> + Send;
}

impl<Input, Output, Error, T, U> ChainRunOneOfParallel<Input, NodeResult<Output, Error>, T> for U
where
    U: ChainSpawn<Input, NodeResult<Output, Error>, FutOutput<Output, Error>, T> + Sync,
    Input: Send,
{
    async fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> NodeResult<Output, Error> {
        let fut_chain = self.spawn_with_storage(input, storage.new_gen());
        let mut fut_chain = pin!(fut_chain);
        poll_fn(
            move |cx| match ChainPollOneOfParallel::poll(fut_chain.as_mut(), cx) {
                SoftFailPoll::Pending => Poll::Pending,
                SoftFailPoll::Ready(res) => {
                    let res = res.map(|(res, new_storage)| {
                        storage.replace(new_storage);
                        res
                    });
                    Poll::Ready(res)
                }
                SoftFailPoll::SoftFail => Poll::Ready(Ok(NodeOutputStruct::SoftFail)),
            },
        )
        .await
    }
}
