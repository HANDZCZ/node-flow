use std::{fmt::Debug, vec};

use crate::{
    context::{Fork, SpawnAsync},
    describe::{Description, Edge, remove_generics_from_name},
    flows::NodeResult,
    node::{Node, NodeOutput as NodeOutputStruct},
};

pub struct Detached<Input, Error, Context, NodeType = (), NodeOutput = (), NodeError = ()> {
    #[expect(clippy::type_complexity)]
    _iec: std::marker::PhantomData<fn() -> (Input, Error, Context)>,
    _node_oe: std::marker::PhantomData<fn() -> (NodeOutput, NodeError)>,
    node: std::sync::Arc<NodeType>,
}

impl<Input, Error, Context> Detached<Input, Error, Context> {
    pub fn new<NodeType, NodeOutput, NodeError>(
        node: NodeType,
    ) -> Detached<Input, Error, Context, NodeType, NodeOutput, NodeError>
    where
        NodeType: Node<Input, NodeOutput, NodeError, Context>,
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
        let aa = crate::describe::D2Describer::new().format(&flow.describe());
        println!("{aa}");

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
