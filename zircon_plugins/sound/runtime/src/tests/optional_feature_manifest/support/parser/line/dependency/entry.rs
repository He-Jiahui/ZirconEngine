pub(in super::super::super) fn parse_optional_feature_dependency_line(
    line: &str,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    super::dispatch::parse_dependency_line(line, plugin_id, capability, primary);
}
