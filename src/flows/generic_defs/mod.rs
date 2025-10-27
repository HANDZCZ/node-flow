pub mod debug;
pub mod flow;
pub mod ioe_conv_builder;

macro_rules! define_flow_and_ioe_conv_builder {
    ($flow_type:ident, $chain_run:ident, |$self:ident| $describe_code:block $(,>$global_param:ident: $global_bound0:ident $(+$global_bound:ident)*)* $(,#$fn_param:ident: $fn_bound0:ident $(+$fn_bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        define_flow_and_ioe_conv_builder!($flow_type, Builder, $chain_run, |$self| $describe_code $(,>$global_param: $global_bound0 $(+$global_bound)*)* $(,#$fn_param: $fn_bound0 $(+$fn_bound)*)* $(#[doc = $doc])*);
    };
    ($flow_type:ident, $builder:ident, $chain_run:ident, |$self:ident| $describe_code:block $(,>$global_param:ident: $global_bound0:ident $(+$global_bound:ident)*)* $(,#$fn_param:ident: $fn_bound0:ident $(+$fn_bound:ident)*)* $(,)? $(#[doc = $doc:expr])*) => {
        $crate::flows::generic_defs::flow::define_flow!($flow_type, $builder, $chain_run, |$self| $describe_code $(,$global_param: $global_bound0 $(+$global_bound)*)* $(#[doc = $doc])*);
        $crate::flows::generic_defs::ioe_conv_builder::define_builder!($flow_type $(,>$global_param: $global_bound0 $(+$global_bound)*)* $(,#$fn_param: $fn_bound0 $(+$fn_bound)*)*);
    };
}
pub(crate) use define_flow_and_ioe_conv_builder;
