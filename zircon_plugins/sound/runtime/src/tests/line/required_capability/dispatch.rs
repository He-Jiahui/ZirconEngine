use super::super::super::super::super::state::PendingOptionManifest;
use super::{field, state, value};

pub(in crate::tests::manifest::support::options::parser::line) fn parse_option_required_capability_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) {
    let Some(value) = field::option_required_capability_value(line) else {
        return;
    };
    state::set_option_required_capability(
        pending,
        value::option_required_capability_from_plugin_toml(value),
    );
}
