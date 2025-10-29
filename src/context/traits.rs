pub trait Fork {
    #[must_use]
    fn fork(&self) -> Self;
}

pub trait Update {
    fn update_from(&mut self, other: Self);
}

pub trait Join: Sized {
    fn join(&mut self, others: Box<[Self]>);
}

pub trait Task<T>: Future<Output = T> {
    fn is_finished(&self) -> bool;
    fn cancel(self);
}

pub trait SpawnAsync {
    fn spawn<F>(fut: F) -> impl Task<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

pub trait SpawnSync {
    fn spawn_blocking<F, O>(func: F) -> impl Task<O>
    where
        F: Fn() -> O + Send + 'static,
        O: Send + 'static;
}

#[cfg(test)]
pub(crate) mod test {
    use std::time::{Duration, Instant};

    use super::{SpawnAsync, SpawnSync, Task};

    mod tokio_ {
        use super::{SpawnAsync, SpawnSync, Task};
        use std::pin::Pin;

        pub struct TokioSpawner;
        struct TokioTask<T>(tokio::task::JoinHandle<T>);

        impl<T> Future for TokioTask<T> {
            type Output = T;

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                let task = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
                task.poll(cx).map(|r| r.unwrap())
            }
        }

        impl<T> Task<T> for TokioTask<T> {
            fn is_finished(&self) -> bool {
                self.0.is_finished()
            }

            fn cancel(self) {
                self.0.abort();
            }
        }

        impl SpawnAsync for TokioSpawner {
            fn spawn<F>(fut: F) -> impl super::Task<F::Output>
            where
                F: Future + Send + 'static,
                F::Output: Send + 'static,
            {
                TokioTask(tokio::spawn(fut))
            }
        }

        impl SpawnSync for TokioSpawner {
            fn spawn_blocking<F, O>(func: F) -> impl Task<O>
            where
                F: Fn() -> O + Send + 'static,
                O: Send + 'static,
            {
                TokioTask(tokio::task::spawn_blocking(func))
            }
        }
    }

    mod none {
        use super::{SpawnAsync, SpawnSync, Task};
        use futures_util::future::MaybeDone;
        use std::pin::Pin;

        pub struct NoneSpawner;

        struct NoneTask<F>(MaybeDone<F>)
        where
            F: Future;

        impl<T, F> Future for NoneTask<F>
        where
            F: Future<Output = T>,
        {
            type Output = T;

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                let mut task = unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) };
                task.as_mut().poll(cx).map(|_| task.take_output().unwrap())
            }
        }

        impl<T, F> Task<T> for NoneTask<F>
        where
            F: Future<Output = T>,
        {
            fn is_finished(&self) -> bool {
                matches!(self.0, MaybeDone::Done(_))
            }

            fn cancel(self) {}
        }

        impl SpawnAsync for NoneSpawner {
            fn spawn<F>(fut: F) -> impl super::Task<F::Output>
            where
                F: Future + Send + 'static,
                F::Output: Send + 'static,
            {
                NoneTask(MaybeDone::Future(fut))
            }
        }

        impl SpawnSync for NoneSpawner {
            fn spawn_blocking<F, O>(func: F) -> impl Task<O>
            where
                F: Fn() -> O + Send + 'static,
                O: Send + 'static,
            {
                NoneTask(MaybeDone::Future(async move { func() }))
            }
        }
    }

    pub use none::NoneSpawner;
    pub use tokio_::TokioSpawner;

    async fn test<T>(spawn_fn: impl Fn(u64) -> T) -> (u64, u64)
    where
        T: Task<()>,
    {
        let mut acc = Vec::new();
        let mut time_sum = 0;

        let start = Instant::now();

        for i in 0..15 {
            let delay = i % 5 + 3;
            time_sum += delay;
            acc.push(spawn_fn(i));
        }
        for f in acc {
            f.await;
        }

        let end = Instant::now();
        let took = end.duration_since(start).as_millis() as u64;
        (time_sum, took)
    }

    async fn test_async<S: SpawnAsync>() -> (u64, u64) {
        test(|delay| {
            S::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay)).await;
            })
        })
        .await
    }

    async fn test_sync<S: SpawnSync>() -> (u64, u64) {
        test(|delay| {
            S::spawn_blocking(move || {
                std::thread::sleep(Duration::from_millis(delay));
            })
        })
        .await
    }

    #[tokio::test]
    async fn test_async_tokio() {
        let (time_sum, took) = test_async::<TokioSpawner>().await;
        println!("time_sum: {time_sum}, took: {took}");
        assert!(time_sum > took);
    }

    #[tokio::test]
    async fn test_async_none() {
        let (time_sum, took) = test_async::<NoneSpawner>().await;
        println!("time_sum: {time_sum}, took: {took}");
        assert!(time_sum <= took);
    }

    #[tokio::test]
    async fn test_sync_tokio() {
        let (time_sum, took) = test_sync::<TokioSpawner>().await;
        println!("time_sum: {time_sum}, took: {took}");
        assert!(time_sum > took);
    }

    #[tokio::test]
    async fn test_sync_none() {
        let (time_sum, took) = test_sync::<NoneSpawner>().await;
        println!("time_sum: {time_sum}, took: {took}");
        assert!(time_sum <= took);
    }
}
