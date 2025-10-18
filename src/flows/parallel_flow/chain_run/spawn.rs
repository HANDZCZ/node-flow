use crate::{
    flows::{ChainLink, NodeIOE, parallel_flow::chain_run::poll::ChainPollParallel},
    future_utils::MaybeReady,
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};

pub trait ChainSpawn<Input, Error, HeadOut, T> {
    type ChainOut;

    fn spawn_with_storage(
        &self,
        input: Input,
        storage: Storage,
    ) -> impl ChainPollParallel<Self::ChainOut>;
}

impl<
    Input,
    Error,
    HeadIOETypes,
    TailNodeInType,
    TailNodeOutType,
    TailNodeErrType,
    HeadOut,
    Head,
    Tail,
>
    ChainSpawn<
        Input,
        Error,
        (HeadOut, NodeOutputStruct<TailNodeOutType>),
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainSpawn<Input, Error, HeadOut, HeadIOETypes, ChainOut = Result<HeadOut, Error>> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    Input: Into<TailNodeInType> + Clone + Send,
    TailNodeOutType: Send,
    Error: Send,
{
    type ChainOut = Result<(HeadOut, NodeOutputStruct<TailNodeOutType>), Error>;

    fn spawn_with_storage(
        &self,
        input: Input,
        storage: Storage,
    ) -> impl ChainPollParallel<Self::ChainOut> {
        let (head, tail) = self;
        let mut new_storage = storage.new_gen();

        let head_res = head.spawn_with_storage(input.clone(), storage);

        let mut tail = tail.clone();
        let tail_fut = async move {
            let output = tail
                .run_with_storage(input.into(), &mut new_storage)
                .await
                .map_err(Into::into)?;
            Ok((output, new_storage))
        };
        (head_res, MaybeReady::Pending(tail_fut))
    }
}

impl<Input, Error, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainSpawn<
        Input,
        Error,
        (NodeOutputStruct<HeadNodeOutType>,),
        ChainLink<(), NodeIOE<HeadNodeInType, HeadNodeOutType, HeadNodeErrType>>,
    > for (Head,)
where
    Input: Into<HeadNodeInType> + Send,
    Head: Node<HeadNodeInType, NodeOutputStruct<HeadNodeOutType>, HeadNodeErrType>
        + Clone
        + Send
        + Sync,
    HeadNodeErrType: Into<Error>,
    HeadNodeOutType: Send,
    Error: Send,
{
    type ChainOut = Result<(NodeOutputStruct<HeadNodeOutType>,), Error>;

    fn spawn_with_storage(
        &self,
        input: Input,
        mut storage: Storage,
    ) -> impl ChainPollParallel<Self::ChainOut> {
        let mut head = self.0.clone();
        let fut = async move {
            let output = head
                .run_with_storage(input.into(), &mut storage)
                .await
                .map_err(Into::into)?;
            Ok((output, storage))
        };
        (MaybeReady::Pending(fut),)
    }
}
