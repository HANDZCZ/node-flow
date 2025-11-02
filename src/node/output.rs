/// Represents the outcome of a node execution, with support for soft failures.
///
/// # Examples
/// ```
/// use node_flow::node::NodeOutput;
///
/// let success: NodeOutput<i32> = NodeOutput::Ok(42);
/// let failure: NodeOutput<i32> = NodeOutput::SoftFail;
///
/// assert_eq!(success.ok(), Some(42));
/// assert_eq!(failure.ok(), None);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum NodeOutput<T> {
    /// Indicates that the node failed in a non-critical way and produced no output.
    ///
    /// This is distinct from a hard failure (error) and may simply mean that
    /// input conditions were not met.
    SoftFail,
    /// Indicates that the node successfully produced a value of type `T`.
    Ok(T),
}

impl<T> NodeOutput<T> {
    /// Converts `NodeOutput<T>` into an [`Option<T>`].
    ///
    /// - Returns `Some(T)` if the output is [`NodeOutput::Ok`].
    /// - Returns `None` if the output is [`NodeOutput::SoftFail`].
    ///
    /// # Examples
    /// ```
    /// use node_flow::node::NodeOutput;
    ///
    /// let output = NodeOutput::Ok(5);
    /// assert_eq!(output.ok(), Some(5));
    ///
    /// let failed = NodeOutput::<i32>::SoftFail;
    /// assert_eq!(failed.ok(), None);
    /// ```
    pub fn ok(self) -> Option<T> {
        match self {
            Self::SoftFail => None,
            Self::Ok(val) => Some(val),
        }
    }

    /// Converts `NodeOutput<T>` into a [`Result<T, E>`],
    /// using a provided error value if soft-failed.
    ///
    /// - Returns `Ok(T)` if the output is [`NodeOutput::Ok`].
    /// - Returns `Err(err)` if the output is [`NodeOutput::SoftFail`].
    ///
    /// # Errors
    /// Returns the provided `err` value if the node soft-failed.
    ///
    /// # Examples
    /// ```
    /// use node_flow::node::NodeOutput;
    ///
    /// let ok: Result<i32, &str> = NodeOutput::Ok(42).ok_or("no value");
    /// assert_eq!(ok, Ok(42));
    ///
    /// let soft_fail: Result<i32, &str> = NodeOutput::SoftFail.ok_or("no value");
    /// assert_eq!(soft_fail, Err("no value"));
    /// ```
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::SoftFail => Err(err),
            Self::Ok(val) => Ok(val),
        }
    }

    /// Converts `NodeOutput<T>` into a [`Result<T, E>`],
    /// lazily computing the error value if soft-failed.
    ///
    /// - Returns `Ok(T)` if the output is [`NodeOutput::Ok`].
    /// - Calls `err()` and returns `Err(err())` if the output is [`NodeOutput::SoftFail`].
    ///
    /// This is the lazy variant of [`NodeOutput::ok_or`], avoiding unnecessary error construction
    /// when the node succeeds.
    ///
    /// # Errors
    /// Calls the provided closure to produce an error if the node soft-failed.
    ///
    /// # Examples
    /// ```
    /// use node_flow::node::NodeOutput;
    ///
    /// let ok: Result<i32, String> = NodeOutput::Ok(10).ok_or_else(|| "soft fail".to_string());
    /// assert_eq!(ok, Ok(10));
    ///
    /// let soft_fail: Result<i32, String> = NodeOutput::SoftFail.ok_or_else(|| "soft fail".to_string());
    /// assert_eq!(soft_fail, Err("soft fail".to_string()));
    /// ```
    pub fn ok_or_else<E>(self, err: impl Fn() -> E) -> Result<T, E> {
        match self {
            Self::SoftFail => Err(err()),
            Self::Ok(val) => Ok(val),
        }
    }
}
