use crate::{BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement};

use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn runtime_features_from_pipeline(
    pipeline: &CompiledRenderPipeline,
) -> SceneRuntimeFeatureFlags {
    let feature_enabled = |feature| {
        pipeline
            .enabled_features
            .iter()
            .any(|enabled| enabled.is_builtin(feature))
    };
    let capability_enabled = |requirement| {
        pipeline
            .enabled_features
            .iter()
            .any(|enabled| enabled.requires_capability(requirement))
    };

    SceneRuntimeFeatureFlags {
        deferred_lighting_enabled: feature_enabled(BuiltinRenderFeature::DeferredLighting)
            && feature_enabled(BuiltinRenderFeature::DeferredGeometry),
        ssao_enabled: feature_enabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion),
        clustered_lighting_enabled: feature_enabled(BuiltinRenderFeature::ClusteredLighting),
        hybrid_global_illumination_enabled: capability_enabled(
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ),
        history_resolve_enabled: feature_enabled(BuiltinRenderFeature::HistoryResolve),
        bloom_enabled: feature_enabled(BuiltinRenderFeature::Bloom),
        color_grading_enabled: feature_enabled(BuiltinRenderFeature::ColorGrading),
        reflection_probes_enabled: feature_enabled(BuiltinRenderFeature::ReflectionProbes),
        baked_lighting_enabled: feature_enabled(BuiltinRenderFeature::BakedLighting),
        particle_rendering_enabled: feature_enabled(BuiltinRenderFeature::Particle),
        virtual_geometry_enabled: capability_enabled(
            RenderFeatureCapabilityRequirement::VirtualGeometry,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
    use crate::render_graph::QueueLane;
    use crate::scene::world::World;
    use crate::{
        FrameHistoryBinding, FrameHistorySlot, RenderFeatureCapabilityRequirement,
        RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset,
        RenderPipelineCompileOptions, RendererFeatureAsset,
    };

    use super::runtime_features_from_pipeline;

    #[test]
    fn plugin_render_feature_capabilities_drive_advanced_runtime_flags() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.virtual_geometry",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::DepthPrepass,
                        "plugin-virtual-geometry-runtime-flag",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.virtual-geometry.runtime-flag")
                    .with_side_effects()],
                )
                .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
            ));
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.hybrid_gi",
                    Vec::new(),
                    vec![FrameHistoryBinding::read_write(
                        FrameHistorySlot::GlobalIllumination,
                    )],
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::Lighting,
                        "plugin-hybrid-gi-runtime-flag",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.hybrid-gi.runtime-flag")
                    .with_side_effects()],
                )
                .with_capability_requirement(
                    RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                ),
            ));

        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                    .with_capability_enabled(
                        RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                    ),
            )
            .unwrap();
        let flags = runtime_features_from_pipeline(&compiled);

        assert!(
            flags.virtual_geometry_enabled,
            "virtual geometry should follow plugin capability metadata"
        );
        assert!(
            flags.hybrid_global_illumination_enabled,
            "hybrid GI should follow plugin capability metadata"
        );
    }

    #[test]
    fn builtin_feature_identity_without_capability_metadata_does_not_drive_advanced_runtime_flags()
    {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
                "legacy-virtual-geometry-without-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "legacy-virtual-geometry-without-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("legacy.virtual-geometry.without-capability")
                .with_side_effects()],
            )));
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
                "legacy-hybrid-gi-without-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "legacy-hybrid-gi-without-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("legacy.hybrid-gi.without-capability")
                .with_side_effects()],
            )));

        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                    .with_capability_enabled(
                        RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                    ),
            )
            .unwrap();
        let flags = runtime_features_from_pipeline(&compiled);

        assert!(
            !flags.virtual_geometry_enabled,
            "virtual geometry runtime state should require capability metadata"
        );
        assert!(
            !flags.hybrid_global_illumination_enabled,
            "hybrid GI runtime state should require capability metadata"
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }
}
