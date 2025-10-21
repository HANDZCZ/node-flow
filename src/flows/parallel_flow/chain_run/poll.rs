use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::future::MaybeDone;

use crate::storage::Storage;

pub trait ChainPollParallel<Output>: Send {
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage_acc: &mut Vec<Storage>,
    ) -> Poll<Output>;
}

impl<Head, Tail, HeadOutput, TailOutput, Error>
    ChainPollParallel<Result<(HeadOutput, TailOutput), Error>> for (Head, MaybeDone<Tail>)
where
    TailOutput: Send,
    Error: Send,
    Head: ChainPollParallel<Result<HeadOutput, Error>>,
    Tail: Future<Output = Result<(TailOutput, Storage), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage_acc: &mut Vec<Storage>,
    ) -> Poll<Result<(HeadOutput, TailOutput), Error>> {
        let (head, tail) = unsafe { self.get_unchecked_mut() };
        let (head, mut tail) = unsafe { (Pin::new_unchecked(head), Pin::new_unchecked(tail)) };
        let tail_ready = tail.as_mut().poll(cx).is_ready() && tail_ready;

        let Poll::Ready(res) = ChainPollParallel::poll(head, cx, tail_ready, storage_acc) else {
            return Poll::Pending;
        };
        match res {
            Ok(head_out) => match tail.take_output().unwrap() {
                Ok((tail_out, node_storage)) => {
                    storage_acc.push(node_storage);
                    Poll::Ready(Ok((head_out, tail_out)))
                }
                Err(e) => Poll::Ready(Err(e)),
            },
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<Head, HeadOutput, Error> ChainPollParallel<Result<(HeadOutput,), Error>> for (MaybeDone<Head>,)
where
    Error: Send,
    HeadOutput: Send,
    Head: Future<Output = Result<(HeadOutput, Storage), Error>> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tail_ready: bool,
        storage_acc: &mut Vec<Storage>,
    ) -> Poll<Result<(HeadOutput,), Error>> {
        let mut head = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
        if head.as_mut().poll(cx).is_ready() && tail_ready {
            match head.take_output().unwrap() {
                Ok((output, node_storage)) => {
                    storage_acc.push(node_storage);
                    Poll::Ready(Ok((output,)))
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            Poll::Pending
        }
    }
}
