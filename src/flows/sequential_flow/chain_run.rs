use crate::{
    flows::{ChainLink, NodeIOE, NodeResult},
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};

pub(crate) trait ChainRunSequential<Input, Output, T> {
    fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> impl Future<Output = Output> + Send;
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
    ChainRunSequential<
        Input,
        NodeResult<Output, Error>,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainRunSequential<Input, NodeResult<TailNodeInType, Error>, HeadIOETypes> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType>
        + Clone
        + Send
        + Sync,
    TailNodeInType: Send,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Send,
    Error: Send,
{
    async fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> NodeResult<Output, Error> {
        let (head, tail) = self;
        if let NodeOutputStruct::Ok(input) = head.run_with_storage(input, storage).await? {
            let output = tail
                .clone()
                .run_with_storage(input, storage)
                .await
                .map_err(Into::into)?;
            return Ok(match output {
                NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
                NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
            });
        }
        Ok(NodeOutputStruct::SoftFail)
    }
}

impl<Input, Output, Error, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainRunSequential<
        Input,
        NodeResult<Output, Error>,
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
    async fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> NodeResult<Output, Error> {
        let output = self
            .0
            .clone()
            .run_with_storage(input.into(), storage)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => NodeOutputStruct::Ok(output.into()),
        })
    }
}
