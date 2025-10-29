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
