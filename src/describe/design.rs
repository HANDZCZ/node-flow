use std::any::type_name;

use crate::node::{Node, NodeOutput};

#[derive(Debug, Clone)]
pub enum Description {
    Node {
        base: DescriptionBase,
    },
    Flow {
        base: DescriptionBase,
        nodes: Vec<Description>,
        edges: Vec<Edge>,
    },
}

impl Description {
    #[must_use]
    pub fn new_node<NodeType, Input, Output, Error, Context>(node: &NodeType) -> Self
    where
        NodeType: Node<Input, NodeOutput<Output>, Error, Context>,
    {
        Self::Node {
            base: DescriptionBase::from_node(node),
        }
    }

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

    #[must_use]
    pub const fn get_base_mut(&mut self) -> &mut DescriptionBase {
        match self {
            Self::Node { base } | Self::Flow { base, .. } => base,
        }
    }

    #[must_use]
    pub const fn get_base_ref(&self) -> &DescriptionBase {
        match self {
            Self::Node { base } | Self::Flow { base, .. } => base,
        }
    }

    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.get_base_mut().description = Some(description.into());
        self
    }

    #[must_use]
    pub fn with_externals(mut self, externals: Vec<ExternalResource>) -> Self {
        self.get_base_mut().externals = Some(externals);
        self
    }

    #[must_use]
    pub fn modify_name(mut self, func: impl FnOnce(&mut String)) -> Self {
        let name = &mut self.get_base_mut().r#type.name;
        func(name);
        self
    }
}

#[derive(Debug, Clone)]
pub struct DescriptionBase {
    pub r#type: Type,
    pub input: Type,
    pub output: Type,
    pub error: Type,
    pub context: Type,
    pub description: Option<String>,
    pub externals: Option<Vec<ExternalResource>>,
}

impl DescriptionBase {
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

    #[must_use]
    pub fn from_node<NodeType, Input, Output, Error, Context>(_node: &NodeType) -> Self
    where
        NodeType: Node<Input, NodeOutput<Output>, Error, Context>,
    {
        Self::from::<NodeType, Input, Output, Error, Context>()
    }

    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn with_externals(mut self, externals: Vec<ExternalResource>) -> Self {
        self.externals = Some(externals);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Type {
    pub name: String,
}

impl Type {
    #[must_use]
    pub fn of<T>() -> Self {
        Self {
            name: type_name::<T>().to_owned(),
        }
    }

    #[must_use]
    pub fn of_val<T>(_: &T) -> Self {
        Self::of::<T>()
    }

    #[cfg(feature = "describe_get_name_simple")]
    #[must_use]
    pub fn get_name_simple(&self) -> String {
        tynm::TypeName::from(self.name.as_str()).as_str()
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub start: EdgeEnding,
    pub end: EdgeEnding,
}

#[derive(Debug, Clone)]
pub enum EdgeEnding {
    ToFlow,
    ToNode { node_index: usize },
}

impl Edge {
    #[must_use]
    pub const fn passthrough() -> Self {
        Self {
            start: EdgeEnding::ToFlow,
            end: EdgeEnding::ToFlow,
        }
    }
    #[must_use]
    pub const fn flow_to_node(node_idx: usize) -> Self {
        Self {
            start: EdgeEnding::ToFlow,
            end: EdgeEnding::ToNode {
                node_index: node_idx,
            },
        }
    }
    #[must_use]
    pub const fn node_to_flow(node_idx: usize) -> Self {
        Self {
            start: EdgeEnding::ToNode {
                node_index: node_idx,
            },
            end: EdgeEnding::ToFlow,
        }
    }
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

#[derive(Debug, Clone)]
pub struct ExternalResource {
    pub r#type: Type,
    pub description: Option<String>,
    pub output: Type,
}

impl ExternalResource {
    #[must_use]
    pub fn new<ResourceType, Output>() -> Self {
        Self {
            r#type: Type::of::<ResourceType>(),
            description: None,
            output: Type::of::<Output>(),
        }
    }

    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
