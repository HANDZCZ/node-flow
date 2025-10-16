/// Defines flow with specified `ChainRun`, additional bounds and doc comments
macro_rules! define_flow {
    ($flow_name:ident, $chain_run:ident $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        define_flow!($flow_name, Builder, $chain_run $(,$param: $bound0 $(+$bound)*)* $(#[doc = $doc])*);
    };
    ($flow_name:ident, $builder:ident, $chain_run:ident $(,$param:ident: $bound0:ident $(+$bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        $(#[doc = $doc])*
        pub struct $flow_name<Input, Output, Error, NodeTypes = (), NodeIOETypes = ()> {
            pub(super) _ioe: std::marker::PhantomData<(Input, Output, Error)>,
            pub(super) _nodes_io: std::marker::PhantomData<NodeIOETypes>,
            pub(super) nodes: std::sync::Arc<NodeTypes>,
        }

        impl<Input, Output, Error> $flow_name<Input, Output, Error>
        where
            // Trait bounds for better and nicer errors
            $($param: $bound0 $(+$bound)*,)*
        {
            #[must_use]
            pub fn builder() -> $builder<Input, Output, Error> {
                $builder::new()
            }
        }

        impl<Input, Output, Error, NodeTypes, NodeIOETypes>
            $crate::node::Node<Input, $crate::node::NodeOutput<Output>, Error>
            for $flow_name<Input, Output, Error, NodeTypes, NodeIOETypes>
        where
            NodeTypes: $chain_run<Input, $crate::flows::NodeResult<Output, Error>, NodeIOETypes>,
        {
            fn run_with_storage(
                &mut self,
                input: Input,
                storage: &mut $crate::storage::Storage,
            ) -> impl Future<Output = $crate::flows::NodeResult<Output, Error>> + Send {
                self.nodes.run_with_storage(input, storage)
            }
        }
    };
}

pub(crate) use define_flow;
