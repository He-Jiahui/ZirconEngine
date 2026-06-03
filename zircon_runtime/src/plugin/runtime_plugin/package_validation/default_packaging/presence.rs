use crate::plugin::ExportPackagingStrategy;

pub(super) fn validate_runtime_plugin_default_packaging_presence(
    owner: &str,
    default_packaging: &[ExportPackagingStrategy],
    diagnostics: &mut Vec<String>,
) {
    if default_packaging.is_empty() {
        diagnostics.push(format!(
            "runtime plugin {owner} default_packaging must declare at least one export packaging strategy"
        ));
    }
}
