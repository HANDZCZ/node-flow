use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// Storage that is used in [`Node`](crate::node::Node).
#[derive(Default)]
pub struct Storage {
    inner: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    changed: HashSet<TypeId>,
}

impl Storage {
    /// Constructs new `Storage`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets reference of a value with type T from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.inner.get(&TypeId::of::<T>()).map(|val| {
            let any_ref: &(dyn Any + Send + Sync) = Arc::as_ref(val);
            any_ref.downcast_ref::<T>().unwrap()
        })
    }

    /// Inserts value with type T to storage.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn insert<T>(&mut self, val: T)
    where
        T: Any + Send + Sync,
    {
        let t_typeid = TypeId::of::<T>();
        self.inner.insert(t_typeid, Arc::new(val));
        self.changed.insert(t_typeid);
    }

    /// Removes value with type T from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn remove<T>(&mut self)
    where
        T: 'static,
    {
        let t_typeid = TypeId::of::<T>();
        if self.inner.remove(&t_typeid).is_some() {
            self.changed.insert(t_typeid);
        }
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

#[test]
fn works() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MyVal(String);

    let mut s = Storage::new();
    s.insert(MyVal("test".into()));
    let v = s.get::<MyVal>();
    assert!(v.is_some());
    assert_eq!(v.unwrap().0, "test".to_string());

    s.insert(MyVal("jop".into()));
    let v = s.get::<MyVal>();
    assert!(v.is_some());
    assert_eq!(v.unwrap().0, "jop".to_string());

    s.remove::<MyVal>();
    let v = s.get::<MyVal>();
    assert!(v.is_none());
}
