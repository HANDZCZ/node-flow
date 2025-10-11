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
// Type defs

use defs::*;
use node_flow::{
    flows::{OneOfSequentialFlow, SequentialFlow},
    node::Node,
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
        O: Send,
        E: Send,
    {
        // Should complain Send not implemented for Self type
        async fn run_with_storage(
            &mut self,
            input: I,
            _storage: &mut node_flow::storage::Storage,
        ) -> Result<NodeOutput<O>, E> {
            Ok(NodeOutput::Ok(input.into()))
        }
    }
}
