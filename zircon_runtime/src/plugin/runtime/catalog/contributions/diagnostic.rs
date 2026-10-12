use crate::plugin::RuntimeExtensionRegistryError;

// 注册失败同时进入普通和 fatal 诊断：前者供展示，后者供 RuntimeExtensionCatalogReport 判定是否成功。
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
