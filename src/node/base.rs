use crate::describe::{Description, DescriptionBase};

/// The `Node` trait serves as the core building block.
///
/// It defines how `Input` is processed into an `Output` (and `Error`) with a given `Context`.
///
/// # Type Parameters
/// - `Input`: The type of data consumed by the node.
/// - `Output`: The type of data produced by the node.
/// - `Error`: The type representing possible errors.
/// - `Context`: The type of context used during execution (it should always be a generic).
///
/// # Examples
/// ```
/// # use node_flow::node::{Node, NodeOutput};
/// # use node_flow::describe::Description;
/// #
/// # struct ExampleNode;
/// # struct MyError;
/// #
/// impl<Context: Send> Node<i32, NodeOutput<String>, MyError, Context> for ExampleNode {
///     async fn run(
///         &mut self,
///         input: i32,
///         _context: &mut Context,
///     ) -> Result<NodeOutput<String>, MyError> {
///         Ok(NodeOutput::Ok(format!("Processed: {}", input)))
///     }
///
///     fn describe(&self) -> Description {
///         Description::new_node::<Self, i32, String, MyError, Context>(self)
///             .with_description("My example node")
///     }
/// }
/// ```
pub trait Node<Input, Output, Error, Context> {
    /// Runs the node.
    ///
    /// This method performs the node's main computation or transformation logic.
    ///
    /// # Parameters
    /// - `input`: The input data to process.
    /// - `context`: Mutable reference to a context, which may be used for configuration, logging, or shared state.
    ///
    /// # Returns
    /// A [`Future`] that resolves to a `Result<Output, Error>`.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::node::Node;
    /// #
    /// # struct ExampleNode;
    /// # struct MyError;
    /// #
    /// impl<Context: Send> Node<i32, String, MyError, Context> for ExampleNode {
    ///     async fn run(
    ///         &mut self,
    ///         input: i32,
    ///         _context: &mut Context,
    ///     ) -> Result<String, MyError> {
    ///         Ok(format!("Processed: {}", input))
    ///     }
    /// }
    /// ```
    fn run(
        &mut self,
        input: Input,
        context: &mut Context,
    ) -> impl Future<Output = Result<Output, Error>> + Send;

    /// Describes this node, its type signature and other specifics.
    ///
    /// See [`Description`] for more details.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::node::{Node, NodeOutput};
    /// # use node_flow::describe::Description;
    /// #
    /// # struct ExampleNode;
    /// # struct MyError;
    /// #
    /// impl<Context: Send> Node<i32, NodeOutput<String>, MyError, Context> for ExampleNode {
    ///     async fn run(&mut self, input: i32, _context: &mut Context)
    ///         -> Result<NodeOutput<String>, MyError> { todo!() }
    ///
    ///     fn describe(&self) -> Description {
    ///         Description::new_node::<Self, i32, String, MyError, Context>(self)
    ///             .with_description("My example node")
    ///     }
    /// }
    /// ```
    // if specialization is ever stabilized this whole function can be removed
    // and Describe trait with default impl for Node<..> could be used
    // https://github.com/rust-lang/rust/issues/31844
    fn describe(&self) -> Description
    where
        Self: Sized,
    {
        let mut base = DescriptionBase::from::<Self, Input, Output, Error, Context>();

        // remove NodeOutput<> from output name
        let output_name = &mut base.output.name;
        if let Some(b_pos) = output_name.find('<')
            && output_name[..b_pos].contains("NodeOutput")
        {
            // remove `..::NodeOutput<`
            output_name.replace_range(0..=b_pos, "");
            // remove ending `>`
            output_name.pop();
        }

        Description::Node { base }
    }
}
