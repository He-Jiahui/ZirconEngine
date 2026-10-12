#[path = "../../../../line/module/support_capabilities.rs"]
mod capabilities;
#[path = "../../../../support/parser/line/module/dispatch.rs"]
mod dispatch;
#[path = "../../../../support/parser/line/module/entry.rs"]
mod entry;
#[path = "../../../../support/parser/line/module/identity.rs"]
mod identity;
#[path = "../../../../support/parser/line/module/kind.rs"]
mod kind;
#[path = "../../../../support/parser/line/module/targets.rs"]
mod targets;

pub(in super::super) use entry::parse_optional_feature_module_line;
