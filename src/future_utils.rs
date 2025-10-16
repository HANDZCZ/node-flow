use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub enum MaybeReady<T>
where
    T: Future,
{
    Pending(T),
    Ready(T::Output),
    Taken,
}

impl<T> MaybeReady<T>
where
    T: Future,
{
    pub fn is_taken(&self) -> bool {
        matches!(self, Self::Taken)
    }

    pub fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> bool {
        match unsafe { self.as_mut().get_unchecked_mut() } {
            MaybeReady::Pending(fut) => {
                let fut = unsafe { Pin::new_unchecked(fut) };
                match fut.poll(cx) {
                    Poll::Ready(res) => {
                        self.set(Self::Ready(res));
                        true
                    }
                    Poll::Pending => false,
                }
            }
            MaybeReady::Ready(_) => true,
            MaybeReady::Taken => panic!("Poll after result taken"),
        }
    }

    pub fn take_unchecked(mut self: Pin<&mut Self>) -> T::Output {
        match &*self {
            MaybeReady::Pending(_) => panic!("Future is pending"),
            MaybeReady::Taken => panic!("Result was already taken"),
            MaybeReady::Ready(_) => {}
        }
        match unsafe { std::mem::replace(self.as_mut().get_unchecked_mut(), Self::Taken) } {
            MaybeReady::Ready(res) => res,
            _ => unreachable!(),
        }
    }
}

unsafe impl<T> Send for MaybeReady<T> where T: Future + Send {}

pub enum SoftFailPoll<T> {
    Pending,
    Ready(T),
    SoftFail,
}
