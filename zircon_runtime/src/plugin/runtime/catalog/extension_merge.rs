#[path = "extension_merge/diagnostic.rs"]
mod diagnostic;
#[path = "extension_merge/feature.rs"]
mod feature;
#[path = "extension_merge/runtime.rs"]
mod runtime;

pub(super) use feature::{merge_feature_extensions, merge_feature_extensions_for_target};
pub(super) use runtime::{merge_runtime_extensions, merge_runtime_extensions_for_target};
