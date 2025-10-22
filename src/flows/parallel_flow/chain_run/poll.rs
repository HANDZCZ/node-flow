use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::future::MaybeDone;

pub trait ChainPollParallel<Output, NodeContext>: Send {
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        context_acc: &mut Vec<NodeContext>,
    ) -> Poll<Output>;
}

impl<Head, Tail, HeadOutput, TailOutput, Error, NodeContext>
    ChainPollParallel<Result<(HeadOutput, TailOutput), Error>, NodeContext>
    for (Head, MaybeDone<Tail>)
where
    TailOutput: Send,
    Error: Send,
    NodeContext: Send,
    Head: ChainPollParallel<Result<HeadOutput, Error>, NodeContext>,
    Tail: Future<Output = Result<(TailOutput, NodeContext), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        context_acc: &mut Vec<NodeContext>,
    ) -> Poll<Result<(HeadOutput, TailOutput), Error>> {
        let (head, tail) = unsafe { self.get_unchecked_mut() };
        let (head, mut tail) = unsafe { (Pin::new_unchecked(head), Pin::new_unchecked(tail)) };
        let tail_ready = tail.as_mut().poll(cx).is_ready() && tail_ready;

        let Poll::Ready(res) = ChainPollParallel::poll(head, cx, tail_ready, context_acc) else {
            return Poll::Pending;
        };
        match res {
            Ok(head_out) => match tail.take_output().unwrap() {
                Ok((tail_out, node_context)) => {
                    context_acc.push(node_context);
                    Poll::Ready(Ok((head_out, tail_out)))
                }
                Err(e) => Poll::Ready(Err(e)),
            },
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<Head, HeadOutput, Error, NodeContext>
    ChainPollParallel<Result<(HeadOutput,), Error>, NodeContext> for (MaybeDone<Head>,)
where
    Error: Send,
    NodeContext: Send,
    HeadOutput: Send,
    Head: Future<Output = Result<(HeadOutput, NodeContext), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        context_acc: &mut Vec<NodeContext>,
    ) -> Poll<Result<(HeadOutput,), Error>> {
        let mut head = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        if head.as_mut().poll(cx).is_ready() && tail_ready {
            match head.take_output().unwrap() {
                Ok((output, node_context)) => {
                    context_acc.push(node_context);
                    Poll::Ready(Ok((output,)))
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            Poll::Pending
        }
    }
}
