use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, value};

pub(in crate::tests::manifest::support::options::parser::line::value) fn parse_option_default_value_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    let Some(value) = field::option_default_value(line) else {
        return false;
    };
    state::set_option_default_value(pending, value::option_default_from_plugin_toml(value));
    true
}
