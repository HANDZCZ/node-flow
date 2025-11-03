/// Builder that checks:
/// - `Input: Clone`
/// - `Input: Into<NodeInput>`
/// - `NodeOutput: Into<Input>`
/// - `NodeError: Into<Error>`
macro_rules! define_builder {
    ($flow_type:ident $(,>$global_param:ident: $global_bound0:ident $(+$global_bound:ident)*)* $(,#$fn_param:ident: $fn_bound0:ident $(+$fn_bound:ident)*)*) => {
        #[doc = concat!("Builder for [`", stringify!($flow_type), "`].")]
        ///
        /// This builder ensures:
        /// - `Input` into the flow can be converted into the input of all nodes
        /// - output of all nodes can be converted into the `Output` of the flow
        /// - error of all nodes can be converted into the `Error` of the flow
        ///
        #[doc = concat!("See also [`", stringify!($flow_type), "`].")]
        pub struct Builder<Input, Output, Error, Context, NodeTypes = (), NodeIOETypes = ()>
        where
            // Trait bounds for better and nicer errors
            $($global_param: $global_bound0 $(+$global_bound)*,)*
        {
            _ioec: std::marker::PhantomData<fn() -> (Input, Output, Error, Context)>,
            _nodes_io: std::marker::PhantomData<fn() -> NodeIOETypes>,
            nodes: NodeTypes,
        }

        $crate::flows::generic_defs::debug::impl_debug_for_builder!(stringify!($flow_type), Builder $(,$global_param: $global_bound0 $(+$global_bound)*)*);

        impl<Input, Output, Error, Context> Default for Builder<Input, Output, Error, Context>
        where
            // Trait bounds for better and nicer errors
            $($global_param: $global_bound0 $(+$global_bound)*,)*
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<Input, Output, Error, Context> Builder<Input, Output, Error, Context>
        where
            // Trait bounds for better and nicer errors
            $($global_param: $global_bound0 $(+$global_bound)*,)*
        {
            #[doc = concat!("Creates a new empty builder for [`", stringify!($flow_type), "`].")]
            #[must_use]
            pub fn new() -> Self {
                Self {
                    _ioec: std::marker::PhantomData,
                    _nodes_io: std::marker::PhantomData,
                    nodes: (),
                }
            }

            /// Adds a new node.
            ///
            /// The new node must satisfy:
            /// - `Self`: `Node<NodeInputType, NodeOutput<NodeOutputType>, NodeErrorType, _>`
            /// - `Input`: `Into<NodeInputType>`,
            /// - `NodeOutputType`: `Into<Output>`,
            /// - `NodeErrorType`: `Into<Error>`,
            ///
            /// # Returns
            /// A new [`Builder`] with the added node.
            pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
                self,
                node: NodeType,
            ) -> Builder<
                Input,
                Output,
                Error,
                Context,
                (NodeType,),
                $crate::flows::ChainLink<
                    (),
                    $crate::flows::NodeIOE<NodeInput, NodeOutput, NodeError>,
                >,
            >
            where
                Input: Into<NodeInput>,
                NodeOutput: Into<Output>,
                NodeError: Into<Error>,
                NodeType:
                    $crate::node::Node<NodeInput, $crate::node::NodeOutput<NodeOutput>, NodeError, Context>,
                // Trait bounds for better and nicer errors
                $($fn_param: $fn_bound0 $(+$fn_bound)*,)*
            {
                Builder {
                    _ioec: std::marker::PhantomData,
                    _nodes_io: std::marker::PhantomData,
                    nodes: (node,),
                }
            }
        }

        impl<Input, Output, Error, Context, NodeTypes, LastNodeIOETypes, OtherNodeIOETypes>
            Builder<
                Input,
                Output,
                Error,
                Context,
                NodeTypes,
                $crate::flows::ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            >
        where
            // Trait bounds for better and nicer errors
            $($global_param: $global_bound0 $(+$global_bound)*,)*
        {
            /// Adds a new node.
            ///
            /// The new node must satisfy:
            /// - `Self`: `Node<NodeInputType, NodeOutput<NodeOutputType>, NodeErrorType, _>`
            /// - `Input`: `Into<NodeInputType>`,
            /// - `NodeOutputType`: `Into<Output>`,
            /// - `NodeErrorType`: `Into<Error>`,
            ///
            /// # Returns
            /// A new [`Builder`] with the added node.
            pub fn add_node<NodeType, NodeInput, NodeOutput, NodeError>(
                self,
                node: NodeType,
            ) -> Builder<
                Input,
                Output,
                Error,
                Context,
                $crate::flows::ChainLink<NodeTypes, NodeType>,
                $crate::flows::ChainLink<
                    $crate::flows::ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
                    $crate::flows::NodeIOE<NodeInput, NodeOutput, NodeError>,
                >,
            >
            where
                Input: Into<NodeInput>,
                NodeOutput: Into<Output>,
                NodeError: Into<Error>,
                NodeType:
                    $crate::node::Node<NodeInput, $crate::node::NodeOutput<NodeOutput>, NodeError, Context>,
                // Trait bounds for better and nicer errors
                $($fn_param: $fn_bound0 $(+$fn_bound)*,)*
            {
                Builder {
                    _ioec: std::marker::PhantomData,
                    _nodes_io: std::marker::PhantomData,
                    nodes: (self.nodes, node),
                }
            }

            #[doc = concat!("Finalizes the builder and produces a [`", stringify!($flow_type), "`] instance.")]
            pub fn build(
                self,
            ) -> $flow_type<
                Input,
                Output,
                Error,
                Context,
                NodeTypes,
                $crate::flows::ChainLink<OtherNodeIOETypes, LastNodeIOETypes>,
            > {
                $flow_type {
                    _ioec: std::marker::PhantomData,
                    _nodes_io: std::marker::PhantomData,
                    nodes: std::sync::Arc::new(self.nodes),
                }
            }
        }
    };
}

pub(crate) use define_builder;
