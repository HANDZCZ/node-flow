use crate::{
    flows::{
        ChainLink, NodeIOE, NodeResult,
        one_of_parallel_flow::{FutOutput, chain_run::poll::ChainPollOneOfParallel},
    },
    future_utils::MaybeReady,
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};

pub trait ChainSpawn<Input, Output, ChainOut, T> {
    fn spawn_with_storage(
        &self,
        input: Input,
        storage: Storage,
    ) -> impl ChainPollOneOfParallel<ChainOut>;
}

impl<
    Input,
    Output,
    Error,
    HeadIOETypes,
    TailNodeInType,
    TailNodeOutType,
    TailNodeErrType,
    Head,
    Tail,
>
    ChainSpawn<
        Input,
        NodeResult<Output, Error>,
        FutOutput<Output, Error>,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head:
        ChainSpawn<Input, NodeResult<Output, Error>, FutOutput<Output, Error>, HeadIOETypes> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Into<TailNodeInType> + Clone + Send,
{
    fn spawn_with_storage(
        &self,
        input: Input,
        storage: Storage,
    ) -> impl ChainPollOneOfParallel<FutOutput<Output, Error>> {
        let (head, tail) = self;
        let mut new_storage = storage.new_gen();

        let head_res = head.spawn_with_storage(input.clone(), storage);

        let mut tail = tail.clone();
        let tail_fut = async move {
            let output = tail
                .run_with_storage(input.into(), &mut new_storage)
                .await
                .map_err(Into::into)?;
            Ok((
                match output {
                    NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                    NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
                },
                new_storage,
            ))
        };
        (head_res, MaybeReady::Pending(tail_fut))
    }
}

impl<Input, Output, Error, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainSpawn<
        Input,
        NodeResult<Output, Error>,
        FutOutput<Output, Error>,
        ChainLink<(), NodeIOE<HeadNodeInType, HeadNodeOutType, HeadNodeErrType>>,
    > for (Head,)
where
    Input: Into<HeadNodeInType> + Send,
    Head: Node<HeadNodeInType, NodeOutputStruct<HeadNodeOutType>, HeadNodeErrType>
        + Clone
        + Send
        + Sync,
    HeadNodeErrType: Into<Error>,
    HeadNodeOutType: Into<Output>,
{
    fn spawn_with_storage(
        &self,
        input: Input,
        mut storage: Storage,
    ) -> impl ChainPollOneOfParallel<FutOutput<Output, Error>> {
        let mut head = self.0.clone();
        let fut = async move {
            let output = head
                .run_with_storage(input.into(), &mut storage)
                .await
                .map_err(Into::into)?;
            Ok((
                match output {
                    NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                    NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
                },
                storage,
            ))
        };
        (MaybeReady::Pending(fut),)
    }
}
