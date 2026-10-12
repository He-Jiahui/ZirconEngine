#[path = "../display_name/dispatch.rs"]
mod dispatch;
#[path = "../../../options/parser/line/display_name/field.rs"]
mod field;
#[path = "../../../options/parser/line/display_name/state.rs"]
mod state;
#[path = "../../../options/parser/line/display_name/value.rs"]
mod value;

pub(in crate::tests::manifest::support::options::parser::line::identity) use dispatch::parse_option_display_name_line;
