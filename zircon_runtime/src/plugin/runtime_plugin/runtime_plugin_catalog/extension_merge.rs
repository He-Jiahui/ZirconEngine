mod diagnostic;
mod feature;
mod runtime;

pub(super) use feature::{merge_feature_extensions, merge_feature_extensions_for_target};
pub(super) use runtime::{merge_runtime_extensions, merge_runtime_extensions_for_target};
