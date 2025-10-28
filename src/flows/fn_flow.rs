use std::fmt::Debug;

use crate::{
    describe::{Description, DescriptionBase, Edge, Type, remove_generics_from_name},
    flows::NodeResult,
    node::{Node, NodeOutput},
};

pub trait Runner<'a, Input, Output, Error, Context, InnerData>: Send + Sync {
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
    // TODO: mention signature issue in docs (&mut Context must be present, it needs to be async closure and inner data must also be present: async |data: ..., _, _: &mut Context| {...})
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
