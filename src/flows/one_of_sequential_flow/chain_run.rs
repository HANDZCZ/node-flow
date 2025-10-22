use crate::{
    context::{Fork, Update},
    flows::{ChainLink, NodeIOE, NodeResult},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub trait ChainRunOneOfSequential<Input, Output, Context, T> {
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
    ChainRunOneOfSequential<
        Input,
        NodeResult<Output, Error>,
        Context,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainRunOneOfSequential<Input, NodeResult<Output, Error>, Context, HeadIOETypes> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType, Context>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Into<TailNodeInType> + Clone + Send,
    Context: Fork + Update + Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> NodeResult<Output, Error> {
        let (head, tail) = self;
        if let NodeOutputStruct::Ok(res) = head.run(input.clone(), context).await? {
            return Ok(NodeOutputStruct::Ok(res));
        }
        let mut new_context = context.fork();
        let output = tail
            .clone()
            .run(input.into(), &mut new_context)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => {
                context.update_from(new_context);
                NodeOutputStruct::Ok(output.into())
            }
        })
    }
}

impl<Input, Output, Error, Context, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainRunOneOfSequential<
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
    Context: Fork + Update + Send,
{
    async fn run(&self, input: Input, context: &mut Context) -> NodeResult<Output, Error> {
        let mut new_context = context.fork();
        let output = self
            .0
            .clone()
            .run(input.into(), &mut new_context)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => {
                context.update_from(new_context);
                NodeOutputStruct::Ok(output.into())
            }
        })
    }
}
