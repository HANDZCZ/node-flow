#[macro_export]
macro_rules! impl_node_output {
    ($node:ty, $input:ty, $output:ty, $error:ty $(,$param:ident: $bound0:ident $(+$bound:ident)*)*) => {
        impl<Context>
            $crate::node::Node<$input, $crate::node::NodeOutput<$output>, $error, Context>
            for $node
        where
            Context: Send,
            $($param: $bound0 $(+$bound)*,)*
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

            fn describe(&self) -> $crate::describe::Description {
                <Self as $crate::node::Node<$input, $output, $error, Context>>::describe(self)
            }
        }
    };
}

#[macro_export]
macro_rules! node {
    ($input:ty, !$output:ty, $error:ty, $context:ty) => {
        impl $crate::node::Node<$input, $output, $error, $context> + Clone + Send + Sync
    };
    ($input:ty, $output:ty, $error:ty, $context:ty) => {
        $crate::node!($input, !$crate::node::NodeOutput<$output>, $error, $context)
    };
}
