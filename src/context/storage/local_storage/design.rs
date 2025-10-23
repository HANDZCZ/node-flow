pub trait LocalStorage {
    /// Gets reference of a value with type `T` from storage if it is present.
    fn get<T>(&self) -> Option<&T>
    where
        T: 'static;

    /// Gets mutable reference of a value with type `T` from storage if it is present.
    fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static;

    /// Inserts value with type `T` to storage and returns the value that was there previously if it was there.
    fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: Merge + Clone + Send + 'static;

    /// Removes and returns value with type `T` from storage if it is present.
    fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static;
}

#[derive(Debug)]
pub enum MergeResult<T> {
    KeepParent,
    ReplaceOrInsert(T),
    Remove,
}

pub trait Merge: Sized {
    fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self>;
}
