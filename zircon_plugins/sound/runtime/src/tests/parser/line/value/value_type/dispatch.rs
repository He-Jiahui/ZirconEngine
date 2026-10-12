use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, value};

pub(in crate::tests::manifest::support::options::parser::line::value) fn parse_option_value_type_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    let Some(value) = field::option_value_type_value(line) else {
        return false;
    };
    state::set_option_value_type(pending, value::option_value_type_from_plugin_toml(value));
    true
}
