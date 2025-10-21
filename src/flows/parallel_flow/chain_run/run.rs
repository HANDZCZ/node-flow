use std::{future::poll_fn, pin::pin};

use crate::{
    flows::parallel_flow::chain_run::{poll::ChainPollParallel, spawn::ChainSpawn},
    storage::Storage,
};

pub trait ChainRunParallel<Input, Output, T> {
    fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> impl Future<Output = Output> + Send;
}

impl<Input, Output, Error, T, U> ChainRunParallel<Input, Result<Output, Error>, T> for U
where
    U: ChainSpawn<Input, Error, Output, T, ChainOut = Result<Output, Error>> + Sync,
    Input: Send,
{
    async fn run_with_storage(&self, input: Input, storage: &mut Storage) -> Result<Output, Error> {
        let fut_chain = self.spawn_with_storage(input, storage.new_gen());
        let mut storage_acc = Vec::with_capacity(U::NUM_FUTURES);
        let mut fut_chain = pin!(fut_chain);
        let res =
            poll_fn(|cx| ChainPollParallel::poll(fut_chain.as_mut(), cx, true, &mut storage_acc))
                .await;
        storage.merge(&mut storage_acc);
        res
    }
}
