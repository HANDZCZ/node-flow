use super::design::{Description, Edge, EdgeEnding, ExternalResource, Type};
use std::{borrow::Cow, fmt::Write};

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct D2Describer {
    pub simple_type_name: bool,
    pub show_context_in_node: bool,
    pub show_description: bool,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
