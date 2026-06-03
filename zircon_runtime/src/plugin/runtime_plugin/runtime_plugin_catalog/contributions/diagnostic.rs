use crate::plugin::RuntimeExtensionRegistryError;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn push_runtime_extension_result(
    result: Result<(), RuntimeExtensionRegistryError>,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    if let Err(error) = result {
        let diagnostic = error.to_string();
        diagnostics.push(diagnostic.clone());
        fatal_diagnostics.push(diagnostic);
    }
}
