/// Implements the [`Node`](crate::node::Node) trait for a type whose output
/// needs to be wrapped in [`NodeOutput`](crate::node::NodeOutput).
///
/// It automatically generates an implementation of:
/// ```ignore
/// impl<Context> Node<Input, NodeOutput<Output>, Error, Context> for NodeType
/// ```
/// From an existing implementation of:
/// ```ignore
/// impl<Context> Node<Input, Output, Error, Context> for NodeType
/// ```
///
/// This macro adds a `Node` implementation that returns `NodeOutput<T>`.
/// It works by wrapping an existing implementation of `Node` that returns `Output`,
/// turning it into `NodeOutput<Output>`.
///
/// # Parameters
/// - `$node`: The node type to add an implementation for.
/// - `$input`, `$output`, `$error`: Input, Output and Error type parameters for the node.
/// - `$param: $bound0 + $bound`: Optional trait bounds for generics.
///
/// See also [`Node`](crate::node::Node), [`NodeOutput`](crate::node::NodeOutput).
///
/// # Examples
/// ```
/// use node_flow::{impl_node_output, node::{Node, NodeOutput}};
///
/// struct ExampleNode;
///
/// impl<Context: Send> Node<i32, i32, String, Context> for ExampleNode {
///     async fn run(&mut self, input: i32, _context: &mut Context) -> Result<i32, String> {
///         Ok(input + 1)
///     }
/// }
///
/// // Automatically implement Node<i32, NodeOutput<i32>, String, _> for ExampleNode.
/// impl_node_output!(ExampleNode, i32, i32, String);
/// ```
#[macro_export]
macro_rules! impl_node_output {
    ($node:ty, $input:ty, $output:ty, $error:ty $(,$param:ident: $bound0:ident $(+$bound:ident)*)*) => {
        impl<Context>
            $crate::node::Node<$input, $crate::node::NodeOutput<$output>, $error, Context>
            for $node
        where
            Context: Send,
            $($param: $bound0 $(+$bound)*,)*
        {
            async fn run(
                &mut self,
                input: $input,
                context: &mut Context,
            ) -> Result<$crate::node::NodeOutput<$output>, $error> {
                Ok($crate::node::NodeOutput::Ok(
                    <Self as $crate::node::Node<$input, $output, $error, Context>>::run(
                        self, input, context,
                    )
                    .await?,
                ))
            }

            fn describe(&self) -> $crate::describe::Description {
                <Self as $crate::node::Node<$input, $output, $error, Context>>::describe(self)
            }
        }
    };
}

/// A helper macro for declaring [`impl Node`](crate::node::Node) return type.
///
/// Use `!` before an output type for it to **not** be wrapped in [`NodeOutput`](crate::node::NodeOutput).
///
/// # Parameters
/// - `$input`: The input type accepted by the node.
/// - `$output`: The output type (raw or wrapped) returned by the node.
/// - `$error`: The error type returned by the node.
/// - `$context`: The context type passed to the node.
///
/// See also [`Node`](crate::node::Node), [`NodeOutput`](crate::node::NodeOutput).
///
/// # Examples
/// ```
/// use node_flow::node::Node;
///
/// #[derive(Clone)]
/// struct ExampleNode;
/// impl<Context: Send> Node<u8, u64, String, Context> for ExampleNode // ...
/// # {
/// #     async fn run(
/// #         &mut self,
/// #         input: u8,
/// #         _context: &mut Context,
/// #     ) -> Result<u64, String> {
/// #         Ok(input as u64)
/// #     }
/// # }
///
/// fn build_flow<Context: Send>() -> node_flow::node!(u8, !u64, String, Context) {
///    ExampleNode
/// }
/// ```
///
/// # Expansion
/// ```ignore
/// node!($input, $output, $error, $context)
/// // expands to:
/// impl node_flow::node::Node<
///     $input,
///     node_flow::node::NodeOutput<$output>,
///     $error,
///     $context
/// > + Clone + Send + Sync
/// ```
/// By adding `!` to output the raw `$output` type is used:
/// ```ignore
/// node!($input, !$output, $error, $context)
/// // expands to:
/// impl node_flow::node::Node<$input, $output, $error, $context> + Clone + Send + Sync
/// ```
#[macro_export]
macro_rules! node {
    ($input:ty, !$output:ty, $error:ty, $context:ty) => {
        impl $crate::node::Node<$input, $output, $error, $context> + Clone + Send + Sync
    };
    ($input:ty, $output:ty, $error:ty, $context:ty) => {
        $crate::node!($input, !$crate::node::NodeOutput<$output>, $error, $context)
    };
}
