use zircon_runtime::plugin::native::NativePluginLoadProjection;

pub(super) fn native_load_state(
    projection: &NativePluginLoadProjection,
    plugin_id: &str,
) -> String {
    if projection.is_loaded(plugin_id) {
        let diagnostics = projection.diagnostics_for_plugin(plugin_id);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains(" entry failed:"))
        {
            return "entry failed".to_string();
        }
        if !projection.has_descriptor(plugin_id) {
            return "loaded without descriptor".to_string();
        }
        if !diagnostics.is_empty() {
            return "loaded with diagnostics".to_string();
        }
        return "loaded".to_string();
    }
    let diagnostics = projection.diagnostics_for_plugin(plugin_id);
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

    #[test]
    fn native_report_consumers_reuse_one_projection_per_operation() {
        for source in [
            include_str!("../enablement/features.rs"),
            include_str!("native.rs"),
            include_str!("../native_registration/manager.rs"),
        ] {
            assert_eq!(source.matches("native_report.projection()").count(), 1);
            assert!(!source.contains("native_report.runtime_plugin_registration_reports()"));
            assert!(!source.contains("native_report.runtime_plugin_feature_registration_reports()"));
            assert!(!source.contains("native_report.diagnostics_for_plugin("));
        }

        let export = include_str!("../export_build/manager.rs");
        let manifest_completion = include_str!("../manifest_completion/native.rs");
        assert_eq!(export.matches("native_report.projection()").count(), 1);
        assert_eq!(
            export
                .matches("exported_native_report.projection()")
                .count(),
            1
        );
        assert!(!export.contains("native_report.descriptor_diagnostics()"));
        assert!(!export.contains("native_report.entry_diagnostics()"));
        assert!(export.contains("complete_project_plugin_manifest_with_native_projection("));
        assert!(!export.contains("complete_project_plugin_manifest_with_native_report("));
        assert_eq!(
            manifest_completion
                .matches("native_report.projection()")
                .count(),
            1
        );
        let projected_completion = manifest_completion
            .split_once("fn complete_project_plugin_manifest_with_native_projection")
            .expect("projection completion helper")
            .1;
        assert!(!projected_completion.contains("NativePluginLoadReport"));
        assert!(!projected_completion.contains("native_report."));
    }
}
