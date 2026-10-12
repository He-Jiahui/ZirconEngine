#[path = "../../../../support/parser/pending/module/append.rs"]
mod append;
#[path = "../../../../support/parser/pending/module/entry.rs"]
mod entry;
#[path = "../../../../parser/module/support_signature.rs"]
mod signature;

pub(in super::super) use entry::push_optional_feature_module;
