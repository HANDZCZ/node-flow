use crate::describe::{Description, DescriptionBase};

pub trait Node<Input, Output, Error, Context> {
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
    ) -> impl Future<Output = Result<Output, Error>> + Send;

    // if specialization is ever stabilized this whole function can be removed
    // and Describe trait with default impl for Node<..> could be used
    // https://github.com/rust-lang/rust/issues/31844
    fn describe(&self) -> Description
    where
        Self: Sized,
    {
        let mut base = DescriptionBase::from::<Self, Input, Output, Error, Context>();

        // remove NodeOutput<> from output name
        let output_name = &mut base.output.name;
        if let Some(b_pos) = output_name.find('<')
            && output_name[..b_pos].contains("NodeOutput")
        {
            // remove `..::NodeOutput<`
            output_name.replace_range(0..=b_pos, "");
            // remove ending `>`
            output_name.pop();
        }

        Description::Node { base }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeOutput<T> {
    SoftFail,
    Ok(T),
}

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
