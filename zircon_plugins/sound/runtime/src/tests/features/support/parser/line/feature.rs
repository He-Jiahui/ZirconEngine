#[path = "../../../../parser/line/feature/capabilities.rs"]
mod capabilities;
#[path = "../../../../support/parser/line/feature/defaults.rs"]
mod defaults;
#[path = "../../../../support/parser/line/feature/dispatch.rs"]
mod dispatch;
#[path = "../../../../support/parser/line/feature/entry.rs"]
mod entry;
#[path = "../../../../support/parser/line/feature/identity.rs"]
mod identity;

pub(in super::super) use entry::parse_optional_feature_line;
