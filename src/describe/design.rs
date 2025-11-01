use std::any::type_name;

use crate::node::{Node, NodeOutput};

/// Represents a description of either a single [`Node`] or an entire flow of connected nodes.
///
/// This enum is primarily used for introspection and visualization of a flow.
#[derive(Debug, Clone)]
pub enum Description {
    /// Single node description.
    Node {
        /// The base description containing type information and metadata.
        base: DescriptionBase,
    },
    /// Description of a flow.
    ///
    /// Flow description is a collection of nodes connected by edges.
    /// It's used for describing composite node structures and pipelines.
    Flow {
        /// The base description containing type information and metadata.
        base: DescriptionBase,
        /// The collection of node descriptions that make up this flow.
        nodes: Vec<Description>,
        /// The connections between nodes within this flow.
        edges: Vec<Edge>,
    },
}

impl Description {
    /// Creates a new [`Description::Node`] from a given [`Node`] instance.
    #[must_use]
    pub fn new_node<NodeType, Input, Output, Error, Context>(node: &NodeType) -> Self
    where
        NodeType: Node<Input, NodeOutput<Output>, Error, Context>,
    {
        Self::Node {
            base: DescriptionBase::from_node(node),
        }
    }

    /// Creates a new [`Description::Flow`] from a given [`Node`] instance.
    ///
    /// # Parameters
    /// - `node`: The flow-level node.
    /// - `nodes`: The list of contained node descriptions.
    /// - `edges`: The connections between nodes.
    #[must_use]
    pub fn new_flow<NodeType, Input, Output, Error, Context>(
        node: &NodeType,
        nodes: Vec<Self>,
        edges: Vec<Edge>,
    ) -> Self
    where
        NodeType: Node<Input, NodeOutput<Output>, Error, Context>,
    {
        Self::Flow {
            base: DescriptionBase::from_node(node),
            edges,
            nodes,
        }
    }

    /// Returns a mutable reference to the underlying [`DescriptionBase`].
    #[must_use]
    pub const fn get_base_mut(&mut self) -> &mut DescriptionBase {
        match self {
            Self::Node { base } | Self::Flow { base, .. } => base,
        }
    }

    /// Returns an immutable reference to the underlying [`DescriptionBase`].
    #[must_use]
    pub const fn get_base_ref(&self) -> &DescriptionBase {
        match self {
            Self::Node { base } | Self::Flow { base, .. } => base,
        }
    }

    /// Sets a description on this node or flow.
    ///
    /// This is primarily used to provide additional documentation or context.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.get_base_mut().description = Some(description.into());
        self
    }

    /// Sets the external resources used by this node or flow.
    ///
    /// External resources could include remote calls, file accesses or other types of external services.
    #[must_use]
    pub fn with_externals(mut self, externals: Vec<ExternalResource>) -> Self {
        self.get_base_mut().externals = Some(externals);
        self
    }

    /// Modifies the name using a provided function.
    ///
    /// This is useful when you only want to modify the name.
    #[must_use]
    pub fn modify_name(mut self, func: impl FnOnce(&mut String)) -> Self {
        let name = &mut self.get_base_mut().r#type.name;
        func(name);
        self
    }
}

/// The base structure describing a node's type signature and metadata.
///
/// Contains information about the node's input, output, error, and context types,
/// along with optional description and external resource metadata.
#[derive(Debug, Clone)]
pub struct DescriptionBase {
    /// The type of the node or flow itself.
    pub r#type: Type,
    /// The type of input accepted by the node.
    pub input: Type,
    /// The type of output produced by the node.
    pub output: Type,
    /// The type of error that may be returned by the node.
    pub error: Type,
    /// The type of context used when executing the node.
    pub context: Type,
    /// An optional description of the node or flow.
    pub description: Option<String>,
    /// Optional list of external resources the node uses.
    pub externals: Option<Vec<ExternalResource>>,
}

impl DescriptionBase {
    /// Creates a [`DescriptionBase`] from type parameters.
    #[must_use]
    pub fn from<NodeType, Input, Output, Error, Context>() -> Self {
        Self {
            r#type: Type::of::<NodeType>(),
            input: Type::of::<Input>(),
            output: Type::of::<Output>(),
            error: Type::of::<Error>(),
            context: Type::of::<Context>(),
            description: None,
            externals: None,
        }
    }

    /// Creates a [`DescriptionBase`] from a given [`Node`] instance.
    #[must_use]
    pub fn from_node<NodeType, Input, Output, Error, Context>(_node: &NodeType) -> Self
    where
        NodeType: Node<Input, NodeOutput<Output>, Error, Context>,
    {
        Self::from::<NodeType, Input, Output, Error, Context>()
    }

    /// Sets a description.
    ///
    /// This is primarily used to provide additional documentation or context.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the external resources.
    ///
    /// External resources could include remote calls, file accesses or other types of external services.
    #[must_use]
    pub fn with_externals(mut self, externals: Vec<ExternalResource>) -> Self {
        self.externals = Some(externals);
        self
    }
}

/// Represents a type.
#[derive(Debug, Clone)]
pub struct Type {
    /// The name of a type.
    ///
    /// By default it should be the fully qualified type name returned by `std::any::type_name::<T>()`.
    pub name: String,
}

impl Type {
    /// Creates a [`Type`] representing the type `T`.
    #[must_use]
    pub fn of<T>() -> Self {
        Self {
            name: type_name::<T>().to_owned(),
        }
    }

    /// Creates a [`Type`] based on the value's type.
    #[must_use]
    pub fn of_val<T>(_: &T) -> Self {
        Self::of::<T>()
    }

    /// Returns a simplified version of the type name.
    ///
    /// Instead of `std::option::Option<std::string::String>` it returns `Option<String>`.
    #[cfg(feature = "describe_get_name_simple")]
    #[must_use]
    pub fn get_name_simple(&self) -> String {
        tynm::TypeName::from(self.name.as_str()).as_str()
    }
}

/// Represents a directional connection between nodes in a flow.
///
/// Each edge connects two [`EdgeEnding`]s, which can be either a node or the flow itself.
#[derive(Debug, Clone)]
pub struct Edge {
    /// The starting point of the edge.
    pub start: EdgeEnding,
    /// The ending point of the edge.
    pub end: EdgeEnding,
}

/// Represents one end of an [`Edge`].
///
/// An `EdgeEnding` can either connect to the flow or to a specific node.
#[derive(Debug, Clone)]
pub enum EdgeEnding {
    /// The edge connects to the flow.
    ToFlow,
    /// The edge connects to a specific node.
    ToNode {
        /// The index of the node within the flow.
        node_index: usize,
    },
}

impl Edge {
    /// Creates an edge that passes directly through the flow without connecting to any nodes.
    #[must_use]
    pub const fn passthrough() -> Self {
        Self {
            start: EdgeEnding::ToFlow,
            end: EdgeEnding::ToFlow,
        }
    }

    /// Creates an edge connecting the flow to a specific node.
    #[must_use]
    pub const fn flow_to_node(node_idx: usize) -> Self {
        Self {
            start: EdgeEnding::ToFlow,
            end: EdgeEnding::ToNode {
                node_index: node_idx,
            },
        }
    }

    /// Creates an edge connecting a node to the flow.
    #[must_use]
    pub const fn node_to_flow(node_idx: usize) -> Self {
        Self {
            start: EdgeEnding::ToNode {
                node_index: node_idx,
            },
            end: EdgeEnding::ToFlow,
        }
    }

    /// Creates an edge connecting one node to another.
    #[must_use]
    pub const fn node_to_node(start_node_idx: usize, end_node_idx: usize) -> Self {
        Self {
            start: EdgeEnding::ToNode {
                node_index: start_node_idx,
            },
            end: EdgeEnding::ToNode {
                node_index: end_node_idx,
            },
        }
    }
}

/// Represents an external resource dependency.
///
/// These resources may represent things like files, APIs, or external data sources.
#[derive(Debug, Clone)]
pub struct ExternalResource {
    /// The type of the external resource.
    pub r#type: Type,
    /// An optional description of the external resource.
    pub description: Option<String>,
    /// The type of data produced by this resource.
    pub output: Type,
}

impl ExternalResource {
    /// Creates a new [`ExternalResource`] description.
    #[must_use]
    pub fn new<ResourceType, Output>() -> Self {
        Self {
            r#type: Type::of::<ResourceType>(),
            description: None,
            output: Type::of::<Output>(),
        }
    }

    /// Sets a description.
    ///
    /// This is primarily used to provide additional documentation or context.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
