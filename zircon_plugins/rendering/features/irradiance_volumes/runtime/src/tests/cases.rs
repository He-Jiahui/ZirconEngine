//! 测试确保外部绑定由材质读取而存活，不借副作用阻止无消费者时裁剪。
use super::*;

#[test]
fn irradiance_volume_feature_is_optional_and_registers_binding_executor() {
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
    assert_eq!(feature.stage_passes[0].pass_name, VOLUME_BIND_PASS);
    assert_eq!(
        feature.stage_passes[0].executor_id.as_str(),
        zircon_runtime::graphics::IRRADIANCE_VOLUME_BIND_EXECUTOR_ID,
        "the product frame must execute the runtime irradiance-volume binder"
    );
    let pass = &feature.stage_passes[0];
    assert_eq!(
        pass.stage,
        zircon_runtime::graphics::RenderPassStage::DepthPrepass
    );
    assert_eq!(pass.resources.len(), 1);
    assert_eq!(
        pass.resources[0].name,
        zircon_runtime::graphics::IRRADIANCE_VOLUME_RESOURCE
    );
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
        "opaque and deferred material passes consume the bound irradiance volume resource"
    );
    assert_eq!(report.extensions.render_pass_executors().len(), 1);
}
