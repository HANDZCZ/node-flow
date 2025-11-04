use std::{fmt::Debug, vec};

use crate::{
    context::{Fork, SpawnAsync},
    describe::{Description, Edge, remove_generics_from_name},
    flows::NodeResult,
    node::{Node, NodeOutput as NodeOutputStruct},
};

/// `Detached` executes a node **asynchronously and independently** of the main flow.
///
/// The node is executed in a **detached task** using the [`SpawnAsync`]
/// context trait and any result or error from the detached node is ignored.
///
/// This flow is useful for side-effect operations such as logging, analytics, or background
/// triggers that should not block or influence the main execution path.
///
/// # Type Parameters
/// - `Input`: The type of data **accepted and produced** by this flow.
/// - `Error`: The type of error emitted by this flow.
/// - `Context`: The type of context used during execution.
///
/// # Examples
/// ```
/// use node_flow::node::{Node, NodeOutput};
/// use node_flow::context::{SpawnAsync, Fork};
/// # use node_flow::context::Task;
/// use node_flow::flows::Detached;
/// use std::future::Future;
///
/// #[derive(Clone)]
/// struct PrintNode;
///
/// struct ExampleCtx;
/// impl Fork for ExampleCtx // ...
/// # { fn fork(&self) -> Self { Self } }
/// impl SpawnAsync for ExampleCtx // ...
/// # {
/// #    fn spawn<F>(fut: F) -> impl Task<F::Output>
/// #     where
/// #         F: Future + Send + 'static,
/// #         F::Output: Send + 'static,
/// #     {
/// #         DummyTask(std::marker::PhantomData)
/// #     }
/// # }
/// # struct DummyTask<T>(std::marker::PhantomData<T>);
/// # impl<T> Future for DummyTask<T> {
/// #     type Output = T;
/// #     fn poll(
/// #         self: std::pin::Pin<&mut Self>,
/// #         _: &mut std::task::Context<'_>
/// #     ) -> std::task::Poll<Self::Output> {
/// #         std::task::Poll::Pending
/// #     }
/// # }
/// # impl<T> Task<T> for DummyTask<T> {
/// #     fn is_finished(&self) -> bool { false }
/// #     fn cancel(self) {}
/// # }
///
/// impl<Ctx: Send> Node<u8, NodeOutput<()>, (), Ctx> for PrintNode {
///     async fn run(&mut self, input: u8, _: &mut Ctx) -> Result<NodeOutput<()>, ()> {
///         println!("Running detached task with input: {input}");
///         Ok(NodeOutput::Ok(()))
///     }
/// }
///
/// # tokio::runtime::Builder::new_current_thread()
/// #     .enable_all()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// async fn main() {
///     let mut detached = Detached::<u8, (), _>::new(PrintNode);
///
///     let mut ctx = ExampleCtx;
///     let result = detached.run(7, &mut ctx).await;
///     assert_eq!(result, Ok(NodeOutput::Ok(7)));
/// }
/// # main().await;
/// # });
/// ```
pub struct Detached<Input, Error, Context, NodeType = (), NodeOutput = (), NodeError = ()> {
    #[expect(clippy::type_complexity)]
    _iec: std::marker::PhantomData<fn() -> (Input, Error, Context)>,
    _node_oe: std::marker::PhantomData<fn() -> (NodeOutput, NodeError)>,
    node: std::sync::Arc<NodeType>,
}

impl<Input, Error, Context> Detached<Input, Error, Context> {
    /// Creates a new [`Detached`] flow by wrapping the given node.
    ///
    /// See also [`Detached`].
    ///
    /// # Examples
    /// ```
    /// use node_flow::flows::Detached;
    /// use node_flow::node::{Node, NodeOutput};
    /// # use node_flow::context::{SpawnAsync, Fork};
    /// # use node_flow::context::Task;
    /// # use std::future::Future;
    ///
    /// #[derive(Clone)]
    /// struct BackgroundTask;
    /// impl<Ctx: Send> Node<(), NodeOutput<()>, (), Ctx> for BackgroundTask // ...
    /// # {
    /// #     async fn run(&mut self, _: (), _: &mut Ctx) -> Result<NodeOutput<()>, ()> {
    /// #         todo!()
    /// #     }
    /// # }
    /// # struct Ctx;
    /// # impl Fork for Ctx { fn fork(&self) -> Self { Self } }
    /// # impl SpawnAsync for Ctx {
    /// #    fn spawn<F>(fut: F) -> impl Task<F::Output>
    /// #     where
    /// #         F: Future + Send + 'static,
    /// #         F::Output: Send + 'static,
    /// #     {
    /// #         DummyTask(std::marker::PhantomData)
    /// #     }
    /// # }
    /// # struct DummyTask<T>(std::marker::PhantomData<T>);
    /// # impl<T> Future for DummyTask<T> // ...
    /// # {
    /// #     type Output = T;
    /// #     fn poll(
    /// #         self: std::pin::Pin<&mut Self>,
    /// #         _: &mut std::task::Context<'_>
    /// #     ) -> std::task::Poll<Self::Output> {
    /// #         std::task::Poll::Pending
    /// #     }
    /// # }
    /// # impl<T> Task<T> for DummyTask<T> {
    /// #     fn is_finished(&self) -> bool { false }
    /// #     fn cancel(self) {}
    /// # }
    ///
    /// let detached = Detached::<(), (), Ctx>::new(BackgroundTask);
    /// ```
    #[expect(clippy::type_repetition_in_bounds)]
    pub fn new<NodeType, NodeOutput, NodeError>(
        node: NodeType,
    ) -> Detached<Input, Error, Context, NodeType, NodeOutput, NodeError>
    where
        NodeType: Node<Input, NodeOutput, NodeError, Context>,
        // Trait bounds for better and nicer errors
        NodeType: Clone + Send,
        Input: Clone + Send,
    {
        Detached {
            _iec: std::marker::PhantomData,
            _node_oe: std::marker::PhantomData,
            node: std::sync::Arc::new(node),
        }
    }
}

impl<Input, Error, Context, NodeType, NodeOutput, NodeError> Debug
    for Detached<Input, Error, Context, NodeType, NodeOutput, NodeError>
where
    NodeType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Detached")
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl<Input, Error, Context, NodeType, NodeOutput, NodeError> Clone
    for Detached<Input, Error, Context, NodeType, NodeOutput, NodeError>
{
    fn clone(&self) -> Self {
        Self {
            _iec: std::marker::PhantomData,
            _node_oe: std::marker::PhantomData,
            node: self.node.clone(),
        }
    }
}

impl<Input, Error, Context, NodeType, NodeOutput, NodeError>
    Node<Input, NodeOutputStruct<Input>, Error, Context>
    for Detached<Input, Error, Context, NodeType, NodeOutput, NodeError>
where
    NodeType: Node<Input, NodeOutput, NodeError, Context> + Clone + Send + 'static,
    Context: SpawnAsync + Fork + Send + 'static,
    Input: Clone + Send + 'static,
{
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
    ) -> impl Future<Output = NodeResult<Input, Error>> + Send {
        let _task = Context::spawn({
            let mut node = self.node.as_ref().clone();
            let input = input.clone();
            let mut context = context.fork();
            async move {
                let _ = node.run(input, &mut context).await;
            }
        });
        async { Ok(NodeOutputStruct::Ok(input)) }
    }

    fn describe(&self) -> Description {
        Description::new_flow(
            self,
            vec![self.node.describe()],
            vec![Edge::passthrough(), Edge::flow_to_node(0)],
        )
        .modify_name(remove_generics_from_name)
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, Instant};

    use super::Detached;
    use crate::{
        context::{Fork, test::TokioSpawner},
        node::{Node, NodeOutput},
    };

    impl Fork for TokioSpawner {
        fn fork(&self) -> Self {
            Self
        }
    }

    #[derive(Clone)]
    pub struct TestNode(tokio::sync::mpsc::Sender<()>);

    impl<I, C> Node<I, (), (), C> for TestNode
    where
        I: Send,
        C: Send,
    {
        async fn run(&mut self, _input: I, _context: &mut C) -> Result<(), ()> {
            tokio::time::sleep(Duration::from_millis(20)).await;
            self.0.send(()).await.unwrap();
            Err(())
        }
    }

    #[tokio::test]
    async fn test_flow() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(5);
        let mut ctx = TokioSpawner;
        let mut flow = Detached::<_, (), _>::new(TestNode(sender));

        let sleep = tokio::time::sleep(Duration::from_millis(50));
        tokio::pin!(sleep);
        let start = Instant::now();
        let res = flow.run(3u8, &mut ctx).await;
        let flow_end = Instant::now();

        tokio::select! {
            _ = receiver.recv() => {}
            _ = &mut sleep => {
                panic!("timeout");
            }
        };
        let end = Instant::now();

        let flow_took = flow_end.duration_since(start);
        let node_took = end.duration_since(start);
        println!("flow_took: {flow_took:?}");
        println!("node_took: {node_took:?}");
        assert_eq!(res, Ok(NodeOutput::Ok(3)));
        assert!(flow_took.as_millis() < 1);
        assert!(node_took.as_millis() > 15);
    }
}
