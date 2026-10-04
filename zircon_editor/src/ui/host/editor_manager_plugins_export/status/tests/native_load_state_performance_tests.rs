#[test]
fn native_load_state_streams_loaded_plugin_checks() {
    let source = include_str!("../native_load_state.rs");
    let temporary_vector = [".collect::<", "Vec<_>>", "()"].concat();

    assert!(!source.contains(&temporary_vector));
}

#[test]
fn native_report_consumers_reuse_one_projection_per_operation() {
    for source in [
        include_str!("../../enablement/features.rs"),
        include_str!("../native.rs"),
        include_str!("../../native_registration/manager.rs"),
    ] {
        assert_eq!(source.matches("native_report.projection()").count(), 1);
        assert!(!source.contains("native_report.runtime_plugin_registration_reports()"));
        assert!(!source.contains("native_report.runtime_plugin_feature_registration_reports()"));
        assert!(!source.contains("native_report.diagnostics_for_plugin("));
    }

    let export = include_str!("../../export_build/manager.rs");
    let manifest_completion = include_str!("../../manifest_completion/native.rs");
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
