use std::{future::poll_fn, pin::pin};

use crate::{
    context::{Fork, Join},
    flows::parallel_flow::chain_run::{poll::ChainPollParallel, spawn::ChainSpawn},
};

pub trait ChainRunParallel<Input, Output, Context, T> {
    fn run(&self, input: Input, context: &mut Context) -> impl Future<Output = Output> + Send;
}

impl<Input, Output, Error, Context, T, U> ChainRunParallel<Input, Result<Output, Error>, Context, T>
    for U
where
    U: ChainSpawn<Input, Error, Context, Output, T, ChainOut = Result<Output, Error>> + Sync,
    Input: Send,
    Context: Fork + Join + Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> Result<Output, Error> {
        let fut_chain = self.spawn(input, context.fork());
        let mut context_acc = Vec::with_capacity(U::NUM_FUTURES);
        let mut fut_chain = pin!(fut_chain);
        let res =
            poll_fn(|cx| ChainPollParallel::poll(fut_chain.as_mut(), cx, true, &mut context_acc))
                .await;
        context.join(context_acc.into_boxed_slice());
        res
    }
}
