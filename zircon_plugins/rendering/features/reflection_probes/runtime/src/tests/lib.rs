use super::*;

#[test]
fn reflection_probes_feature_has_no_unrequested_capture_or_composite_pass() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report.manifest.enabled_by_default);
    assert!(
        report.extensions.render_features()[0]
            .stage_passes
            .is_empty()
    );
    assert!(report.extensions.render_pass_executors().is_empty());
}
