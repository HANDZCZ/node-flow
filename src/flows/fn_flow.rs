use std::fmt::Debug;

use crate::{
    describe::{Description, DescriptionBase, Edge, Type, remove_generics_from_name},
    flows::NodeResult,
    node::{Node, NodeOutput},
};

/// The `Runner` is used in [`FnFlow`] and is basically a replacement for [`Node`].
///
/// It defines how `Input` and inner data is processed into an `Output` (and `Error`) with a given `Context`.
///
/// See also [`FnFlow`], [`Node`].
pub trait Runner<'a, Input, Output, Error, Context, InnerData>: Send + Sync {
    /// Executes the runner using the provided inner data, input, and context.
    ///
    /// # Parameters
    /// - `data`: Inner data used in this runner.
    /// - `input`: The input data to process.
    /// - `context`: Mutable reference to the a context.
    fn run(
        &self,
        data: InnerData,
        input: Input,
        context: &'a mut Context,
    ) -> impl Future<Output = NodeResult<Output, Error>> + Send;
}

impl<'a, Input, Output, Error, Context, InnerData, T, F>
    Runner<'a, Input, Output, Error, Context, InnerData> for T
where
    Input: Send,
    F: Future<Output = NodeResult<Output, Error>> + Send + 'a,
    T: Fn(InnerData, Input, &'a mut Context) -> F + Send + Sync,
    Context: 'a,
{
    fn run(
        &self,
        data: InnerData,
        input: Input,
        context: &'a mut Context,
    ) -> impl Future<Output = NodeResult<Output, Error>> {
        (self)(data, input, context)
    }
}

/// `FnFlow` takes some async function and wraps around it to crate a node.
///
/// This flow allows for setting custom [`Description`]
/// through the [`FnFlow::with_description`] function,
/// since there is no other way to get it (like with [`Node::describe`]).
///
/// # Type Parameters
/// - `Input`: The type of data accepted by this flow.
/// - `Output`: The type of data produced by this flow.
/// - `Error`: The type of error emitted by this flow.
/// - `Context`: The type of context used during execution.
///
/// # Examples
/// ```
/// use node_flow::flows::FnFlow;
/// use node_flow::node::{Node, NodeOutput};
///
/// #[derive(Clone)]
/// struct SomeExpensiveData(Vec<u32>);
///
/// # tokio::runtime::Builder::new_current_thread()
/// #     .enable_all()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// async fn main() {
///     let mut flow = FnFlow::<u32, u32, (), _>::new(
///         SomeExpensiveData((0..1<<15).collect()),
///         async |SomeExpensiveData(data), input, _: &mut _| {
///             let res = data.iter().sum::<u32>() / data.len() as u32 + input;
///             Ok(NodeOutput::Ok(res))
///         },
///     );
///
///     let result = flow.run(1, &mut ()).await;
///     assert_eq!(result, Ok(NodeOutput::Ok(1<<14)));
/// }
/// # main().await;
/// # });
/// ```
pub struct FnFlow<Input, Output, Error, Context, InnerData = (), R = ()> {
    #[expect(clippy::type_complexity)]
    _ioec: std::marker::PhantomData<fn() -> (Input, Output, Error, Context)>,
    inner_data: std::sync::Arc<InnerData>,
    runner_description: Option<std::sync::Arc<Description>>,
    runner: R,
}
impl<Input, Output, Error, Context> FnFlow<Input, Output, Error, Context>
where
    // Trait bounds for better and nicer errors
    Input: Send,
    Context: Send,
{
    /// Creates a new [`FnFlow`] from a given runner.
    ///
    /// The runner must satisfy:
    /// - `Self`: `Runner<'_, Input, Output, Error, _, InnerData>`
    ///
    /// When using closure as a runner it always needs:
    /// - to be an **async closure** - because of lifetimes
    /// - *for context to*:
    ///     - have the **type of a context** specified when **using** context - because it cannot infer the type\
    ///       *or*
    ///     - have the context specified as `_: &mut _` when **not using** context - because it cannot satisfy that `Runner` is implemented
    /// - to have the **type of inner data** specified when **using** inner data - because it cannot infer the type
    ///
    /// # Examples
    /// ```
    /// # use node_flow::flows::FnFlow;
    /// # use node_flow::node::{Node, NodeOutput};
    /// # use node_flow::context::Fork;
    /// # #[derive(Clone)]
    /// struct SomeData(u16);
    /// # struct Context;
    /// # impl Fork for Context { fn fork(&self) -> Self { Self } }
    ///
    /// FnFlow::<u8, u16, (), Context>::new(
    ///     SomeData(15),
    ///     async |_, _, _: &mut _| {
    ///         Ok(NodeOutput::Ok(30))
    ///     },
    /// );
    /// FnFlow::<u8, u16, (), Context>::new(
    ///     SomeData(15),
    ///     async |data: SomeData, _, _: &mut _| {
    ///         Ok(NodeOutput::Ok(data.0 + 30))
    ///     },
    /// );
    /// FnFlow::<u8, u16, (), Context>::new(
    ///     SomeData(15),
    ///     async |_, _, ctx: &mut Context| {
    ///         let _forked_ctx = ctx.fork();
    ///         Ok(NodeOutput::Ok(30))
    ///     },
    /// );
    /// ```
    pub fn new<InnerData, R>(
        inner_data: InnerData,
        runner: R,
    ) -> FnFlow<Input, Output, Error, Context, InnerData, R>
    where
        InnerData: Clone + Send + Sync,
        for<'a> R: Runner<'a, Input, Output, Error, Context, InnerData>,
    {
        FnFlow {
            _ioec: std::marker::PhantomData,
            inner_data: std::sync::Arc::new(inner_data),
            runner_description: None,
            runner,
        }
    }
}

impl<Input, Output, Error, Context, InnerData, R>
    FnFlow<Input, Output, Error, Context, InnerData, R>
{
    /// Attaches a custom [`Description`] to this flow.
    ///
    /// This is be useful when something complex happens in the flow.
    ///
    /// # Examples
    /// ```
    /// use node_flow::flows::FnFlow;
    /// use node_flow::describe::{Description, DescriptionBase};
    /// use node_flow::node::NodeOutput;
    ///
    /// let desc = Description::Node {
    ///     base: DescriptionBase::from::<(), String, usize, u8, u32>()
    /// }.with_description("I am lying about my types! But shh...");
    /// let flow = FnFlow::<u8, u16, (), ()>::new((), async|_, x, _: &mut _| {
    ///     Ok(NodeOutput::Ok(x as u16))
    /// }).with_description(desc);
    /// ```
    #[must_use]
    pub fn with_description(mut self, description: Description) -> Self {
        self.runner_description = Some(std::sync::Arc::new(description));
        self
    }
}

impl<Input, Output, Error, Context, InnerData, R> Debug
    for FnFlow<Input, Output, Error, Context, InnerData, R>
where
    InnerData: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnFlow")
            .field("inner_data", &self.inner_data)
            .finish_non_exhaustive()
    }
}

impl<Input, Output, Error, Context, InnerData, R> Clone
    for FnFlow<Input, Output, Error, Context, InnerData, R>
where
    R: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _ioec: std::marker::PhantomData,
            inner_data: self.inner_data.clone(),
            runner_description: self.runner_description.clone(),
            runner: self.runner.clone(),
        }
    }
}

impl<Input, Output, Error, Context, InnerData, R> Node<Input, NodeOutput<Output>, Error, Context>
    for FnFlow<Input, Output, Error, Context, InnerData, R>
where
    InnerData: Clone,
    for<'a> R: Runner<'a, Input, Output, Error, Context, InnerData>,
{
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
    ) -> impl Future<Output = Result<NodeOutput<Output>, Error>> + Send {
        self.runner
            .run(self.inner_data.as_ref().clone(), input, context)
    }

    fn describe(&self) -> crate::describe::Description {
        if let Some(desc) = self.runner_description.as_ref() {
            return desc.as_ref().clone();
        }

        let runner = Description::Node {
            base: DescriptionBase {
                r#type: Type {
                    name: "Runner".to_owned(),
                },
                input: Type {
                    name: String::new(),
                },
                output: Type {
                    name: String::new(),
                },
                error: Type::of::<Error>(),
                context: Type::of::<Context>(),
                description: None,
                externals: None,
            },
        };

        let inner_data = Description::Node {
            base: DescriptionBase {
                r#type: Type::of::<InnerData>(),
                input: Type {
                    name: String::new(),
                },
                output: Type {
                    name: String::new(),
                },
                error: Type {
                    name: String::new(),
                },
                context: Type {
                    name: String::new(),
                },
                description: None,
                externals: None,
            },
        };

        Description::new_flow(
            self,
            vec![runner, inner_data],
            vec![
                Edge::flow_to_node(0),
                Edge::node_to_flow(0),
                Edge::node_to_node(1, 0),
            ],
        )
        .modify_name(remove_generics_from_name)
    }
}

#[cfg(test)]
mod test {
    use super::FnFlow as Flow;
    use crate::{
        context::storage::{
            LocalStorage,
            local_storage::{LocalStorageImpl, tests::MyVal},
        },
        node::{Node, NodeOutput},
    };

    #[tokio::test]
    async fn test_flow() {
        let mut st = LocalStorageImpl::new();
        let mut flow = Flow::<u8, u64, (), _>::new(
            (5u8, "aaa".to_owned(), 12u32),
            async |data: (u8, String, u32), input: u8, context: &mut LocalStorageImpl| {
                context.insert(MyVal::default());
                Ok(NodeOutput::Ok(
                    data.0 as u64 + data.1.len() as u64 + data.2 as u64 + input as u64,
                ))
            },
        );
        let res = flow.run(3, &mut st).await;
        assert_eq!(res, Ok(NodeOutput::Ok(23)));
    }
}
