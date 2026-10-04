use super::*;

#[test]
fn decals_feature_registers_projector_component_and_pass() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.extensions.components()[0].type_id,
        DECAL_PROJECTOR_COMPONENT_TYPE
    );
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        "decal-projector-composite"
    );
}
