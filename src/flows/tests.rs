use std::marker::PhantomData;

use crate::{
    context::storage::local_storage::{LocalStorage, Merge},
    node::{Node, NodeOutput},
};

#[derive(Clone)]
pub struct Passer<I, O, E>(PhantomData<(I, O, E)>);

impl<I, O, E> Passer<I, O, E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I, O, E, C> Node<I, NodeOutput<O>, E, C> for Passer<I, O, E>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
    C: Send,
{
    async fn run(&mut self, input: I, _context: &mut C) -> Result<NodeOutput<O>, E> {
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
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
impl<I, O, E, C> Node<I, NodeOutput<O>, E, C> for SoftFailNode<I, O, E>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
    C: Send,
{
    async fn run(&mut self, _input: I, _context: &mut C) -> Result<NodeOutput<O>, E> {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
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
impl<I, O, E, T, C> Node<I, NodeOutput<O>, E, C>
    for InsertIntoStorageAssertWasNotInStorage<I, O, E, T>
where
    I: Into<O> + Send,
    O: Send,
    E: Send,
    T: Default + Merge + Clone + Send + 'static,
    C: LocalStorage + Send,
{
    async fn run(&mut self, _input: I, context: &mut C) -> Result<NodeOutput<O>, E> {
        assert!(
            context.insert(T::default()).is_none(),
            "{} was in storage",
            std::any::type_name::<T>()
        );
        Ok(NodeOutput::SoftFail)
    }
}
