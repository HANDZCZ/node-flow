use crate::{describe::Description, node::Node};

#[async_trait::async_trait]
pub trait BoxedNode<Input, Output, Error, Context> {
    async fn run_boxed(&mut self, input: Input, context: &mut Context) -> Result<Output, Error>
    where
        Input: 'async_trait,
        Output: 'async_trait,
        Error: 'async_trait;

    fn describe(&self) -> Description;
}

impl<Input, Output, Error, Context, T> BoxedNode<Input, Output, Error, Context> for T
where
    T: Node<Input, Output, Error, Context>,
{
    fn run_boxed<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        input: Input,
        context: &'life1 mut Context,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Output, Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Input: 'async_trait,
        Output: 'async_trait,
        Error: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(<Self as Node<Input, Output, Error, Context>>::run(
            self, input, context,
        ))
    }

    fn describe(&self) -> Description {
        <Self as Node<Input, Output, Error, Context>>::describe(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::node::NodeOutput;

    use super::BoxedNode;

    #[tokio::test]
    async fn works() {
        let n = crate::flows::tests::Passer::<u8, u16, ()>::new();
        let mut b: Box<dyn BoxedNode<_, _, _, _>> = Box::new(n);
        let res = b.run_boxed(5u8, &mut ()).await;
        assert_eq!(res, Ok(NodeOutput::Ok(5u16)));
    }
}
