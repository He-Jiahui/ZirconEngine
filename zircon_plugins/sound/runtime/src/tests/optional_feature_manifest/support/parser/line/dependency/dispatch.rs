use super::{identity, primary};

pub(super) fn parse_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    if identity::parse_dependency_identity_line(line, plugin_id, capability) {
        return;
    }

    if primary::parse_dependency_primary_line(line, primary) {
        return;
    }
}
