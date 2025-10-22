use futures_util::future::MaybeDone;

use crate::{
    context::Fork,
    flows::{
        ChainLink, NodeIOE, NodeResult,
        one_of_parallel_flow::{FutOutput, chain_run::poll::ChainPollOneOfParallel},
    },
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub trait ChainSpawn<Input, Output, Context, ChainOut, T> {
    fn run(&self, input: Input, context: Context)
    -> impl ChainPollOneOfParallel<ChainOut, Context>;
}

impl<
    Input,
    Output,
    Error,
    Context,
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
        Context,
        FutOutput<Output, Error, Context>,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainSpawn<
            Input,
            NodeResult<Output, Error>,
            Context,
            FutOutput<Output, Error, Context>,
            HeadIOETypes,
        > + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Into<TailNodeInType> + Clone + Send,
    Output: Send,
    Error: Send,
    Context: Fork + Send,
{
    fn run(
        &self,
        input: Input,
        context: Context,
    ) -> impl ChainPollOneOfParallel<FutOutput<Output, Error, Context>, Context> {
        let (head, tail) = self;
        let mut new_context = context.fork();

        let head_res = head.run(input.clone(), context);

        let mut tail = tail.clone();
        let tail_fut = async move {
            let output = tail
                .run(input.into(), &mut new_context)
                .await
                .map_err(Into::into)?;
            Ok((
                match output {
                    NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                    NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
                },
                new_context,
            ))
        };
        (head_res, MaybeDone::Future(tail_fut))
    }
}

impl<Input, Output, Error, Context, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainSpawn<
        Input,
        NodeResult<Output, Error>,
        Context,
        FutOutput<Output, Error, Context>,
        ChainLink<(), NodeIOE<HeadNodeInType, HeadNodeOutType, HeadNodeErrType>>,
    > for (Head,)
where
    Input: Into<HeadNodeInType> + Send,
    Head: Node<HeadNodeInType, NodeOutputStruct<HeadNodeOutType>, HeadNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    HeadNodeErrType: Into<Error>,
    HeadNodeOutType: Into<Output>,
    Output: Send,
    Error: Send,
    Context: Send,
{
    fn run(
        &self,
        input: Input,
        mut context: Context,
    ) -> impl ChainPollOneOfParallel<FutOutput<Output, Error, Context>, Context> {
        let mut head = self.0.clone();
        let fut = async move {
            let output = head
                .run(input.into(), &mut context)
                .await
                .map_err(Into::into)?;
            Ok((
                match output {
                    NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                    NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
                },
                context,
            ))
        };
        (MaybeDone::Future(fut),)
    }
}
