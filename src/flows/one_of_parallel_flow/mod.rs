mod chain_run;

use crate::{
    flows::generic_defs::define_flow_and_ioe_conv_builder, node::NodeOutput as NodeOutputStruct,
    storage::Storage,
};
use chain_run::ChainRunOneOfParallel as ChainRun;

type FutOutput<Output, Error> = Result<(NodeOutputStruct<Output>, Storage), Error>;

define_flow_and_ioe_conv_builder!(
    OneOfParallelFlow,
    ChainRun,
    >Input: Send + Clone,
    #NodeType: Send + Sync + Clone
    /// Docs :)
);
