#[path = "../../../parser/dependency/support_append.rs"]
mod append;
#[path = "../../../parser/dependency/support_entry.rs"]
mod entry;
#[path = "../../../parser/dependency/support_signature.rs"]
mod signature;

pub(in super::super) use entry::push_optional_feature_dependency;
