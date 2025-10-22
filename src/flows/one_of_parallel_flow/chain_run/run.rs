use std::{future::poll_fn, pin::pin, task::Poll};

use crate::{
    context::{Fork, Update},
    flows::{
        NodeResult,
        one_of_parallel_flow::{
            FutOutput,
            chain_run::{poll::ChainPollOneOfParallel, spawn::ChainSpawn},
        },
    },
    future_utils::SoftFailPoll,
    node::NodeOutput as NodeOutputStruct,
};

pub trait ChainRunOneOfParallel<Input, Output, Context, T> {
    fn run(&self, input: Input, context: &mut Context) -> impl Future<Output = Output> + Send;
}

impl<Input, Output, Error, Context, T, U>
    ChainRunOneOfParallel<Input, NodeResult<Output, Error>, Context, T> for U
where
    U: ChainSpawn<Input, NodeResult<Output, Error>, Context, FutOutput<Output, Error, Context>, T>
        + Sync,
    Input: Send,
    Context: Fork + Update + Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> NodeResult<Output, Error> {
        let fut_chain = self.run(input, context.fork());
        let mut fut_chain = pin!(fut_chain);
        poll_fn(
            move |cx| match ChainPollOneOfParallel::poll(fut_chain.as_mut(), cx) {
                SoftFailPoll::Pending => Poll::Pending,
                SoftFailPoll::Ready(res) => {
                    let res = res.map(|(res, new_context)| {
                        context.update_from(new_context);
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
