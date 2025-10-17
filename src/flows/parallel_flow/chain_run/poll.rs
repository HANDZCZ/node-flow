use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{future_utils::MaybeReady, storage::Storage};

pub trait ChainPollParallel<Output>: Send {
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage: &mut Storage,
    ) -> Poll<Output>;
}

impl<Head, Tail, HeadOutput, TailOutput, Error>
    ChainPollParallel<Result<(HeadOutput, TailOutput), Error>> for (Head, MaybeReady<Tail>)
where
    Head: ChainPollParallel<Result<HeadOutput, Error>>,
    Tail: Future<Output = Result<(TailOutput, Storage), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage: &mut Storage,
    ) -> Poll<Result<(HeadOutput, TailOutput), Error>> {
        let (head, tail) = unsafe { self.get_unchecked_mut() };
        let (head, mut tail) = unsafe { (Pin::new_unchecked(head), Pin::new_unchecked(tail)) };
        let tail_ready = tail.as_mut().poll(cx) && tail_ready;

        let Poll::Ready(res) = ChainPollParallel::poll(head, cx, tail_ready, storage) else {
            return Poll::Pending;
        };
        match res {
            Ok(head_out) => match tail.take_output() {
                Ok((tail_out, node_storage)) => {
                    // TODO: merge storage
                    Poll::Ready(Ok((head_out, tail_out)))
                }
                Err(e) => Poll::Ready(Err(e)),
            },
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<Head, HeadOutput, Error> ChainPollParallel<Result<(HeadOutput,), Error>> for (MaybeReady<Head>,)
where
    Head: Future<Output = Result<(HeadOutput, Storage), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage: &mut Storage,
    ) -> Poll<Result<(HeadOutput,), Error>> {
        let mut head = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        if head.as_mut().poll(cx) && tail_ready {
            match head.take_output() {
                Ok((output, node_storage)) => {
                    // TODO: merge storage
                    Poll::Ready(Ok((output,)))
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            Poll::Pending
        }
    }
}
