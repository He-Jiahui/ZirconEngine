use super::*;
#[test]
fn light_cookie_feature_is_optional_and_registers_atlas_executor() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features().len(), 1);
    assert_eq!(report.extensions.render_features()[0].stage_passes.len(), 1);
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        ATLAS_BUILD_PASS
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
}
