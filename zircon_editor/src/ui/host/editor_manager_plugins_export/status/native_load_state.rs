use zircon_runtime::NativePluginLoadReport;

pub(super) fn native_load_state(report: &NativePluginLoadReport, plugin_id: &str) -> String {
    let loaded_plugins = report
        .loaded
        .iter()
        .filter(|plugin| plugin.plugin_id == plugin_id)
        .collect::<Vec<_>>();
    if !loaded_plugins.is_empty() {
        let diagnostics = report.diagnostics_for_plugin(plugin_id);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains(" entry failed:"))
        {
            return "entry failed".to_string();
        }
        if loaded_plugins
            .iter()
            .any(|plugin| plugin.descriptor.is_none())
        {
            return "loaded without descriptor".to_string();
        }
        if !diagnostics.is_empty() {
            return "loaded with diagnostics".to_string();
        }
        return "loaded".to_string();
    }
    let diagnostics = report.diagnostics_for_plugin(plugin_id);
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("library is missing"))
    {
        return "missing library".to_string();
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("failed to load"))
    {
        return "load failed".to_string();
    }
    "manifest only".to_string()
}
