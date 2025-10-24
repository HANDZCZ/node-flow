use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use async_lock::RwLock;
use futures_util::FutureExt;

use crate::context::{Fork, Join, Update, storage::shared_storage::SharedStorage};

type StorageItem = Arc<RwLock<Option<Box<dyn Any + Send + Sync>>>>;

#[derive(Default, Clone)]
pub struct SharedStorageImpl {
    inner: Arc<Mutex<HashMap<TypeId, StorageItem>>>,
}

impl Debug for SharedStorageImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedStorageImpl").finish_non_exhaustive()
    }
}

impl SharedStorageImpl {
    /// Constructs new `SharedStorageImpl`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SharedStorage for SharedStorageImpl {
    fn get<T>(&self) -> impl Future<Output = Option<impl Deref<Target = T>>> + Send
    where
        T: 'static,
    {
        let rw_lock = {
            let guard = self.inner.lock().unwrap();
            guard.get(&TypeId::of::<T>()).cloned()
        };

        async move {
            let rw_lock = rw_lock?;
            let rw_lock_guard = rw_lock.read_arc().await;
            if rw_lock_guard.is_none() {
                return None;
            }
            let read_guard = guards::ReadGuard {
                guard: rw_lock_guard,
                _item_type: std::marker::PhantomData,
            };

            Some(read_guard)
        }
    }

    fn get_mut<T>(&mut self) -> impl Future<Output = Option<impl DerefMut<Target = T>>> + Send
    where
        T: 'static,
    {
        let rw_lock = {
            let guard = self.inner.lock().unwrap();
            guard.get(&TypeId::of::<T>()).cloned()
        };

        async move {
            let rw_lock = rw_lock?;
            let rw_lock_guard = rw_lock.write_arc().await;
            if rw_lock_guard.is_none() {
                return None;
            }
            let write_guard = guards::WriteGuard {
                guard: rw_lock_guard,
                _item_type: std::marker::PhantomData,
            };

            Some(write_guard)
        }
    }

    fn insert<T>(&mut self, val: T) -> impl Future<Output = Option<T>> + Send
    where
        T: Send + Sync + 'static,
    {
        let rw_lock = {
            let mut guard = self.inner.lock().unwrap();
            match guard.entry(TypeId::of::<T>()) {
                Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(Arc::new(RwLock::new(Some(Box::new(val)))));
                    return futures_util::future::ready(None).left_future();
                }
            }
        };

        async move {
            let mut rw_lock_guard = rw_lock.write().await;
            let val = rw_lock_guard.replace(Box::new(val))?;
            let val = *val.downcast::<T>().unwrap();
            Some(val)
        }
        .right_future()
    }

    fn insert_with_if_absent<T, E>(
        &self,
        fut: impl Future<Output = Result<T, E>> + Send,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        T: Send + Sync + 'static,
        E: Send,
    {
        let mut guard = self.inner.lock().unwrap();
        match guard.entry(TypeId::of::<T>()) {
            Entry::Occupied(_) => futures_util::future::ready(Ok(())).left_future(),
            Entry::Vacant(vacant_entry) => {
                let rw_lock = Arc::new(RwLock::new(None));
                let mut rw_lock_guard = rw_lock.write_arc_blocking();
                vacant_entry.insert(rw_lock);
                async move {
                    let val = fut.await?;
                    *rw_lock_guard = Some(Box::new(val));
                    Ok(())
                }
                .right_future()
            }
        }
    }

    fn remove<T>(&mut self) -> impl Future<Output = Option<T>> + Send
    where
        T: 'static,
    {
        let rw_lock = {
            let mut guard = self.inner.lock().unwrap();
            guard.remove(&TypeId::of::<T>())
        };

        async move {
            let rw_lock = rw_lock?;
            let mut rw_lock_guard = rw_lock.write().await;
            let val = rw_lock_guard.take()?;
            let val = *val.downcast::<T>().unwrap();
            Some(val)
        }
    }
}

impl Fork for SharedStorageImpl {
    fn fork(&self) -> Self {
        self.clone()
    }
}

impl Update for SharedStorageImpl {
    fn update_from(&mut self, _other: Self) {}
}

impl Join for SharedStorageImpl {
    fn join(&mut self, _others: Box<[Self]>) {}
}

mod guards {
    use std::{
        any::Any,
        ops::{Deref, DerefMut},
    };

    pub struct ReadGuard<T: 'static> {
        pub guard: async_lock::RwLockReadGuardArc<Option<Box<dyn Any + Send + Sync>>>,
        pub _item_type: std::marker::PhantomData<T>,
    }

    impl<T: 'static> Deref for ReadGuard<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            let any_ref: &dyn Any = &**self.guard.as_ref().unwrap();
            any_ref.downcast_ref::<T>().unwrap()
        }
    }

    pub struct WriteGuard<T: 'static> {
        pub guard: async_lock::RwLockWriteGuardArc<Option<Box<dyn Any + Send + Sync>>>,
        pub _item_type: std::marker::PhantomData<T>,
    }

    impl<T: 'static> Deref for WriteGuard<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            let any_ref: &dyn Any = &**self.guard.as_ref().unwrap();
            any_ref.downcast_ref::<T>().unwrap()
        }
    }

    impl<T: 'static> DerefMut for WriteGuard<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            let any_ref: &mut dyn Any = &mut **self.guard.as_mut().unwrap();
            any_ref.downcast_mut::<T>().unwrap()
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

    #[tokio::test]
    async fn works() {
        let mut s = SharedStorageImpl::new();
        let _ = s.insert(MyVal("test".into()));
        //println!("{s:#?}");
        let v = s.get::<MyVal>().await;
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "test".to_string());

        let v = s.get_mut::<MyVal>().await;
        assert!(v.is_some());
        assert_eq!(v.as_ref().unwrap().0, "test".to_string());
        *v.unwrap() = MyVal("hmm".into());

        let v = s.insert(MyVal("jop".into())).await;
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "hmm".to_string());

        let v = s.remove::<MyVal>().await;
        assert!(v.is_some());
        assert_eq!(v.unwrap().0, "jop".to_string());
    }

    #[tokio::test]
    async fn test_merge() {
        let mut parent = SharedStorageImpl::new();
        let mut child1 = parent.fork();
        let _ = child1.insert(MyVal("bbb".to_owned())).await;
        let mut child2 = parent.fork();
        let _ = child2.insert(MyVal("ccc".to_owned())).await;
        let mut child3 = parent.fork();
        let _ = child3.insert(MyVal("ddd".to_owned())).await;
        parent.join(Box::new([child1, child2, child3]));
        let mut child = parent.fork();
        let _ = child.insert(MyVal("aaa".to_owned())).await;
        parent.join(Box::new([child]));

        let res = parent.get::<MyVal>().await;
        assert_eq!(res.unwrap().0, "aaa".to_owned());
    }
}
