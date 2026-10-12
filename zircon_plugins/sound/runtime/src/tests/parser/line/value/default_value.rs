#[path = "../../../line/value/default_value/dispatch.rs"]
mod dispatch;
#[path = "default_value/field.rs"]
mod field;
#[path = "default_value/state.rs"]
mod state;
#[path = "default_value/value.rs"]
mod value;

pub(super) use dispatch::parse_option_default_value_line;
