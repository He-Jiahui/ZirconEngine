use super::fields::RuntimePluginPackageCoordinateFields;

pub(super) fn validate_runtime_plugin_package_coordinate_completeness(
    fields: &RuntimePluginPackageCoordinateFields<'_>,
    diagnostics: &mut Vec<String>,
) {
    if fields.declares_any() && !fields.declares_all() {
        diagnostics.push(
            "runtime plugin package manifest package coordinates must declare package_prefix, package_company, and package_name together or leave all empty"
                .to_string(),
        );
    }
}
