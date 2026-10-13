//! 测试约束图集通道通过显式图依赖服务后续材质通道；不将图集建立视为无条件副作用。
use super::*;
#[test]
fn light_cookie_feature_is_optional_and_registers_atlas_executor() {
    let report = plugin_feature_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features().len(), 1);
    let feature = &report.extensions.render_features()[0];
    assert_eq!(feature.name, FEATURE_NAME);
    assert_eq!(
        feature.required_extract_sections,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "advanced_lighting".to_string(),
        ]
    );
    assert_eq!(feature.stage_passes.len(), 1);
    assert_eq!(feature.stage_passes[0].pass_name, ATLAS_BUILD_PASS);
    assert_eq!(
        feature.stage_passes[0].executor_id.as_str(),
        LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID,
        "the product frame must execute the runtime cookie-atlas builder"
    );
    let pass = &feature.stage_passes[0];
    assert_eq!(
        pass.stage,
        zircon_runtime::graphics::RenderPassStage::DepthPrepass
    );
    assert_eq!(pass.resources.len(), 1);
    assert_eq!(pass.resources[0].name, LIGHT_COOKIE_ATLAS_RESOURCE);
    assert_eq!(
        pass.resources[0].kind,
        zircon_runtime::graphics::RenderFeatureResourceKind::External
    );
    assert_eq!(
        pass.resources[0].access,
        zircon_runtime::graphics::RenderFeatureResourceAccess::Write
    );
    assert!(
        !pass.flags.has_side_effects,
        "mesh and deferred lighting passes consume the cookie atlas through the graph"
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
}
