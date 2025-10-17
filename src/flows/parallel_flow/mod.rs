mod builder;
pub use builder::*;
mod flow;
pub use flow::*;

use crate::{flows::NodeResult, storage::Storage};
mod chain_run;

pub trait Joiner<'a, Input, Output, Error>: Send + Sync {
    fn join(
        &self,
        input: Input,
        storage: &'a mut Storage,
    ) -> impl Future<Output = NodeResult<Output, Error>> + Send;
}

impl<'a, Input, Output, Error, T, F> Joiner<'a, Input, Output, Error> for T
where
    Input: Send,
    T: Send + Sync,
    F: Future<Output = NodeResult<Output, Error>> + Send + 'a,
    T: Fn(Input, &'a mut Storage) -> F,
{
    fn join(
        &self,
        input: Input,
        storage: &'a mut Storage,
    ) -> impl Future<Output = NodeResult<Output, Error>> {
        (self)(input, storage)
    }
}
