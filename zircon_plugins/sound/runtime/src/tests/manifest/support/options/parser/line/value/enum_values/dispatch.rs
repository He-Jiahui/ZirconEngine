use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, values};

pub(in crate::tests::manifest::support::options::parser::line::value) fn parse_option_enum_values_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    let Some(value) = field::option_enum_values_value(line) else {
        return false;
    };
    state::set_option_enum_values(pending, values::option_enum_values_from_plugin_toml(value));
    true
}
