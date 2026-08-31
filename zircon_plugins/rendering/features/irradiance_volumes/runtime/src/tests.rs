use super::*;

#[test]
fn irradiance_volume_feature_is_optional_and_registers_binding_executor() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features().len(), 1);
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        VOLUME_BIND_PASS
    );
    assert!(
        !report.extensions.render_features()[0].stage_passes[0]
            .flags
            .has_side_effects,
        "opaque and deferred material passes consume the bound irradiance volume resource"
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
}
