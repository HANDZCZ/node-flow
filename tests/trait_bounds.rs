#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
// Used for testing error messages when trait bounds are not satisfied
// This test wan not designed to be ran
// You can disable flows from throwing error when testing different flows by adding some cfg like #[cfg(doc)]

// artificially added implementations
// TestNode
unsafe impl<I, O, E> Send for TestNode<I, O, E> {}
unsafe impl<I, O, E> Sync for TestNode<I, O, E> {}
impl<I, O, E> Clone for TestNode<I, O, E> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

// SomeData
unsafe impl Send for SomeData {}
impl Clone for SomeData {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

// ----------------------------------------------------------------------------------------------------------------

// SequentialFlow

async fn test_sequential_flow() {
    let mut storage = Storage::new();

    // Node test
    let _res = SequentialFlow::<u8, u128, ()>::builder()
        // Should complain about Clone not implemented
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run_with_storage(5u8, &mut storage)
        .await;

    // IOE test
    // Should complain about Send not implemented
    let _res = SequentialFlow::<SomeData, (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
    // Should not complain
    let _res = SequentialFlow::<(), SomeData, ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;
    // Should complain about Send not implemented
    let _res = SequentialFlow::<(), (), SomeData>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;

    // Node IOE test
    let _res = SequentialFlow::<(), (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// OneOfSequentialFlow

async fn test_one_of_sequential_flow() {
    let mut storage = Storage::new();

    // Node test
    let _res = OneOfSequentialFlow::<u8, u128, ()>::builder()
        // Should complain about Clone not implemented
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run_with_storage(5u8, &mut storage)
        .await;

    // IOE test
    // Should complain about Send not implemented
    let _res = OneOfSequentialFlow::<SomeData, (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
    // Should not complain
    let _res = OneOfSequentialFlow::<(), SomeData, ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;
    // Should not complain
    let _res = OneOfSequentialFlow::<(), (), SomeData>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;

    // Node IOE test
    let _res = OneOfSequentialFlow::<(), (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// OneOfParallelFlow

async fn test_one_of_parallel_flow() {
    let mut storage = Storage::new();

    // Node test
    let _res = OneOfParallelFlow::<u8, u128, ()>::builder()
        // Should complain about Clone not implemented
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run_with_storage(5u8, &mut storage)
        .await;

    // IOE test
    // Should complain about Send not implemented
    let _res = OneOfParallelFlow::<SomeData, (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
    // Should not complain
    let _res = OneOfParallelFlow::<(), SomeData, ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;
    // Should not complain
    let _res = OneOfParallelFlow::<(), (), SomeData>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run_with_storage((), &mut storage)
        .await;

    // Node IOE test
    let _res = OneOfParallelFlow::<(), (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run_with_storage(().into(), &mut storage)
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// ParallelFlow

async fn test_parallel_flow() {
    let mut storage = Storage::new();

    // Node test
    let _res = ParallelFlow::<u8, u128, ()>::builder()
        // Should complain about Clone not implemented
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build(async |_, _: &mut Storage| Ok(NodeOutput::SoftFail))
        .run_with_storage(5u8, &mut storage)
        .await;

    // IOE test
    // Should complain about Send not implemented
    let _res = ParallelFlow::<SomeData, (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut Storage| Ok(NodeOutput::SoftFail))
        .run_with_storage(().into(), &mut storage)
        .await;
    // Should not complain
    let _res = ParallelFlow::<(), SomeData, ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut Storage| Ok(NodeOutput::SoftFail))
        .run_with_storage((), &mut storage)
        .await;
    // Should not complain
    let _res = ParallelFlow::<(), (), SomeData>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut Storage| Ok(NodeOutput::SoftFail))
        .run_with_storage((), &mut storage)
        .await;

    // Node IOE test
    let _res = ParallelFlow::<(), (), ()>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build(async |_, _: &mut Storage| Ok(NodeOutput::SoftFail))
        .run_with_storage(().into(), &mut storage)
        .await;
}

// ----------------------------------------------------------------------------------------------------------------
// Type defs

use defs::*;
use node_flow::{
    flows::{OneOfParallelFlow, OneOfSequentialFlow, ParallelFlow, SequentialFlow},
    node::{Node, NodeOutput},
    storage::Storage,
};
mod defs {
    use std::{cell::UnsafeCell, marker::PhantomData};

    use node_flow::node::{Node, NodeOutput};

    // Some type that doesn't implement Send, Sync and Clone
    pub struct SomeData(UnsafeCell<*const ()>);
    impl From<()> for SomeData {
        fn from(value: ()) -> Self {
            unimplemented!()
        }
    }
    impl From<SomeData> for () {
        fn from(value: SomeData) -> Self {
            unimplemented!()
        }
    }

    // Node type that doesn't implement Send, Sync and Clone
    pub struct TestNode<I, O, E = ()>(PhantomData<(I, O, E)>, UnsafeCell<*const ()>);

    impl<I, O, E> TestNode<I, O, E> {
        pub fn new() -> Self {
            unimplemented!()
        }
    }

    impl<I, O, E> Node<I, NodeOutput<O>, E> for TestNode<I, O, E>
    where
        I: Into<O> + Send,
    {
        // Will always yell, but it is needed in this form for Node IOE tests
        async fn run_with_storage(
            &mut self,
            input: I,
            _storage: &mut node_flow::storage::Storage,
        ) -> Result<NodeOutput<O>, E> {
            Ok(NodeOutput::Ok(input.into()))
        }
    }
}
