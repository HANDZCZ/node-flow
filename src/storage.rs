use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

pub enum MergeResult<T> {
    KeepParent,
    ReplaceOrInsert(T),
    Remove,
}

pub trait Merge: Sized {
    fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self>;
}

trait StorageItem: Any + Send {
    fn duplicate(&self) -> Box<dyn StorageItem>;
    fn merge(
        &self,
        parent: Option<&dyn StorageItem>,
        others: Box<[Box<dyn StorageItem>]>,
    ) -> MergeResult<Box<dyn StorageItem>>;
}

impl<T> StorageItem for T
where
    T: Merge + Any + Send + Clone,
{
    fn duplicate(&self) -> Box<dyn StorageItem> {
        Box::new(self.clone())
    }

    // SAFETY: self can never be used otherwise it can lead to UB
    fn merge(
        &self,
        parent: Option<&dyn StorageItem>,
        others: Box<[Box<dyn StorageItem>]>,
    ) -> MergeResult<Box<dyn StorageItem>> {
        let others = others
            .into_iter()
            .map(|b| *(b as Box<dyn Any>).downcast::<T>().unwrap())
            .collect::<Box<_>>();
        let parent = parent.map(|v| (v as &dyn Any).downcast_ref::<T>().unwrap());
        match <T as Merge>::merge(parent, others) {
            MergeResult::ReplaceOrInsert(val) => MergeResult::ReplaceOrInsert(Box::new(val)),
            MergeResult::KeepParent => MergeResult::KeepParent,
            MergeResult::Remove => MergeResult::Remove,
        }
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
        T: Merge + Any + Send + Clone,
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

    /// Merges changed items in other storages into this one.
    ///
    /// Storages passed as others can be used later, but they will be missing all changed items
    /// (any item accessed through: [`get_mut`](Self::get_mut), [`remove`](Self::remove), [`insert`](Self::insert))!
    pub(crate) fn merge(&mut self, others: &mut [Self]) {
        let mut changed = std::mem::take(&mut others[0].changed);
        others
            .iter_mut()
            .skip(1)
            .for_each(|s| changed.extend(s.changed.drain()));

        for key in changed {
            let parent = self.inner.get(&key).map(Box::as_ref);
            let other_items = others
                .iter_mut()
                .filter_map(|s| s.inner.remove(&key))
                .collect::<Box<[_]>>();
            if other_items.is_empty() {
                continue;
            }
            let res = {
                // Call merge on dyn StorageItem type
                // All types (inside of a `parent` and `other_items[...]`) have the same type
                let dispatcher: &dyn StorageItem = match parent {
                    Some(p) => p,
                    None => &*other_items[0],
                };
                // SAFETY: reference is only used for VTable lookup, the self type is otherwise unused,
                //         this reference is then dropped and never used since it will most likely point to a non-existent data
                let dispatcher: &dyn StorageItem = unsafe { &*std::ptr::from_ref(dispatcher) };
                dispatcher.merge(parent, other_items)
            };
            match res {
                MergeResult::KeepParent => {}
                MergeResult::ReplaceOrInsert(val) => {
                    self.inner.insert(key, val);
                    self.changed.insert(key);
                }
                MergeResult::Remove => {
                    if self.inner.remove(&key).is_some() {
                        self.changed.insert(key);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    pub struct MyVal(pub String);

    impl Default for MyVal {
        fn default() -> Self {
            Self("|".to_owned())
        }
    }

    impl Merge for MyVal {
        fn merge(parent: Option<&Self>, others: Box<[Self]>) -> MergeResult<Self> {
            let len = parent.as_ref().map(|v| v.0.len()).unwrap_or_default()
                + others.iter().map(|v| v.0.len()).sum::<usize>();
            if len == 0 {
                return MergeResult::KeepParent;
            }
            let mut res = String::with_capacity(len);
            if let Some(v) = parent {
                res.push_str(&v.0);
            }
            for v in others {
                res.push_str(&v.0);
            }
            MergeResult::ReplaceOrInsert(MyVal(res))
        }
    }

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

    #[test]
    fn test_merge() {
        let mut parent = Storage::new();
        let mut child1 = parent.new_gen();
        child1.insert(MyVal("bbb".to_owned()));
        let mut child2 = parent.new_gen();
        child2.insert(MyVal("ccc".to_owned()));
        let mut child3 = parent.new_gen();
        child3.insert(MyVal("ddd".to_owned()));
        parent.merge(&mut [child1, child2, child3]);
        let mut child = parent.new_gen();
        child.insert(MyVal("aaa".to_owned()));
        parent.merge(&mut [child]);

        let res = parent.get::<MyVal>();
        assert_eq!(res.unwrap().0, "bbbcccdddaaa".to_owned());
    }
}
