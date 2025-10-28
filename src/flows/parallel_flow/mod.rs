mod builder;
pub use builder::*;
mod flow;
pub use flow::*;

use crate::flows::NodeResult;
mod chain_run;

pub trait Joiner<'a, Input, Output, Error, Context>: Send + Sync {
    fn join(
        &self,
        input: Input,
        context: &'a mut Context,
    ) -> impl Future<Output = NodeResult<Output, Error>> + Send;
}

impl<'a, Input, Output, Error, Context, T, F> Joiner<'a, Input, Output, Error, Context> for T
where
    Input: Send,
    F: Future<Output = NodeResult<Output, Error>> + Send + 'a,
    T: Fn(Input, &'a mut Context) -> F + Send + Sync,
    Context: 'a,
{
    fn join(
        &self,
        input: Input,
        context: &'a mut Context,
    ) -> impl Future<Output = NodeResult<Output, Error>> {
        (self)(input, context)
    }
}
