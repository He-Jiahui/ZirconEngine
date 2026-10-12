#[path = "../../../../support/options/parser/line/dispatch.rs"]
mod dispatch;
#[path = "../../../../support/options/parser/line/identity.rs"]
mod identity;
#[path = "../../../../options/parser/line/requirement.rs"]
mod requirement;
#[path = "../../../../support/options/parser/line/value.rs"]
mod value;

pub(in crate::tests::manifest::support::options::parser) use dispatch::parse_option_manifest_line;
