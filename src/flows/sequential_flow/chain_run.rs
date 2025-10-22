use crate::{
    flows::{ChainLink, NodeIOE, NodeResult},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub trait ChainRunSequential<Input, Output, Context, T> {
    fn run(&self, input: Input, context: &mut Context) -> impl Future<Output = Output> + Send;
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
    ChainRunSequential<
        Input,
        NodeResult<Output, Error>,
        Context,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head:
        ChainRunSequential<Input, NodeResult<TailNodeInType, Error>, Context, HeadIOETypes> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    TailNodeInType: Send,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Send,
    Error: Send,
    Context: Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> NodeResult<Output, Error> {
        let (head, tail) = self;
        if let NodeOutputStruct::Ok(input) = head.run(input, context).await? {
            let output = tail.clone().run(input, context).await.map_err(Into::into)?;
            return Ok(match output {
                NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
            });
        }
        Ok(NodeOutputStruct::SoftFail)
    }
}

impl<Input, Output, Error, Context, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainRunSequential<
        Input,
        NodeResult<Output, Error>,
        Context,
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
    Context: Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> NodeResult<Output, Error> {
        let output = self
            .0
            .clone()
            .run(input.into(), context)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
        })
    }
}
