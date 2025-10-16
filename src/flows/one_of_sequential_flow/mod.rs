mod chain_run;

use crate::flows::generic_defs::define_flow_and_ioe_conv_builder;
use chain_run::ChainRunOneOfSequential as ChainRun;

define_flow_and_ioe_conv_builder!(
    OneOfSequentialFlow,
    ChainRun,
    >Input: Send + Clone,
    #NodeType: Send + Sync + Clone
    /// Docs :)
);
