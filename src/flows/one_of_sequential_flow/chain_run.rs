use crate::{
    flows::{ChainLink, NodeIOE, NodeResult},
    node::{Node, NodeOutput as NodeOutputStruct},
    storage::Storage,
};

pub(crate) trait ChainRunOneOfSequential<Input, Output, T> {
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
    ChainRunOneOfSequential<
        Input,
        NodeResult<Output, Error>,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainRunOneOfSequential<Input, NodeResult<Output, Error>, HeadIOETypes> + Sync,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType>
        + Clone
        + Send
        + Sync,
    TailNodeErrType: Into<Error>,
    TailNodeOutType: Into<Output>,
    Input: Into<TailNodeInType> + Clone + Send,
{
    async fn run_with_storage(
        &self,
        input: Input,
        storage: &mut Storage,
    ) -> NodeResult<Output, Error> {
        let (head, tail) = self;
        if let NodeOutputStruct::Ok(res) = head.run_with_storage(input.clone(), storage).await? {
            return Ok(NodeOutputStruct::Ok(res));
        }
        let mut new_storage = storage.new_gen();
        let output = tail
            .clone()
            .run_with_storage(input.into(), &mut new_storage)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => {
                storage.replace(new_storage);
                NodeOutputStruct::Ok(output.into())
            }
        })
    }
}

impl<Input, Output, Error, HeadNodeInType, HeadNodeOutType, HeadNodeErrType, Head>
    ChainRunOneOfSequential<
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
        let mut new_storage = storage.new_gen();
        let output = self
            .0
            .clone()
            .run_with_storage(input.into(), &mut new_storage)
            .await
            .map_err(Into::into)?;
        Ok(match output {
            NodeOutputStruct::SoftFail => NodeOutputStruct::SoftFail,
            NodeOutputStruct::Ok(output) => {
                storage.replace(new_storage);
                NodeOutputStruct::Ok(output.into())
            }
        })
    }
}
