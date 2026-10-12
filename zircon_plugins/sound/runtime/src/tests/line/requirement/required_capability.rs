#[path = "../required_capability/dispatch.rs"]
mod dispatch;
#[path = "../../parser/line/required_capability/field.rs"]
mod field;
#[path = "../../parser/line/required_capability/state.rs"]
mod state;
#[path = "../../parser/line/required_capability/value.rs"]
mod value;

pub(in crate::tests::manifest::support::options::parser::line) use dispatch::parse_option_required_capability_line;
