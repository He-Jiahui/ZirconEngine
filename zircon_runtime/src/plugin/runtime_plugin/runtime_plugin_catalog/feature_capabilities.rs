mod base;
mod declaration;
mod feature;

pub(super) use base::base_capabilities_for_target;
pub(super) use declaration::feature_declares_capability_for_target;
pub(super) use feature::feature_capabilities_for_target;
