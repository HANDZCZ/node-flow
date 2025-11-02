/// The `Fork` trait is used for creating a new instances a context from an existing one.
///
/// `Fork` is used in a flow where context must sent into branches.
/// For example, when spawning parallel tasks that each require their own context.
///
/// # Examples
/// ```
/// use node_flow::context::Fork;
///
/// #[derive(Clone)]
/// struct ExampleContext {
///     id: usize,
/// }
///
/// impl Fork for ExampleContext {
///     fn fork(&self) -> Self {
///         Self { id: self.id + 1 }
///     }
/// }
///
/// let ctx = ExampleContext { id: 0 };
/// let forked = ctx.fork();
/// assert_eq!(forked.id, 1);
/// ```
pub trait Fork {
    /// Creates a forked instance of the implementor.
    #[must_use]
    fn fork(&self) -> Self;
}

/// The `Update` trait is used for updating the state of a context from another.
///
/// `Update` allows incremental merging or synchronization between two instances of the same type.
///
/// # Examples
/// ```
/// use node_flow::context::Update;
///
/// struct Stats {
///     count: usize,
/// }
///
/// impl Update for Stats {
///     fn update_from(&mut self, other: Self) {
///         self.count += other.count;
///     }
/// }
///
/// let mut a = Stats { count: 5 };
/// let b = Stats { count: 3 };
/// a.update_from(b);
/// assert_eq!(a.count, 8);
/// ```
pub trait Update {
    /// Merges or synchronizes the state of `other` into `self`.
    ///
    /// The specifics depend on the implementor.
    /// For example it could be implemented as additive merging,
    /// overwriting fields, or some conflict resolution.
    fn update_from(&mut self, other: Self);
}

/// The `Join` trait is used for merging multiple instances of a context into one.
///
/// `Join` is typically used after parallel execution, where several contexts
/// (or partial results) must be combined back into a single instance.
///
/// This trait complements [`Fork`], enabling a *fork-join* lifecycle for contexts.
///
/// # Examples
/// ```
/// use node_flow::context::Join;
///
/// struct Sum(usize);
///
/// impl Join for Sum {
///     fn join(&mut self, others: Box<[Self]>) {
///         for other in others {
///             self.0 += other.0;
///         }
///     }
/// }
///
/// let mut total = Sum(5);
/// total.join(Box::new([Sum(2), Sum(3)]));
/// assert_eq!(total.0, 10);
/// ```
pub trait Join: Sized {
    /// Joins the state of multiple `others` into this instance.
    ///
    /// Implementors define how merging should occur.
    /// For example it could be summation, set unions or aggregation.
    fn join(&mut self, others: Box<[Self]>);
}

/// The `Task` trait represents an asynchronous task.
///
/// `Task` is an abstraction over a specific task in some async runtime like
/// [`tokio::task::JoinHandle<T>`](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html).
///
/// # Examples
/// ```
/// use node_flow::context::Task;
/// use std::future::Future;
///
/// struct DummyTask;
///
/// impl Future for DummyTask {
///     type Output = u8;
///     fn poll(
///         self: std::pin::Pin<&mut Self>,
///         _: &mut std::task::Context<'_>
///     ) -> std::task::Poll<Self::Output> {
///         std::task::Poll::Ready(5)
///     }
/// }
///
/// impl Task<u8> for DummyTask {
///     fn is_finished(&self) -> bool { true }
///     fn cancel(self) {}
/// }
/// ```
pub trait Task<T>: Future<Output = T> {
    /// Returns `true` if the task has finished.
    fn is_finished(&self) -> bool;
    /// Cancels the task if it is still running.
    ///
    /// The implementation should attempt to stop or drop any ongoing work.
    /// Be aware that tasks spawned using [`SpawnSync::spawn_blocking`] may or may not be canceled,
    /// because they are not async (it all depends on the implementor).
    fn cancel(self);
}

/// The `SpawnAsync` trait provides an interface for spawning asynchronous tasks on a runtime or executor.
///
/// This trait abstracts over asynchronous task execution environments
/// (such as Tokio, smol, or a custom thread pool).
/// It returns a handle implementing the [`Task`] trait.
///
/// # Examples
/// ```
/// use node_flow::context::{SpawnAsync, Task};
/// use std::future::Future;
///
/// struct MyRuntime;
/// struct DummyTask<T>(T);
/// impl<T> Future for DummyTask<T> // ...
/// # {
/// #     type Output = T;
/// #     fn poll(
/// #         self: std::pin::Pin<&mut Self>,
/// #         _: &mut std::task::Context<'_>
/// #     ) -> std::task::Poll<Self::Output> {
/// #         todo!()
/// #     }
/// # }
/// impl<T> Task<T> for DummyTask<T> // ...
/// # {
/// #     fn is_finished(&self) -> bool { todo!() }
/// #     fn cancel(self) {}
/// # }
///
/// impl SpawnAsync for MyRuntime {
///     fn spawn<F>(fut: F) -> impl Task<F::Output>
///     where
///         F: Future + Send + 'static,
///         F::Output: Send + 'static,
///     {
///         // Example stub (replace with actual runtime call)
///         DummyTask(todo!())
///     }
/// }
/// ```
pub trait SpawnAsync {
    /// Spawns an asynchronous concurrent task.
    ///
    /// The task must be `Send + 'static`, as it may execute on another thread.
    ///
    /// # Returns
    /// A task handle implementing [`Task`] trait.
    fn spawn<F>(fut: F) -> impl Task<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

/// The `SpawnSync` trait provides an interface for spawning **blocking** (synchronous) tasks.
///
/// This trait is the synchronous version of the [`SpawnAsync`] trait,
/// allowing potentially long-running CPU-bound or blocking operations to be executed.
/// It returns a handle implementing the [`Task`] trait.
///
/// # Examples
/// ```
/// use node_flow::context::{SpawnSync, Task};
///
/// struct MyRuntime;
/// struct DummyTask<T>(T);
/// impl<T> Future for DummyTask<T> // ...
/// # {
/// #     type Output = T;
/// #     fn poll(
/// #         self: std::pin::Pin<&mut Self>,
/// #         _: &mut std::task::Context<'_>
/// #     ) -> std::task::Poll<Self::Output> {
/// #         todo!()
/// #     }
/// # }
/// impl<T> Task<T> for DummyTask<T> // ...
/// # {
/// #     fn is_finished(&self) -> bool { todo!() }
/// #     fn cancel(self) {}
/// # }
///
/// impl SpawnSync for MyRuntime {
///     fn spawn_blocking<F, O>(func: F) -> impl Task<O>
///     where
///         F: Fn() -> O + Send + 'static,
///         O: Send + 'static,
///     {
///         // Example stub (replace with actual runtime call)
///         DummyTask(func())
///     }
/// }
/// ```
pub trait SpawnSync {
    /// Spawns a blocking (synchronous) function in a background.
    ///
    /// The function `func` is executed on a separate worker thread. The returned
    /// task can be awaited or canceled, depending on the runtime implementation.
    ///
    /// # Type Parameters
    /// - `F`: A closure or function that produces an output of type `O`.
    /// - `O`: The output type of the blocking computation.
    ///
    /// # Returns
    /// A task handle implementing [`Task<O>`] trait.
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
