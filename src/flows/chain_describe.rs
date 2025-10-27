use crate::{
    describe::Description,
    flows::{ChainLink, NodeIOE},
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub trait ChainDescribe<Context, T> {
    const COUNT: usize;

    fn describe(&self, description_acc: &mut Vec<Description>);
}

impl<Head, Tail, Context, HeadIOETypes, TailNodeInType, TailNodeOutType, TailNodeErrType>
    ChainDescribe<
        Context,
        ChainLink<HeadIOETypes, NodeIOE<TailNodeInType, TailNodeOutType, TailNodeErrType>>,
    > for (Head, Tail)
where
    Head: ChainDescribe<Context, HeadIOETypes>,
    Tail: Node<TailNodeInType, NodeOutputStruct<TailNodeOutType>, TailNodeErrType, Context>,
{
    const COUNT: usize = Head::COUNT + 1;

    fn describe(&self, description_acc: &mut Vec<Description>) {
        let (head, tail) = self;
        ChainDescribe::describe(head, description_acc);
        description_acc.push(tail.describe());
    }
}

impl<Head, Context, HeadNodeInType, HeadNodeOutType, HeadNodeErrType>
    ChainDescribe<Context, ChainLink<(), NodeIOE<HeadNodeInType, HeadNodeOutType, HeadNodeErrType>>>
    for (Head,)
where
    Head: Node<HeadNodeInType, NodeOutputStruct<HeadNodeOutType>, HeadNodeErrType, Context>,
{
    const COUNT: usize = 1;

    fn describe(&self, description_acc: &mut Vec<Description>) {
        description_acc.push(self.0.describe());
    }
}
