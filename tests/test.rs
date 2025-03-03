use async_trait::async_trait;
use node_flow::{
    flows::{OneOfSequentialFlow, SequentialFlow},
    impl_node_output,
    node::{Node, NodeOutput},
    storage::Storage,
};

#[derive(Clone, Debug)]
struct StringMatcher(&'static str);
struct StringMatcherError;

impl From<StringMatcherError> for String {
    fn from(_value: StringMatcherError) -> Self {
        std::any::type_name::<Self>().into()
    }
}

#[async_trait]
impl Node<String, NodeOutput<String>, StringMatcherError> for StringMatcher {
    async fn run_with_storage<'input>(
        &mut self,
        input: String,
        _storage: &mut Storage,
    ) -> Result<NodeOutput<String>, StringMatcherError> {
        if !input.contains(self.0) {
            return Ok(NodeOutput::SoftFail);
        }
        Ok(NodeOutput::Ok(input))
    }
}

#[derive(Clone, Debug)]
struct StringForwarder;
struct StringForwarderError;

impl From<StringForwarderError> for String {
    fn from(_value: StringForwarderError) -> Self {
        std::any::type_name::<Self>().into()
    }
}

#[async_trait]
impl Node<String, String, StringForwarderError> for StringForwarder {
    async fn run_with_storage<'input>(
        &mut self,
        input: String,
        _storage: &mut Storage,
    ) -> Result<String, StringForwarderError> {
        Ok(input)
    }
}
impl_node_output!(StringForwarder, String, String, StringForwarderError);

struct WrapString(String);
impl From<String> for WrapString {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<WrapString> for String {
    fn from(value: WrapString) -> Self {
        value.0
    }
}

#[derive(Clone, Debug)]
struct StringToWrapString;

#[async_trait]
impl Node<String, WrapString, String> for StringToWrapString {
    async fn run_with_storage<'input>(
        &mut self,
        input: String,
        _storage: &mut Storage,
    ) -> Result<WrapString, String> {
        Ok(WrapString(input))
    }
}
impl_node_output!(StringToWrapString, String, WrapString, String);

#[derive(Clone, Debug)]
struct WrapStringToString;

#[async_trait]
impl Node<WrapString, String, String> for WrapStringToString {
    async fn run_with_storage<'input>(
        &mut self,
        input: WrapString,
        _storage: &mut Storage,
    ) -> Result<String, String> {
        Ok(input.0)
    }
}
impl_node_output!(WrapStringToString, WrapString, String, String);

//----------------------------------------------------------------------------------------------------------------

// SequentialFlow

#[tokio::test]
async fn sequential_flow_success() {
    let mut flow = SequentialFlow::<String, String, String>::builder()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .build();
    let mut storage = Storage::new();
    let res = flow.run_with_storage("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}

#[tokio::test]
async fn sequential_flow_soft_fail() {
    let mut flow = SequentialFlow::<String, String, String>::builder()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .build();
    let mut storage = Storage::new();
    let res = flow.run_with_storage("".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::SoftFail));
}

#[tokio::test]
async fn sequential_flow_io_conversion_success() {
    let mut flow = SequentialFlow::<String, String, String>::builder()
        // convert
        .add_node(WrapStringToString)
        .add_node(StringForwarder)
        .add_node(StringToWrapString)
        // convert
        .add_node(StringForwarder)
        // convert
        .add_node(WrapStringToString)
        .add_node(StringForwarder)
        .add_node(StringToWrapString)
        // convert
        .build();
    let mut storage = Storage::new();
    let res = flow.run_with_storage("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}
