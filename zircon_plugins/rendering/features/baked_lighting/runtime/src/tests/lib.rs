use super::*;

#[test]
fn baked_lighting_feature_registers_composite_pass() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report.manifest.enabled_by_default);
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        "baked-lighting-composite"
    );
}
