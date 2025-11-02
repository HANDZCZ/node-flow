/// Provides type-based local storage for arbitrary values.
///
/// `LocalStorage` is a type-based per branch local storage,
/// which is not be shared with any other branch.
/// It allows storing and retrieving values by their type.
/// Each type `T` has at most **one instance** stored at a time.
///
/// This trait is designed for use in systems that need to manage
/// per-node state which should be shared between nodes in the same branch,
/// but not between branches.
///
/// # Examples
/// ```
/// use node_flow::context::storage::LocalStorage;
/// use std::collections::HashMap;
/// use std::any::{TypeId, Any};
///
/// struct ExampleStorage(HashMap<TypeId, Box<dyn Any>>);
///
/// impl LocalStorage for ExampleStorage {
///     fn get<T>(&self) -> Option<&T>
///     where
///         T: 'static,
///     {
///         self.0.get(&TypeId::of::<T>())?
///             .downcast_ref::<T>()
///     }
///
///     fn get_mut<T>(&mut self) -> Option<&mut T>
///     where
///         T: 'static,
///     {
///         self.0.get_mut(&TypeId::of::<T>())?
///             .downcast_mut::<T>()
///     }
///
///     fn insert<T>(&mut self, val: T) -> Option<T>
///     where
///         T: Clone + Send + 'static,
///     {
///         self.0
///             .insert(TypeId::of::<T>(), Box::new(val))
///             .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
///     }
///
///     fn remove<T>(&mut self) -> Option<T>
///     where
///         T: 'static,
///     {
///         self.0
///             .remove(&TypeId::of::<T>())
///             .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
///     }
/// }
/// ```
pub trait LocalStorage {
    /// Gets reference of a value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::context::storage::{LocalStorage, local_storage::{Merge, MergeResult, LocalStorageImpl}};
    /// # type ExampleStorage = LocalStorageImpl;
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// struct ExampleValue(u8);
    /// impl Merge for ExampleValue // ...
    /// # {
    /// #     fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> { todo!() }
    /// # }
    /// let mut storage = ExampleStorage::new();
    ///
    /// storage.insert(ExampleValue(5u8));
    /// let result: Option<&ExampleValue> = storage.get();
    /// assert_eq!(result, Some(&ExampleValue(5u8)));
    /// let result: Option<&u16> = storage.get();
    /// assert_eq!(result, None);
    /// ```
    fn get<T>(&self) -> Option<&T>
    where
        T: 'static;

    /// Gets mutable reference of a value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::context::storage::{LocalStorage, local_storage::{Merge, MergeResult, LocalStorageImpl}};
    /// # type ExampleStorage = LocalStorageImpl;
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// struct ExampleValue(u8);
    /// impl Merge for ExampleValue // ...
    /// # {
    /// #     fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> { todo!() }
    /// # }
    /// let mut storage = ExampleStorage::new();
    ///
    /// storage.insert(ExampleValue(5u8));
    /// if let Some(val) = storage.get_mut::<ExampleValue>() {
    ///     val.0 = 15u8;
    /// }
    /// let result: Option<&ExampleValue> = storage.get();
    /// assert_eq!(result, Some(&ExampleValue(15u8)));
    /// ```
    fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static;

    /// Inserts value with type `T` to storage and returns the value that was there previously if it was there.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::context::storage::{LocalStorage, local_storage::{Merge, MergeResult, LocalStorageImpl}};
    /// # type ExampleStorage = LocalStorageImpl;
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// struct ExampleValue(u8);
    /// impl Merge for ExampleValue // ...
    /// # {
    /// #     fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> { todo!() }
    /// # }
    /// let mut storage = ExampleStorage::new();
    ///
    /// let result = storage.insert(ExampleValue(5u8));
    /// assert_eq!(result, None);
    /// storage.insert(ExampleValue(15u8));
    /// let result = storage.insert(ExampleValue(25u8));
    /// assert_eq!(result, Some(ExampleValue(15u8)));
    /// let result: Option<&ExampleValue> = storage.get();
    /// assert_eq!(result, Some(&ExampleValue(25u8)));
    /// ```
    fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: Merge + Clone + Send + 'static;

    /// Removes and returns value with type `T` from storage if it is present.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::context::storage::{LocalStorage, local_storage::{Merge, MergeResult, LocalStorageImpl}};
    /// # type ExampleStorage = LocalStorageImpl;
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// struct ExampleValue(u8);
    /// impl Merge for ExampleValue // ...
    /// # {
    /// #     fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> { todo!() }
    /// # }
    /// let mut storage = ExampleStorage::new();
    ///
    /// let result = storage.insert(ExampleValue(5u8));
    /// assert_eq!(result, None);
    /// let result = storage.remove();
    /// assert_eq!(result, Some(ExampleValue(5u8)));
    /// let result = storage.remove::<ExampleValue>();
    /// assert_eq!(result, None);
    /// ```
    fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static;
}

/// Represents the result of merging multiple instances of a type during context merging.
///
/// This enum is used by the [`Merge`] trait to determine how merging should affect the parent value.
/// It is up to the implementor to decide whether to keep, replace, or remove it entirely.
///
/// See also [`Merge`] trait.
#[derive(Debug)]
pub enum MergeResult<T> {
    /// Keep the existing parent value as-is, whether it exists or not.
    KeepParent,
    /// Replace the parent value or insert this value if it does not exist.
    ReplaceOrInsert(T),
    /// Remove the value from the parent context entirely.
    Remove,
}

/// Defines how multiple instances of a type are merged.
///
/// The `Merge` trait is used to combine several versions of a value into a single instance.
/// It is mainly used in a fork-join lifecycle.
///
/// Implementations should define the merging logic between an optional parent
/// value and a collection of child values.
///
/// # Examples
/// ```
/// use node_flow::context::storage::local_storage::{Merge, MergeResult};
///
/// struct Counter(u32);
///
/// impl Merge for Counter {
///     fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> {
///         let sum: u32 = others.iter().map(|c| c.0).sum();
///         let base = parent.map_or(0, |p| p.0);
///         MergeResult::ReplaceOrInsert(Counter(base + sum))
///     }
/// }
/// ```
pub trait Merge: Sized {
    /// Merges the parent value with a list of child values and returns a [`MergeResult`].
    ///
    /// # Parameters
    /// - `parent`: An optional reference to the existing value in the parent context.
    /// - `others`: A list of values to merge into the parent.
    ///
    /// # Returns
    /// A [`MergeResult`] indicating how the parent should be updated.
    fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self>;
}
