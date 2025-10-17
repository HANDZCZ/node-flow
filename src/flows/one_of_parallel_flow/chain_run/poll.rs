use std::{pin::Pin, task::Context};

use crate::{
    flows::one_of_parallel_flow::FutOutput,
    future_utils::{MaybeReady, SoftFailPoll},
    node::NodeOutput as NodeOutputStruct,
};

pub trait ChainPollOneOfParallel<Output>: Send {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> SoftFailPoll<Output>;
}

impl<Head, Tail, Output, Error> ChainPollOneOfParallel<FutOutput<Output, Error>>
    for (Head, MaybeReady<Tail>)
where
    Head: ChainPollOneOfParallel<FutOutput<Output, Error>>,
    Tail: Future<Output = FutOutput<Output, Error>> + Send,
{
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> SoftFailPoll<FutOutput<Output, Error>> {
        let (head, tail) = unsafe { self.get_unchecked_mut() };
        let head = unsafe { Pin::new_unchecked(head) };
        let head_pending = match ChainPollOneOfParallel::poll(head, cx) {
            SoftFailPoll::Pending => true,
            SoftFailPoll::Ready(res) => return SoftFailPoll::Ready(res),
            SoftFailPoll::SoftFail => false,
        };

        match (tail.is_taken(), head_pending) {
            (true, true) => return SoftFailPoll::Pending,
            (true, false) => return SoftFailPoll::SoftFail,
            (false, _) => {}
        }
        let mut tail = unsafe { Pin::new_unchecked(tail) };
        if tail.as_mut().poll(cx) {
            match tail.take_output() {
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

impl<Head, Output, Error> ChainPollOneOfParallel<FutOutput<Output, Error>> for (MaybeReady<Head>,)
where
    Head: Future<Output = FutOutput<Output, Error>> + Send,
{
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> SoftFailPoll<FutOutput<Output, Error>> {
        if self.0.is_taken() {
            return SoftFailPoll::SoftFail;
        }
        let mut head = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        if head.as_mut().poll(cx) {
            match head.take_output() {
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
