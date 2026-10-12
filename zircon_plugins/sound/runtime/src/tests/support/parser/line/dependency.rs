#[path = "../../../line/dependency/support_dispatch.rs"]
mod dispatch;
#[path = "dependency/entry.rs"]
mod entry;
#[path = "../../../parser/line/dependency/identity.rs"]
mod identity;
#[path = "dependency/primary.rs"]
mod primary;

pub(in super::super) use entry::parse_optional_feature_dependency_line;
