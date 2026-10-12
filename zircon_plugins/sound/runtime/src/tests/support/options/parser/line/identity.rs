#[path = "../../../../options/parser/line/identity/dispatch.rs"]
mod dispatch;
#[path = "../../../../parser/line/identity/display_name.rs"]
mod display_name;
#[path = "../../../../options/parser/line/identity/key.rs"]
mod key;

pub(in crate::tests::manifest::support::options::parser::line) use dispatch::parse_option_identity_line;
