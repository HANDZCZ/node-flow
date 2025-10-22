pub trait Node<Input, Output, Error, Context> {
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
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
        impl<Context: Send>
            $crate::node::Node<$input, $crate::node::NodeOutput<$output>, $error, Context>
            for $node
        {
            async fn run(
                &mut self,
                input: $input,
                context: &mut Context,
            ) -> Result<$crate::node::NodeOutput<$output>, $error> {
                Ok($crate::node::NodeOutput::Ok(
                    <Self as $crate::node::Node<$input, $output, $error, Context>>::run(
                        self, input, context,
                    )
                    .await?,
                ))
            }
        }
    };
}
