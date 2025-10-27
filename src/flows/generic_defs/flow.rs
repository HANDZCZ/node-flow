/// Defines flow with specified `ChainRun`, additional bounds and doc comments
macro_rules! define_flow {
    ($flow_name:ident, $chain_run:ident, |$self:ident| $describe_code:block $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        define_flow!($flow_name, Builder, $chain_run, |$self| $describe_code $(,$param: $bound0 $(+$bound)*)* $(#[doc = $doc])*);
    };
    ($flow_name:ident, $builder:ident, $chain_run:ident, |$self:ident| $describe_code:block $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        $(#[doc = $doc])*
        pub struct $flow_name<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()> {
            pub(super) _ioec: std::marker::PhantomData<fn() -> (Input, Output, Error, Context)>,
            pub(super) _nodes_io: std::marker::PhantomData<fn() -> NodeIOETypes>,
            pub(super) nodes: std::sync::Arc<NodeTypes>,
        }

        $crate::flows::generic_defs::debug::impl_debug_for_flow!(stringify!($flow_name), $flow_name);

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
            NodeTypes: $chain_run<Input, $crate::flows::NodeResult<Output, Error>, Context, NodeIOETypes>
                + $crate::flows::chain_describe::ChainDescribe<Context, NodeIOETypes>,
        {
            fn run(
                &mut self,
                input: Input,
                context: &mut Context,
            ) -> impl Future<Output = $crate::flows::NodeResult<Output, Error>> + Send {
                $chain_run::run(self.nodes.as_ref(), input, context)
            }

            fn describe(& $self) -> $crate::describe::Description {
                $describe_code
            }
        }
    };
}

pub(crate) use define_flow;
