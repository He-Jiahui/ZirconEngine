use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, value};

pub(in crate::tests::manifest::support::options::parser::line::identity) fn parse_option_key_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    let Some(value) = field::option_key_value(line) else {
        return false;
    };
    state::set_option_key(pending, value::option_key_from_plugin_toml(value));
    true
}
