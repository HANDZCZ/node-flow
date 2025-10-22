/// Defines flow with specified `ChainRun`, additional bounds and doc comments
macro_rules! define_flow {
    ($flow_name:ident, $chain_run:ident $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        define_flow!($flow_name, Builder, $chain_run $(,$param: $bound0 $(+$bound)*)* $(#[doc = $doc])*);
    };
    ($flow_name:ident, $builder:ident, $chain_run:ident $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        $(#[doc = $doc])*
        pub struct $flow_name<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()> {
            pub(super) _ioec: std::marker::PhantomData<fn() -> (Input, Output, Error, Context)>,
            pub(super) _nodes_io: std::marker::PhantomData<fn() -> NodeIOETypes>,
            pub(super) nodes: std::sync::Arc<NodeTypes>,
        }

        impl<Input, Output, Error, Context, NodeTypes, NodeIOETypes> Clone
            for $flow_name<Input, Output, Error, Context, NodeTypes, NodeIOETypes>
        {
            fn clone(&self) -> Self {
                Self {
                    _ioec: std::marker::PhantomData,
                    _nodes_io: std::marker::PhantomData,
                    nodes: self.nodes.clone(),
                }
            }
        }

        impl<Input, Output, Error, Context> $flow_name<Input, Output, Error, Context>
        where
            // Trait bounds for better and nicer errors
            $($param: $bound0 $(+$bound)*,)*
        {
            #[must_use]
            pub fn builder() -> $builder<Input, Output, Error, Context> {
                $builder::new()
            }
        }

        impl<Input, Output, Error, Context, NodeTypes, NodeIOETypes>
            $crate::node::Node<Input, $crate::node::NodeOutput<Output>, Error, Context>
            for $flow_name<Input, Output, Error, Context, NodeTypes, NodeIOETypes>
        where
            NodeTypes: $chain_run<Input, $crate::flows::NodeResult<Output, Error>, Context, NodeIOETypes>,
        {
            fn run(
                &mut self,
                input: Input,
                context: &mut Context,
            ) -> impl Future<Output = $crate::flows::NodeResult<Output, Error>> + Send {
                $chain_run::run(self.nodes.as_ref(), input, context)
            }
        }
    };
}

pub(crate) use define_flow;
