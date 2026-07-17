use zircon_runtime::plugin::native::NativePluginLoadReport;

pub(super) fn native_load_state(report: &NativePluginLoadReport, plugin_id: &str) -> String {
    let has_loaded_plugin = report
        .loaded
        .iter()
        .any(|plugin| plugin.plugin_id == plugin_id);
    if has_loaded_plugin {
        let diagnostics = report.diagnostics_for_plugin(plugin_id);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains(" entry failed:"))
        {
            return "entry failed".to_string();
        }
        if report
            .loaded
            .iter()
            .any(|plugin| plugin.plugin_id == plugin_id && plugin.descriptor.is_none())
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn native_load_state_streams_loaded_plugin_checks() {
        let source = include_str!("native_load_state.rs");
        let temporary_vector = [".collect::<", "Vec<_>>", "()"].concat();

        assert!(!source.contains(&temporary_vector));
    }
}
