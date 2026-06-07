mod dispatch;
mod identity;
mod primary;

pub(in super::super) fn parse_optional_feature_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    dispatch::parse_dependency_line(line, plugin_id, capability, primary);
}
