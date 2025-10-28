use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::context::{
    Fork, Join, Update,
    storage::local_storage::{LocalStorage, Merge, MergeResult},
};

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

#[derive(Default)]
pub struct LocalStorageImpl {
    inner: HashMap<TypeId, Box<dyn StorageItem>>,
    changed: HashSet<TypeId>,
}

impl Debug for LocalStorageImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalStorageImpl").finish_non_exhaustive()
    }
}

impl LocalStorageImpl {
    /// Constructs new `LocalStorageImpl`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl LocalStorage for LocalStorageImpl {
    fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.inner.get(&TypeId::of::<T>()).map(|val| {
            let any_ref: &dyn Any = &**val;
            any_ref.downcast_ref::<T>().unwrap()
        })
    }

    fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.inner.get_mut(&TypeId::of::<T>()).map(|val| {
            self.changed.insert(TypeId::of::<T>());
            let any_debug_ref: &mut dyn Any = &mut **val;
            any_debug_ref.downcast_mut::<T>().unwrap()
        })
    }

    fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: Merge + Clone + Send + 'static,
    {
        self.changed.insert(TypeId::of::<T>());
        self.inner
            .insert(TypeId::of::<T>(), Box::new(val))
            .map(|val| *(val as Box<dyn Any>).downcast::<T>().unwrap())
    }

    fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static,
    {
        self.inner.remove(&TypeId::of::<T>()).map(|val| {
            self.changed.insert(TypeId::of::<T>());
            *(val as Box<dyn Any>).downcast::<T>().unwrap()
        })
    }
}

impl Fork for LocalStorageImpl {
    fn fork(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            changed: HashSet::new(),
        }
    }
}

impl Update for LocalStorageImpl {
    fn update_from(&mut self, other: Self) {
        self.inner = other.inner;
        self.changed.extend(other.changed.iter());
    }
}

impl Join for LocalStorageImpl {
    fn join(&mut self, mut others: Box<[Self]>) {
        if others.is_empty() {
            return;
        }

        // gather TypeId of all changed items
        let mut changed = others[0].changed.clone();
        others
            .iter_mut()
            .skip(1)
            .for_each(|s| changed.extend(s.changed.iter()));

        for key in changed {
            // collect items from self and from other_items if the item was changed
            let parent = self.inner.get(&key).map(Box::as_ref);
            let other_items = others
                .iter_mut()
                .filter_map(|s| {
                    s.changed
                        .remove(&key)
                        .then(|| s.inner.remove(&key))
                        .flatten()
                })
                .collect::<Box<[_]>>();

            // decide if and how the items are merged
            // allow match_same_arms for comments
            #[expect(clippy::match_same_arms)]
            match (parent.is_none(), other_items.is_empty()) {
                // parent and other_items are empty
                //     => item was inserted in a branch and then removed
                // = skip item
                (true, true) => continue,
                // parent is empty and other_items contain exactly one item
                //     => item was inserted in exactly one branch
                //  or => item was inserted in multiple branches, but later it was removed from all but one branch
                // = insert first and only item
                (true, false) if other_items.len() == 1 => {
                    let first = other_items.into_iter().next().unwrap();
                    self.inner.insert(key, first);
                    self.changed.insert(key);
                    continue;
                }
                // parent is empty and other_items contain more than one item
                //     => more than one branch inserted item
                // = merge needed
                (true, false) => {}
                // parent is not empty and other_items is empty
                //     => item was removed in all branches
                // = remove item
                (false, true) => {
                    self.inner.remove(&key);
                    self.changed.insert(key);
                    continue;
                }
                // parent and other_items are not empty
                //     => at least one branch inserted item
                // = merge needed
                (false, false) => {}
            }

            // Merge trait is needed for merging
            let res = {
                // All types (inside of a `parent` and `other_items[...]`) have the same type
                let dispatcher: &dyn StorageItem = parent.map_or_else(|| &*other_items[0], |p| p);

                // Call merge on dyn StorageItem type
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
        let mut s = LocalStorageImpl::new();
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
        let mut parent = LocalStorageImpl::new();
        let mut child1 = parent.fork();
        child1.insert(MyVal("bbb".to_owned()));
        let mut child2 = parent.fork();
        child2.insert(MyVal("ccc".to_owned()));
        let mut child3 = parent.fork();
        child3.insert(MyVal("ddd".to_owned()));
        parent.join(Box::new([child1, child2, child3]));
        let mut child = parent.fork();
        child.insert(MyVal("aaa".to_owned()));
        parent.join(Box::new([child]));

        let res = parent.get::<MyVal>();
        assert_eq!(res.unwrap().0, "bbbcccdddaaa".to_owned());
    }
}
