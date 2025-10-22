use futures_util::future::MaybeDone;

use crate::{
    context::Fork,
    flows::{ChainLink, NodeIOE, parallel_flow::chain_run::poll::ChainPollParallel},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub trait ChainSpawn<Input, Error, Context, HeadOut, T> {
    type ChainOut;
    const NUM_FUTURES: usize;

    fn spawn(
        &self,
        input: Input,
        context: Context,
    ) -> impl ChainPollParallel<Self::ChainOut, Context>;
}

impl<
    Input,
    Error,
    Context,
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
        Context,
        (HeadOut, NodeOutputStruct<TailNodeOutType>),
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainSpawn<Input, Error, Context, HeadOut, HeadIOETypes, ChainOut = Result<HeadOut, Error>>
        + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    Input: Into<TailNodeInType> + Clone + Send,
    TailNodeOutType: Send,
    Error: Send,
    Context: Fork + Send,
{
    type ChainOut = Result<(HeadOut, NodeOutputStruct<TailNodeOutType>), Error>;
    const NUM_FUTURES: usize = Head::NUM_FUTURES + 1;

    fn spawn(
        &self,
        input: Input,
        context: Context,
    ) -> impl ChainPollParallel<Self::ChainOut, Context> {
        let (head, tail) = self;
        let mut new_context = context.fork();

        let head_res = head.spawn(input.clone(), context);

        let mut tail = tail.clone();
        let tail_fut = async move {
            let output = tail
                .run(input.into(), &mut new_context)
                .await
                .map_err(Into::into)?;
            Ok((output, new_context))
        };
        (head_res, MaybeDone::Future(tail_fut))
    }
}

impl<Input, Error, Context, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainSpawn<
        Input,
        Error,
        Context,
        (NodeOutputStruct<HeadNodeOutType>,),
        ChainLink<(), NodeIOE<HeadNodeInType, HeadNodeOutType, HeadNodeErrType>>,
    > for (Head,)
where
    Input: Into<HeadNodeInType> + Send,
    Head: Node<HeadNodeInType, NodeOutputStruct<HeadNodeOutType>, HeadNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    HeadNodeErrType: Into<Error>,
    HeadNodeOutType: Send,
    Error: Send,
    Context: Send,
{
    type ChainOut = Result<(NodeOutputStruct<HeadNodeOutType>,), Error>;
    const NUM_FUTURES: usize = 1;

    fn spawn(
        &self,
        input: Input,
        mut context: Context,
    ) -> impl ChainPollParallel<Self::ChainOut, Context> {
        let mut head = self.0.clone();
        let fut = async move {
            let output = head
                .run(input.into(), &mut context)
                .await
                .map_err(Into::into)?;
            Ok((output, context))
        };
        (MaybeDone::Future(fut),)
    }
}
