use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, value};

pub(in crate::tests::manifest::support::options::parser::line::identity) fn parse_option_display_name_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    let Some(value) = field::option_display_name_value(line) else {
        return false;
    };
    state::set_option_display_name(pending, value::option_display_name_from_plugin_toml(value));
    true
}
