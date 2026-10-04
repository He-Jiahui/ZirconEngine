use super::*;

#[test]
fn post_process_feature_registers_legacy_pass_contract() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features()[0].name, FEATURE_NAME);
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0]
            .executor_id
            .as_str(),
        EXECUTOR_ID
    );
}
