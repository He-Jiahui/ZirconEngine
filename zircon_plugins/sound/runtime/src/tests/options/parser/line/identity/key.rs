#[path = "../key/dispatch.rs"]
mod dispatch;
#[path = "../../../../support/options/parser/line/key/field.rs"]
mod field;
#[path = "../../../../support/options/parser/line/key/state.rs"]
mod state;
#[path = "../../../../support/options/parser/line/key/value.rs"]
mod value;

pub(in crate::tests::manifest::support::options::parser::line::identity) use dispatch::parse_option_key_line;
