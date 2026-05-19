use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasFallbackReason, AntiAliasMode, AntiAliasSettings, ProjectionMode,
    RenderCapabilitySummary, RenderFrameExtract, RenderFramework, RenderPipelineHandle,
    RenderQualityProfile, RenderViewportDescriptor, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::scene::anti_alias::fxaa::{FXAA_EXECUTOR_ID, FXAA_PASS_NAME};
use crate::graphics::WgpuRenderFramework;
use crate::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderPassStage, RenderPipelineAsset,
};

#[test]
fn anti_alias_settings_report_structured_fallbacks() {
    let fxaa_capable = RenderCapabilitySummary {
        backend_name: "aa-test".to_string(),
        supports_fxaa: true,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    };
    let no_screen_space_aa = RenderCapabilitySummary {
        backend_name: "aa-test".to_string(),
        supports_fxaa: false,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    };

    let auto = AntiAliasSettings::auto().resolve(&fxaa_capable, false);
    assert_eq!(auto.requested_mode, AntiAliasMode::Auto);
    assert_eq!(auto.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(
        auto.reason,
        Some(AntiAliasFallbackReason::AutoResolvedToFxaa)
    );

    let dlss = AntiAliasSettings::dlss().resolve(&fxaa_capable, false);
    assert_eq!(dlss.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(dlss.reason, Some(AntiAliasFallbackReason::UnsupportedDlss));

    let msaa = AntiAliasSettings::msaa(8).resolve(&fxaa_capable, false);
    assert_eq!(msaa.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(
        msaa.reason,
        Some(AntiAliasFallbackReason::UnsupportedMsaaSampleCount)
    );

    let taa = AntiAliasSettings::taa().resolve(&fxaa_capable, false);
    assert_eq!(taa.effective_mode, AntiAliasMode::Fxaa);
    assert_eq!(taa.reason, Some(AntiAliasFallbackReason::MissingHistory));

    let unsupported_auto = AntiAliasSettings::auto().resolve(&no_screen_space_aa, false);
    assert_eq!(unsupported_auto.effective_mode, AntiAliasMode::Off);
    assert_eq!(
        unsupported_auto.reason,
        Some(AntiAliasFallbackReason::UnsupportedFxaa)
    );
}

#[test]
fn render_product_anti_alias_compiles_fxaa_pass_for_default_3d() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&perspective_extract())
        .unwrap();

    assert!(compiled
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::AntiAlias)));
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias));

    let post = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == "post-process")
        .expect("post-process pass should compile");
    let fxaa = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == FXAA_PASS_NAME)
        .expect("FXAA pass should compile");
    let runtime_ui = compiled
        .pass_stages
        .iter()
        .position(|entry| entry.pass_name == "runtime-ui")
        .expect("runtime UI should remain after postprocess");

    assert!(post < fxaa && fxaa < runtime_ui);
    assert_eq!(
        compiled.pass_stages[fxaa].stage,
        RenderPassStage::PostProcess
    );
    let fxaa_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == FXAA_PASS_NAME)
        .expect("compiled graph should contain FXAA pass");
    assert_eq!(fxaa_pass.executor_id.as_deref(), Some(FXAA_EXECUTOR_ID));
}

#[test]
fn render_product_anti_alias_submit_records_fxaa_stats_and_graph_node() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-aa-product")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, perspective_extract())
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        stats.last_anti_alias_fallback.requested_mode,
        AntiAliasMode::Auto
    );
    assert_eq!(
        stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Fxaa
    );
    assert_eq!(
        stats.last_anti_alias_fallback.reason,
        Some(AntiAliasFallbackReason::AutoResolvedToFxaa)
    );
    assert_eq!(stats.last_anti_alias_graph_executed_pass_count, 1);
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&FXAA_EXECUTOR_ID.to_string()));
    assert!(stats
        .last_post_process_graph_executed_nodes
        .contains(&FXAA_PASS_NAME.to_string()));
}

fn perspective_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(800),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Perspective,
        ),
    )
}
