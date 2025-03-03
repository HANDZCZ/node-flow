use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Storage that is used in [`Node`](crate::node::Node).
#[derive(Default)]
pub struct Storage {
    inner: HashMap<TypeId, Box<dyn Any + Send>>,
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
        T: Any + Send,
    {
        self.inner.get(&TypeId::of::<T>()).map(|val| {
            let any_debug_ref: &(dyn Any + Send) = &**val;
            any_debug_ref.downcast_ref::<T>().unwrap()
        })
    }

    /// Gets mutable reference of a value with type T from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any + Send,
    {
        self.inner.get_mut(&TypeId::of::<T>()).map(|val| {
            let any_debug_ref: &mut (dyn Any + Send) = &mut **val;
            any_debug_ref.downcast_mut::<T>().unwrap()
        })
    }

    /// Inserts value with type T to storage and returns the value that was there previously if it was there.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: Any + Send,
    {
        self.inner
            .insert(TypeId::of::<T>(), Box::new(val))
            .map(|val: Box<dyn Any + Send>| *val.downcast::<T>().unwrap())
    }

    /// Removes and returns value with type T from storage if it is present.
    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Any + Send,
    {
        self.inner
            .remove(&TypeId::of::<T>())
            .map(|val: Box<dyn Any + Send>| *val.downcast::<T>().unwrap())
    }
}

#[test]
fn works() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MyVal(String);

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
