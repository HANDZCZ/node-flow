use std::ops::{Deref, DerefMut};

pub trait SharedStorage {
    /// Gets reference of a value with type `T` from storage if it is present.
    fn get<T>(&self) -> impl Future<Output = Option<impl Deref<Target = T>>> + Send
    where
        T: 'static;

    /// Gets mutable reference of a value with type `T` from storage if it is present.
    fn get_mut<T>(&mut self) -> impl Future<Output = Option<impl DerefMut<Target = T>>> + Send
    where
        T: 'static;

    /// Inserts value with type `T` to storage and returns the value that was there previously if it was there.
    fn insert<T>(&mut self, val: T) -> impl Future<Output = Option<T>> + Send
    where
        T: Send + Sync + 'static;

    /// Inserts value with type `T` to storage if it doesn't contain it.
    fn insert_with_if_absent<T, E>(
        &self,
        fut: impl Future<Output = Result<T, E>> + Send,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        T: Send + Sync + 'static,
        E: Send;

    /// Removes and returns value with type `T` from storage if it is present.
    fn remove<T>(&mut self) -> impl Future<Output = Option<T>> + Send
    where
        T: 'static;
}
