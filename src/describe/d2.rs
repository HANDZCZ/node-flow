use super::design::{Description, Edge, EdgeEnding, ExternalResource, Type};
use std::{borrow::Cow, fmt::Write};

/// A configurable formatter for converting [`Description`] structures into
/// [D2](https://d2lang.com/) graph syntax.
///
/// # Examples
///
/// ```
/// use node_flow::describe::{Description, D2Describer};
/// use node_flow::node::{Node, NodeOutput};
/// use node_flow::flows::FnFlow;
///
/// # struct ExampleNode;
/// #
/// # impl Node<i32, NodeOutput<String>, (), ()> for ExampleNode {
/// #     async fn run(
/// #         &mut self,
/// #         input: i32,
/// #         _context: &mut (),
/// #     ) -> Result<NodeOutput<String>, ()> {
/// #         Ok(NodeOutput::Ok(format!("Processed: {}", input)))
/// #     }
/// # }
/// let flow = ExampleNode;
/// let some_description = flow.describe();
///
/// let mut describer = D2Describer::new();
/// describer.modify(|cfg| {
///     cfg.show_description = true;
///     cfg.show_externals = true;
/// });
///
/// let d2_code = describer.format(&some_description);
/// println!("{}", d2_code);
/// // Output could be fed to a D2 renderer for visualization.
/// ```
#[expect(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct D2Describer {
    /// Whether to display simplified type names instead of full paths.
    ///
    /// When enabled, types like `my_crate::nodes::ExampleNode` become `ExampleNode`.
    /// This makes diagrams more readable, especially for complex flows.
    pub simple_type_name: bool,
    /// Whether to display the node context type inside each node.
    ///
    /// When enabled, context will be added to node's description.
    pub show_context_in_node: bool,
    /// Whether to include the node's description.
    ///
    /// When enabled, description will be included in the node.
    pub show_description: bool,
    /// Whether to include information about external resources.
    ///
    /// When enabled, external resources will be included in the node.
    pub show_externals: bool,
}

impl Default for D2Describer {
    fn default() -> Self {
        Self {
            simple_type_name: true,
            show_context_in_node: false,
            show_description: false,
            show_externals: false,
        }
    }
}

fn escape_str(val: &str) -> String {
    val.replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

impl D2Describer {
    /// Creates a new [`D2Describer`] using default configuration.
    ///
    /// Default settings:
    /// - `simple_type_name`: `true`
    /// - `show_context_in_node`: `false`
    /// - `show_description`: `false`
    /// - `show_externals`: `false`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows modification of the configuration using a closure.
    ///
    /// # Examples
    /// ```
    /// # use node_flow::describe::D2Describer;
    /// let mut describer = D2Describer::new();
    /// describer.modify(|cfg| {
    ///     cfg.show_description = true;
    ///     cfg.show_externals = true;
    /// });
    /// ```
    pub fn modify(&mut self, func: impl FnOnce(&mut Self)) -> &mut Self {
        func(self);
        self
    }

    fn get_type_name<'a>(&self, r#type: &'a Type) -> Cow<'a, str> {
        if r#type.name.is_empty() {
            return Cow::Borrowed("\"\"");
        }

        if self.simple_type_name {
            let res = r#type.get_name_simple();
            // fallback
            if res.is_empty() {
                return Cow::Borrowed(&r#type.name);
            }
            Cow::Owned(res)
        } else {
            Cow::Borrowed(&r#type.name)
        }
    }

    /// Formats a [`Description`] into a D2 diagram text representation.
    ///
    /// The resulting string can be passed directly to the D2 CLI or rendered using
    /// the [D2 playground](https://play.d2lang.com/).
    ///
    /// # Parameters
    /// - `desc`: The [`Description`] to be rendered.
    ///
    /// # Returns
    /// A string containing valid D2 source code representing the description graph.
    #[must_use]
    pub fn format(&self, desc: &Description) -> String {
        let id = rand::random();
        let (input, output, context) = {
            let base = desc.get_base_ref();
            (&base.input, &base.output, &base.context)
        };
        let mut res = format!(
            r"direction: down
classes: {{
    node: {{
        style.border-radius: 8
    }}
    flow: {{
        style.border-radius: 8
    }}
    edge: {{
        style.font-size: 18
    }}
    node_flow_description: {{
        shape: page
    }}
    external_resource: {{
        shape: parallelogram
    }}
    start_end: {{
        shape: oval
        style.italic: true
    }}
}}
Start: {{
    class: start_end
    desc: |md
      **Context**: {context}\
      **Input**: {input}
    |
}}
Start -> {id}: {input} {{
    class: edge
}}
End: {{
    class: start_end
    desc: |md
      **Output**: {output}
    |
}}
{id} -> End: {output} {{
    class: edge
}}
",
            context = escape_str(&self.get_type_name(context)),
            input = escape_str(&self.get_type_name(input)),
            output = escape_str(&self.get_type_name(output)),
        );

        self.process(desc, id, &mut res);

        res
    }

    fn process(&self, desc: &Description, id: u64, out: &mut String) {
        self.start_define_base(desc, id, out);

        let Description::Flow { base, nodes, edges } = desc else {
            out.push_str("}\n");
            return;
        };

        let nodes_and_ids = nodes
            .iter()
            .map(|node_desc| {
                let id = rand::random();
                self.process(node_desc, id, out);
                (id, node_desc.get_base_ref())
            })
            .collect::<Vec<_>>();

        writeln!(
            out,
            r"start: Start {{
                class: start_end
                desc: |md
                    **Context**: {context}\
                    **Input**: {input}
                |
            }}
            end: End {{
                class: start_end
                desc: |md
                    **Output**: {output}
                |
            }}",
            context = escape_str(&self.get_type_name(&base.context)),
            input = escape_str(&self.get_type_name(&base.input)),
            output = escape_str(&self.get_type_name(&base.output))
        )
        .unwrap();
        for Edge { start, end } in edges {
            let start_type = match start {
                EdgeEnding::ToFlow => {
                    out.push_str("start");
                    "\"\""
                }
                EdgeEnding::ToNode { node_index } => {
                    let node = &nodes_and_ids[*node_index];
                    out.push_str(&node.0.to_string());
                    &escape_str(&self.get_type_name(&node.1.output))
                }
            };
            out.push_str(" -> ");
            let end_type = match end {
                EdgeEnding::ToFlow => {
                    out.push_str("end");
                    "\"\""
                }
                EdgeEnding::ToNode { node_index } => {
                    let node = &nodes_and_ids[*node_index];
                    out.push_str(&node.0.to_string());
                    &escape_str(&self.get_type_name(&node.1.input))
                }
            };
            writeln!(
                out,
                r": {{
                    class: edge
                    source-arrowhead: {start_type}
                    target-arrowhead: {end_type}
                }}",
            )
            .unwrap();
        }

        out.push_str("}\n");
    }

    fn start_define_base(&self, desc: &Description, id: u64, out: &mut String) {
        let base = desc.get_base_ref();
        let is_node = matches!(desc, Description::Node { .. });
        writeln!(
            out,
            r"{}:{} {{
                class: {}",
            id,
            escape_str(&self.get_type_name(&base.r#type)),
            if is_node { "node" } else { "flow" }
        )
        .unwrap();

        let has_description = base.description.is_some() && self.show_description;
        let show_context = is_node && self.show_context_in_node && !base.context.name.is_empty();
        if has_description || show_context {
            writeln!(out, "desc: |md").unwrap();
            if show_context {
                writeln!(
                    out,
                    r"**Context**: {}<br/>",
                    escape_str(&self.get_type_name(&base.context))
                )
                .unwrap();
            }
            if has_description {
                out.push_str(&escape_str(base.description.as_ref().unwrap()));
            }
            writeln!(
                out,
                "
                | {{
                    class: node_flow_description
                }}",
            )
            .unwrap();
        }

        if !self.show_externals {
            return;
        }
        let Some(externals) = &base.externals else {
            return;
        };

        for ExternalResource {
            r#type,
            description,
            output,
        } in externals
        {
            let ext_id: u64 = rand::random();
            writeln!(
                out,
                r"{}:{} {{
                    class: external_resource
                    desc: |md
                        **output**: {}\
                        {}
                    |
                }}",
                ext_id,
                escape_str(&self.get_type_name(r#type)),
                escape_str(&self.get_type_name(output)),
                escape_str(description.as_ref().map(String::as_str).unwrap_or_default()),
            )
            .unwrap();
        }
    }
}
