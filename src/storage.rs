use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

trait StorageItem: Any + Send {
    fn duplicate(&self) -> Box<dyn StorageItem>;
}

impl<T> StorageItem for T
where
    T: Any + Send + Clone,
{
    fn duplicate(&self) -> Box<dyn StorageItem> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn StorageItem> {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}

/// Storage that is used in [`Node`](crate::node::Node).
#[derive(Default)]
pub struct Storage {
    inner: HashMap<TypeId, Box<dyn StorageItem>>,
    changed: HashSet<TypeId>,
}

impl Storage {
    /// Constructs new `Storage`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets reference of a value with type `T` from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.inner.get(&TypeId::of::<T>()).map(|val| {
            let any_ref: &dyn Any = &**val;
            any_ref.downcast_ref::<T>().unwrap()
        })
    }

    /// Gets mutable reference of a value with type `T` from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any,
    {
        self.inner.get_mut(&TypeId::of::<T>()).map(|val| {
            self.changed.insert(TypeId::of::<T>());
            let any_debug_ref: &mut dyn Any = &mut **val;
            any_debug_ref.downcast_mut::<T>().unwrap()
        })
    }

    /// Inserts value with type `T` to storage and returns the value that was there previously if it was there.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: Any + Send + Clone,
    {
        self.changed.insert(TypeId::of::<T>());
        self.inner
            .insert(TypeId::of::<T>(), Box::new(val))
            .map(|val| *(val as Box<dyn Any>).downcast::<T>().unwrap())
    }

    /// Removes and returns value with type `T` from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Any,
    {
        self.inner.remove(&TypeId::of::<T>()).map(|val| {
            self.changed.insert(TypeId::of::<T>());
            *(val as Box<dyn Any>).downcast::<T>().unwrap()
        })
    }

    pub(crate) fn new_gen(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            changed: HashSet::new(),
        }
    }

    pub(crate) fn replace(&mut self, other: Self) {
        self.inner = other.inner;
        self.changed.extend(other.changed.iter());
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct MyVal(String);

    #[test]
    fn works() {
        let mut s = Storage::new();
        s.insert(MyVal("test".into()));
        //println!("{s:#?}");
        let v = s.get::<MyVal>();
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "test".to_string());

        let v = s.get_mut::<MyVal>();
        assert!(v.is_some());
        assert_eq!(v.as_ref().unwrap().0, "test".to_string());
        *v.unwrap() = MyVal("hmm".into());

        let v = s.insert(MyVal("jop".into()));
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "hmm".to_string());

        let v = s.remove::<MyVal>();
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "jop".to_string());
    }
}
