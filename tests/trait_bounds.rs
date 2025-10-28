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
unsafe impl Sync for SomeData {}
impl Clone for SomeData {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

// DummyContext
unsafe impl Send for DummyContext {}
impl Fork for DummyContext {
    fn fork(&self) -> Self {
        unimplemented!()
    }
}
impl Update for DummyContext {
    fn update_from(&mut self, other: Self) {
        unimplemented!()
    }
}
impl Join for DummyContext {
    fn join(&mut self, others: Box<[Self]>) {
        unimplemented!()
    }
}

// ----------------------------------------------------------------------------------------------------------------

// SequentialFlow

// #[cfg(doc)]
async fn test_sequential_flow() {
    let mut storage = LocalStorageImpl::new();

    // Node test
    let _res = SequentialFlow::<u8, u128, (), _>::builder()
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run(5u8, &mut storage)
        .await;

    // IOE test
    let _res = SequentialFlow::<SomeData, (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run(().into(), &mut storage)
        .await;
    let _res = SequentialFlow::<(), SomeData, (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;
    let _res = SequentialFlow::<(), (), SomeData, _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;

    // Node IOE test
    let _res = SequentialFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run(().into(), &mut storage)
        .await;

    // Context test
    let _res = SequentialFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut DummyContext::new())
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// OneOfSequentialFlow

// #[cfg(doc)]
async fn test_one_of_sequential_flow() {
    let mut storage = LocalStorageImpl::new();

    // Node test
    let _res = OneOfSequentialFlow::<u8, u128, (), _>::builder()
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run(5u8, &mut storage)
        .await;

    // IOE test
    let _res = OneOfSequentialFlow::<SomeData, (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run(().into(), &mut storage)
        .await;
    let _res = OneOfSequentialFlow::<(), SomeData, (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;
    let _res = OneOfSequentialFlow::<(), (), SomeData, _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;

    // Node IOE test
    let _res = OneOfSequentialFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run(().into(), &mut storage)
        .await;

    // Context test
    let _res = OneOfSequentialFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut DummyContext::new())
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// OneOfParallelFlow

// #[cfg(doc)]
async fn test_one_of_parallel_flow() {
    let mut storage = LocalStorageImpl::new();

    // Node test
    let _res = OneOfParallelFlow::<u8, u128, (), _>::builder()
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build()
        .run(5u8, &mut storage)
        .await;

    // IOE test
    let _res = OneOfParallelFlow::<SomeData, (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run(().into(), &mut storage)
        .await;
    let _res = OneOfParallelFlow::<(), SomeData, (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;
    let _res = OneOfParallelFlow::<(), (), SomeData, _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut storage)
        .await;

    // Node IOE test
    let _res = OneOfParallelFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build()
        .run(().into(), &mut storage)
        .await;

    // Context test
    let _res = OneOfParallelFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build()
        .run((), &mut DummyContext::new())
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// ParallelFlow

// #[cfg(doc)]
async fn test_parallel_flow() {
    let mut storage = LocalStorageImpl::new();

    // Node test
    let _res = ParallelFlow::<u8, u128, (), _>::builder()
        .add_node(TestNode::<u16, u16>::new())
        .add_node(TestNode::<u32, u64>::new())
        .build(async |_, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail))
        .run(5u8, &mut storage)
        .await;

    // IOE test
    let _res = ParallelFlow::<SomeData, (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail))
        .run(().into(), &mut storage)
        .await;
    let _res = ParallelFlow::<(), SomeData, (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail))
        .run((), &mut storage)
        .await;
    let _res = ParallelFlow::<(), (), SomeData, _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail))
        .run((), &mut storage)
        .await;

    // Node IOE test
    let _res = ParallelFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<SomeData, ()>::new())
        .add_node(TestNode::<(), SomeData>::new())
        .add_node(TestNode::<(), (), SomeData>::new())
        .build(async |_, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail))
        .run(().into(), &mut storage)
        .await;

    // Context test
    let _res = ParallelFlow::<(), (), (), _>::builder()
        .add_node(TestNode::<(), ()>::new())
        .add_node(TestNode::<(), ()>::new())
        .build(async |_, _: &mut _| Ok(NodeOutput::SoftFail))
        .run((), &mut DummyContext::new())
        .await;
}

// ----------------------------------------------------------------------------------------------------------------

// FnFlow

// #[cfg(doc)]
async fn test_fn_flow() {
    let mut storage = LocalStorageImpl::new();

    // IOE test
    let _res = FnFlow::<SomeData, (), (), _>::new(
        (5u8, String::new(), 120usize),
        async |_, _, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail),
    )
    .run(().into(), &mut storage)
    .await;
    let _res = FnFlow::<(), SomeData, (), _>::new(
        (5u8, String::new(), 120usize),
        async |_, _, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail),
    )
    .run((), &mut storage)
    .await;
    let _res = FnFlow::<(), (), SomeData, _>::new(
        (5u8, String::new(), 120usize),
        async |_, _, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail),
    )
    .run((), &mut storage)
    .await;

    // Context test
    let _res = FnFlow::<(), (), (), _>::new(
        (5u8, String::new(), 120usize),
        async |_, _, _: &mut DummyContext| Ok(NodeOutput::SoftFail),
    )
    .run((), &mut DummyContext::new())
    .await;

    // Inner data test
    let _res = FnFlow::<(), (), (), _>::new(
        (5u8, String::new(), 120usize, SomeData::new()),
        async |_, _, _: &mut LocalStorageImpl| Ok(NodeOutput::SoftFail),
    )
    .run((), &mut storage)
    .await;
}

// ----------------------------------------------------------------------------------------------------------------
// Type defs

use defs::*;
use node_flow::{
    context::{Fork, Join, Update, storage::local_storage::LocalStorageImpl},
    flows::{FnFlow, OneOfParallelFlow, OneOfSequentialFlow, ParallelFlow, SequentialFlow},
    node::{Node, NodeOutput},
};
mod defs {
    use std::{cell::UnsafeCell, marker::PhantomData};

    use node_flow::node::{Node, NodeOutput};

    // Some type that doesn't implement Send, Sync and Clone
    pub struct SomeData(UnsafeCell<*const ()>);
    impl SomeData {
        pub fn new() -> Self {
            unimplemented!()
        }
    }
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

    impl<I, O, E, C> Node<I, NodeOutput<O>, E, C> for TestNode<I, O, E>
    where
        I: Into<O> + Send,
        C: Send,
    {
        async fn run(&mut self, input: I, _context: &mut C) -> Result<NodeOutput<O>, E> {
            Ok(NodeOutput::Ok(input.into()))
        }
    }

    pub struct DummyContext(UnsafeCell<*const ()>);

    impl DummyContext {
        pub fn new() -> Self {
            unimplemented!()
        }
    }
}
