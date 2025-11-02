use std::ops::{Deref, DerefMut};

/// Provides type-based shared storage for concurrent environments.
///
/// `SharedStorage` is a type-based shared storage.
/// Items in this storage are be shared with all other branches.
/// It allows storing and retrieving values by their type.
/// Each type `T` has at most **one instance** stored at a time.
///
/// This trait is designed for use in systems that require shared, concurrent
/// access to arbitrary state.
///
/// Unlike [`LocalStorage`](crate::context::storage::LocalStorage) implementors of this trait
/// are able to safely manage data shared across branches.
///
/// # Design
///
/// Implementations are expected to handle concurrency correctly (for example via
/// `RwLock`, `Mutex`, or some lock-free data structure) and to return appropriate
/// guard objects implementing [`Deref`] or [`DerefMut`] for temporary access.
///
/// # Examples
/// ```
/// use std::sync::{Arc, RwLock};
/// use std::future::Future;
/// use std::ops::{Deref, DerefMut};
/// use node_flow::context::storage::SharedStorage;
///
/// // Always empty storage (example only)
/// struct ExampleSharedStorage;
/// // Guard for get and get_mut
/// struct Guard<T>(T);
/// impl<T> Deref for Guard<T> // ...
/// # {
/// #     type Target = T;
/// #
/// #     fn deref(&self) -> &Self::Target {
/// #         unreachable!()
/// #     }
/// # }
/// impl<T> DerefMut for Guard<T> // ...
/// # {
/// #     fn deref_mut(&mut self) -> &mut Self::Target {
/// #         unreachable!()
/// #     }
/// # }
///
/// impl SharedStorage for ExampleSharedStorage {
///     fn get<T>(&self) -> impl Future<Output = Option<impl Deref<Target = T>>> + Send {
///         async { None::<Guard<T>> }
///     }
///
///     fn get_mut<T>(&mut self) -> impl Future<Output = Option<impl DerefMut<Target = T>>> + Send {
///         async { None::<Guard<T>> }
///     }
///
///     fn insert<T>(&mut self, _val: T) -> impl Future<Output = Option<T>> + Send {
///         async { None }
///     }
///
///     fn insert_with_if_absent<T, E>(
///         &self,
///         fut: impl Future<Output = Result<T, E>> + Send,
///     ) -> impl Future<Output = Result<(), E>> + Send {
///         async {
///             let _ = fut.await;
///             Ok(())
///         }
///     }
///
///     fn remove<T>(&mut self) -> impl Future<Output = Option<T>> + Send {
///         async { None }
///     }
/// }
/// ```
pub trait SharedStorage {
    /// Gets reference of a value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// # use node_flow::context::storage::{SharedStorage, shared_storage::SharedStorageImpl};
    /// # type ExampleStorage = SharedStorageImpl;
    /// use std::ops::Deref;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct ExampleValue(u8);
    /// let mut storage = ExampleStorage::new();
    ///
    /// let _ = storage.insert(ExampleValue(5u8)).await;
    /// let guard = storage.get().await;
    /// let result: Option<&ExampleValue> = guard.as_deref();
    /// assert_eq!(result, Some(&ExampleValue(5u8)));
    /// let guard = storage.get().await;
    /// let result: Option<&u16> = guard.as_deref();
    /// assert_eq!(result, None);
    /// # });
    /// ```
    fn get<T>(&self) -> impl Future<Output = Option<impl Deref<Target = T>>> + Send
    where
        T: 'static;

    /// Gets mutable reference of a value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// # use node_flow::context::storage::{SharedStorage, shared_storage::SharedStorageImpl};
    /// # type ExampleStorage = SharedStorageImpl;
    /// use std::ops::Deref;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct ExampleValue(u8);
    /// let mut storage = ExampleStorage::new();
    ///
    /// let _ = storage.insert(ExampleValue(5u8)).await;
    /// if let Some(mut val) = storage.get_mut::<ExampleValue>().await {
    ///     val.0 = 15u8;
    /// }
    /// let guard = storage.get().await;
    /// let result: Option<&ExampleValue> = guard.as_deref();
    /// assert_eq!(result, Some(&ExampleValue(15u8)));
    /// # });
    /// ```
    fn get_mut<T>(&mut self) -> impl Future<Output = Option<impl DerefMut<Target = T>>> + Send
    where
        T: 'static;

    /// Inserts value with type `T` to storage and returns the value that was there previously if it was there.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// # use node_flow::context::storage::{SharedStorage, shared_storage::SharedStorageImpl};
    /// # type ExampleStorage = SharedStorageImpl;
    /// use std::ops::Deref;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct ExampleValue(u8);
    /// let mut storage = ExampleStorage::new();
    ///
    /// let result = storage.insert(ExampleValue(5u8)).await;
    /// assert_eq!(result, None);
    /// let _ = storage.insert(ExampleValue(15u8)).await;
    /// let result = storage.insert(ExampleValue(25u8)).await;
    /// assert_eq!(result, Some(ExampleValue(15u8)));
    /// let guard = storage.get().await;
    /// let result: Option<&ExampleValue> = guard.as_deref();
    /// assert_eq!(result, Some(&ExampleValue(25u8)));
    /// # });
    /// ```
    fn insert<T>(&mut self, val: T) -> impl Future<Output = Option<T>> + Send
    where
        T: Send + Sync + 'static;

    /// Inserts value with type `T` to storage if it doesn't contain it.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// # use node_flow::context::storage::{SharedStorage, shared_storage::SharedStorageImpl};
    /// # type ExampleStorage = SharedStorageImpl;
    /// use std::ops::Deref;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct ExampleValue(u8);
    /// let mut storage = ExampleStorage::new();
    ///
    /// storage.insert_with_if_absent(async { Ok::<_, ()>(ExampleValue(5u8)) }).await.unwrap();
    /// storage.insert_with_if_absent(async { Ok::<_, ()>(ExampleValue(15u8)) }).await.unwrap();
    /// let result = storage.remove().await;
    /// assert_eq!(result, Some(ExampleValue(5u8)));
    /// storage.insert_with_if_absent(async { Ok::<_, ()>(ExampleValue(25u8)) }).await.unwrap();
    /// let result = storage.remove().await;
    /// assert_eq!(result, Some(ExampleValue(25u8)));
    /// # });
    /// ```
    fn insert_with_if_absent<T, E>(
        &self,
        fut: impl Future<Output = Result<T, E>> + Send,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        T: Send + Sync + 'static,
        E: Send;

    /// Removes and returns value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// # use node_flow::context::storage::{SharedStorage, shared_storage::SharedStorageImpl};
    /// # type ExampleStorage = SharedStorageImpl;
    /// use std::ops::Deref;
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct ExampleValue(u8);
    /// let mut storage = ExampleStorage::new();
    ///
    /// let result = storage.insert(ExampleValue(5u8)).await;
    /// assert_eq!(result, None);
    /// let result = storage.remove().await;
    /// assert_eq!(result, Some(ExampleValue(5u8)));
    /// let result = storage.remove::<ExampleValue>().await;
    /// assert_eq!(result, None);
    /// # });
    /// ```
    fn remove<T>(&mut self) -> impl Future<Output = Option<T>> + Send
    where
        T: 'static;
}
