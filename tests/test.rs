use node_flow::{
    context::storage::local_storage::LocalStorageImpl,
    flows::{OneOfSequentialFlow, SequentialFlow},
    impl_node_output,
    node::{Node, NodeOutput},
};

#[derive(Clone, Debug)]
struct StringMatcher(&'static str);
struct StringMatcherError;

impl From<StringMatcherError> for String {
    fn from(_value: StringMatcherError) -> Self {
        std::any::type_name::<Self>().into()
    }
}

impl<C> Node<String, NodeOutput<String>, StringMatcherError, C> for StringMatcher
where
    C: Send,
{
    async fn run(
        &mut self,
        input: String,
        _context: &mut C,
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

impl<C: Send> Node<String, String, StringForwarderError, C> for StringForwarder {
    async fn run(
        &mut self,
        input: String,
        _context: &mut C,
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

impl<C: Send> Node<String, WrapString, String, C> for StringToWrapString {
    async fn run(&mut self, input: String, _context: &mut C) -> Result<WrapString, String> {
        Ok(WrapString(input))
    }
}
impl_node_output!(StringToWrapString, String, WrapString, String);

#[derive(Clone, Debug)]
struct WrapStringToString;

impl<C: Send> Node<WrapString, String, String, C> for WrapStringToString {
    async fn run(&mut self, input: WrapString, _context: &mut C) -> Result<String, String> {
        Ok(input.0)
    }
}
impl_node_output!(WrapStringToString, WrapString, String, String);

#[derive(Clone, Debug)]
struct ForwarderT;

impl<T: Send + 'static, C: Send> Node<T, T, String, C> for ForwarderT {
    async fn run(&mut self, input: T, _context: &mut C) -> Result<T, String> {
        Ok(input)
    }
}
impl<T: Send + 'static, C: Send> Node<T, NodeOutput<T>, String, C> for ForwarderT {
    async fn run(&mut self, input: T, _context: &mut C) -> Result<NodeOutput<T>, String> {
        Ok(NodeOutput::Ok(input))
    }
}

//----------------------------------------------------------------------------------------------------------------

// SequentialFlow

#[tokio::test]
async fn sequential_flow_success() {
    let mut flow = SequentialFlow::<String, String, String, _>::builder()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .build();
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}

#[tokio::test]
async fn sequential_flow_soft_fail() {
    let mut flow = SequentialFlow::<String, String, String, _>::builder()
        .add_node(StringMatcher("match"))
        .add_node(StringForwarder)
        .build();
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::SoftFail));
}

#[tokio::test]
async fn sequential_flow_io_conversion_success() {
    let mut flow = SequentialFlow::<String, String, String, _>::builder()
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
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}

//----------------------------------------------------------------------------------------------------------------

// OneOfSequentialFlow

#[tokio::test]
async fn one_of_sequential_flow_success() {
    let mut flow = OneOfSequentialFlow::<String, String, String, _>::builder()
        .add_node(StringMatcher("nope"))
        .add_node(StringMatcher("still no"))
        .add_node(StringMatcher("hmm no"))
        .add_node(StringForwarder)
        .build();
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}

#[tokio::test]
async fn one_of_sequential_flow_soft_fail() {
    let mut flow = OneOfSequentialFlow::<String, String, String, _>::builder()
        .add_node(StringMatcher("match"))
        .add_node(StringMatcher("match"))
        .add_node(StringMatcher("match"))
        .build();
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::SoftFail));
}

#[tokio::test]
async fn one_of_sequential_flow_io_conversion_success() {
    let mut flow = OneOfSequentialFlow::<String, String, String, _>::builder()
        .add_node::<_, WrapString, _, _>(ForwarderT)
        .build();
    let mut storage = LocalStorageImpl::new();
    let res = flow.run("match".into(), &mut storage).await;
    assert_eq!(res, Ok(NodeOutput::Ok("match".to_owned())));
}
