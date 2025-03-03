use crate::storage::Storage;
use async_trait::async_trait;

#[cfg_attr(not(all(doc, not(doctest))), async_trait)]
pub trait Node<Input, Output, Error> {
    async fn run_with_storage<'input>(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> Result<Output, Error>
    where
        Input: 'input;
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeOutput<T> {
    SoftFail,
    Ok(T),
}
