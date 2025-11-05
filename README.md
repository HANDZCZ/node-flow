# Node Flow

[![Crates.io Version](https://img.shields.io/crates/v/node-flow?style=for-the-badge)](https://crates.io/crates/node-flow)
[![DOCS](https://img.shields.io/badge/docs-node_flow?style=for-the-badge&logo=docs.rs&labelColor=%23567&color=%233A3)](https://handzcz.github.io/node-flow/)

**Node Flow** is runtime-agnostic, composable, asynchronous node-based framework for building
structured and reusable data processing pipelines, workflows, or control flows.

The core idea is that each **node** represents a self-contained asynchronous operation,
and **flows** define how multiple nodes are composed and executed.

## Example

```rust
use node_flow::node::{Node, NodeOutput};
use node_flow::flows::SequentialFlow;

// Example node
#[derive(Clone)]
struct AddOne;

struct ExampleCtx;

impl<Ctx: Send> Node<u8, NodeOutput<u8>, (), Ctx> for AddOne {
    async fn run(&mut self, input: u8, _: &mut Ctx) -> Result<NodeOutput<u8>, ()> {
        Ok(NodeOutput::Ok(input + 1))
    }
}

async fn main() {
    let mut flow = SequentialFlow::<u8, u8, (), _>::builder()
        .add_node(AddOne)
        .add_node(AddOne)
        .add_node(AddOne)
        .build();

    let mut ctx = ExampleCtx;
    let result = flow.run(5u8, &mut ctx).await;
    assert_eq!(result, Ok(NodeOutput::Ok(8)));
}
```

## When to use Node Flow

Use this crate when you need:
- Composable async control flows (e.g., fallback chains, parallel processing).
- Declarative and type-safe node composition.
- Inspectable or visualizable flow structures.
