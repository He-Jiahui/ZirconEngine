#[path = "../../../parser/line/requirement/dispatch.rs"]
mod dispatch;
#[path = "../../../line/requirement/required_capability.rs"]
mod required_capability;

pub(in crate::tests::manifest::support::options::parser::line) use dispatch::parse_option_requirement_line;
