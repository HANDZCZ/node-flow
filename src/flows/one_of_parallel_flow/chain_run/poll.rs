use std::{pin::Pin, task::Context};

use futures_util::future::MaybeDone;

use crate::{
    flows::one_of_parallel_flow::FutOutput, future_utils::SoftFailPoll,
    node::NodeOutput as NodeOutputStruct,
};

pub trait ChainPollOneOfParallel<Output, NodeContext>: Send {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> SoftFailPoll<Output>;
}

impl<Head, Tail, Output, Error, NodeContext>
    ChainPollOneOfParallel<FutOutput<Output, Error, NodeContext>, NodeContext>
    for (Head, MaybeDone<Tail>)
where
    Error: Send,
    Output: Send,
    NodeContext: Send,
    Head: ChainPollOneOfParallel<FutOutput<Output, Error, NodeContext>, NodeContext>,
    Tail: Future<Output = FutOutput<Output, Error, NodeContext>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> SoftFailPoll<FutOutput<Output, Error, NodeContext>> {
        let (head, tail) = unsafe { self.get_unchecked_mut() };
        let head = unsafe { Pin::new_unchecked(head) };
        let head_pending = match ChainPollOneOfParallel::poll(head, cx) {
            SoftFailPoll::Pending => true,
            SoftFailPoll::Ready(res) => return SoftFailPoll::Ready(res),
            SoftFailPoll::SoftFail => false,
        };

        match (matches!(tail, MaybeDone::Gone), head_pending) {
            (true, true) => return SoftFailPoll::Pending,
            (true, false) => return SoftFailPoll::SoftFail,
            (false, _) => {}
        }
        let mut tail = unsafe { Pin::new_unchecked(tail) };
        if tail.as_mut().poll(cx).is_ready() {
            match tail.take_output().unwrap() {
                output if matches!(output, Ok((NodeOutputStruct::Ok(_), _)) | Err(_)) => {
                    SoftFailPoll::Ready(output)
                }
                _ if head_pending => SoftFailPoll::Pending,
                _ => SoftFailPoll::SoftFail,
            }
        } else {
            SoftFailPoll::Pending
        }
    }
}

impl<Head, Output, Error, NodeContext>
    ChainPollOneOfParallel<FutOutput<Output, Error, NodeContext>, NodeContext> for (MaybeDone<Head>,)
where
    Error: Send,
    Output: Send,
    NodeContext: Send,
    Head: Future<Output = FutOutput<Output, Error, NodeContext>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> SoftFailPoll<FutOutput<Output, Error, NodeContext>> {
        if matches!(self.0, MaybeDone::Gone) {
            return SoftFailPoll::SoftFail;
        }
        let mut head = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        if head.as_mut().poll(cx).is_ready() {
            match head.take_output().unwrap() {
                output if matches!(output, Ok((NodeOutputStruct::Ok(_), _)) | Err(_)) => {
                    SoftFailPoll::Ready(output)
                }
                _ => SoftFailPoll::SoftFail,
            }
        } else {
            SoftFailPoll::Pending
        }
    }
}
