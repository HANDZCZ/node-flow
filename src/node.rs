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

#[macro_export]
macro_rules! impl_node_output {
    ($node:ty, $input:ty, $output:ty, $error:ty) => {
        #[cfg_attr(not(all(doc, not(doctest))), $crate::async_trait::async_trait)]
        impl $crate::node::Node<$input, $crate::node::NodeOutput<$output>, $error> for $node {
            async fn run_with_storage<'input>(
                &mut self,
                input: $input,
                storage: &mut $crate::storage::Storage,
            ) -> Result<$crate::node::NodeOutput<$output>, $error> {
                Ok($crate::node::NodeOutput::Ok(
                    <Self as $crate::node::Node<$input, $output, $error>>::run_with_storage(
                        self, input, storage,
                    )
                    .await?,
                ))
            }
        }
    };
}
