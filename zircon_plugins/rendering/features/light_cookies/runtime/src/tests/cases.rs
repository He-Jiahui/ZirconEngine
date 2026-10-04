//! 测试约束图集通道通过显式图依赖服务后续材质通道；不将图集建立视为无条件副作用。
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
    assert!(
        !report.extensions.render_features()[0].stage_passes[0]
            .flags
            .has_side_effects,
        "mesh and deferred lighting passes consume the cookie atlas through the graph"
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
}
