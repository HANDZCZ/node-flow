use std::{marker::PhantomData, task::Poll};

use crate::{
    node::{Node, NodeOutput},
    storage::Storage,
};

pub fn poll_once<Fut, Output>(fut: Fut) -> Poll<Output>
where
    Fut: Future<Output = Output>,
{
    let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
    Future::poll(Box::pin(fut).as_mut(), &mut ctx)
}

#[derive(Clone)]
pub struct Passer<I, O, E>(PhantomData<(I, O, E)>);

impl<I, O, E> Passer<I, O, E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I, O, E> Node<I, NodeOutput<O>, E> for Passer<I, O, E>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
{
    async fn run_with_storage(
        &mut self,
        input: I,
        _storage: &mut Storage,
    ) -> Result<NodeOutput<O>, E> {
        Ok(NodeOutput::Ok(input.into()))
    }
}

#[derive(Clone)]
pub struct SoftFailNode<I, O, E>(PhantomData<(I, O, E)>);

impl<I, O, E> SoftFailNode<I, O, E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<I, O, E> Node<I, NodeOutput<O>, E> for SoftFailNode<I, O, E>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
{
    async fn run_with_storage(
        &mut self,
        _input: I,
        _storage: &mut Storage,
    ) -> Result<NodeOutput<O>, E> {
        Ok(NodeOutput::SoftFail)
    }
}

#[derive(Clone)]
pub struct InsertIntoStorageAssertWasNotInStorage<I, O, E, T>(PhantomData<(I, O, E, T)>);

impl<I, O, E, T> InsertIntoStorageAssertWasNotInStorage<I, O, E, T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<I, O, E, T> Node<I, NodeOutput<O>, E> for InsertIntoStorageAssertWasNotInStorage<I, O, E, T>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
    T: Default + Clone + Send + 'static,
{
    async fn run_with_storage(
        &mut self,
        _input: I,
        storage: &mut Storage,
    ) -> Result<NodeOutput<O>, E> {
        assert!(
            storage.insert(T::default()).is_none(),
            "{} was in storage",
            std::any::type_name::<T>()
        );
        Ok(NodeOutput::SoftFail)
    }
}
