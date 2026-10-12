#[path = "feature/entry.rs"]
mod entry;
#[path = "../../../parser/feature/support_normalize.rs"]
mod normalize;
#[path = "feature/output.rs"]
mod output;
#[path = "../../../parser/feature/support_static_manifest.rs"]
mod static_manifest;

pub(in super::super) use entry::push_optional_feature;
