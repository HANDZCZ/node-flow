use crate::storage::Storage;

pub trait Node<Input, Output, Error> {
    fn run_with_storage(
        &mut self,
        input: Input,
        storage: &mut Storage,
    ) -> impl Future<Output = Result<Output, Error>> + Send;
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeOutput<T> {
    SoftFail,
    Ok(T),
}

#[macro_export]
macro_rules! impl_node_output {
    ($node:ty, $input:ty, $output:ty, $error:ty) => {
        impl $crate::node::Node<$input, $crate::node::NodeOutput<$output>, $error> for $node {
            async fn run_with_storage(
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
