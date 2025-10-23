macro_rules! impl_debug_for_builder {
    ($flow_name:expr, $builder:tt $(,$param:ident: $bound0:ident $(+$bound:ident)*)*) => {
        impl<Input, Output, Error, Context, NodeTypes, NodeIOETypes> std::fmt::Debug
            for $builder<Input, Output, Error, Context, NodeTypes, NodeIOETypes>
        where
            NodeTypes: $crate::flows::chain_debug::ChainDebug,
            // Trait bounds for better and nicer errors
            $($param: $bound0 $(+$bound)*,)*
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(&format!("{}Builder", $flow_name))
                    .field("nodes", &self.nodes.as_list())
                    .finish_non_exhaustive()
            }
        }
    };
}

pub(crate) use impl_debug_for_builder;

macro_rules! impl_debug_for_flow {
    ($flow_name:expr, $flow_type:tt $(,$param:ident: $bound0:ident $(+$bound:ident)*)*) => {
        impl<Input, Output, Error, Context, NodeTypes, NodeIOETypes> std::fmt::Debug
            for $flow_type<Input, Output, Error, Context, NodeTypes, NodeIOETypes>
        where
            NodeTypes: $crate::flows::chain_debug::ChainDebug,
            // Trait bounds for better and nicer errors
            $($param: $bound0 $(+$bound)*,)*
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct($flow_name)
                    .field("nodes", &self.nodes.as_list())
                    .finish_non_exhaustive()
            }
        }
    };
}

pub(crate) use impl_debug_for_flow;
