mod builder;
pub use builder::*;
mod chain_run;

use crate::flows::generic_defs::flow::define_flow;
use chain_run::ChainRunSequential as ChainRun;

define_flow!(
    SequentialFlow,
    ChainRun,
    Input: Send,
    Error: Send,
    /// Docs :)
);

