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
    let plugin_feature_enabled = |name: &str| {
        pipeline
            .enabled_features
            .iter()
            .any(|enabled| enabled.builtin_feature().is_none() && enabled.feature_name() == name)
    };
    let render_feature_enabled = |feature, plugin_name: &str| {
        feature_enabled(feature) || plugin_feature_enabled(plugin_name)
    };

    SceneRuntimeFeatureFlags {
        deferred_lighting_enabled: feature_enabled(BuiltinRenderFeature::DeferredLighting)
            && feature_enabled(BuiltinRenderFeature::DeferredGeometry),
        ssao_enabled: render_feature_enabled(
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
            "screen_space_ambient_occlusion",
        ),
        clustered_lighting_enabled: feature_enabled(BuiltinRenderFeature::ClusteredLighting),
        hybrid_global_illumination_enabled: capability_enabled(
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ),
        history_resolve_enabled: feature_enabled(BuiltinRenderFeature::HistoryResolve),
        bloom_enabled: feature_enabled(BuiltinRenderFeature::Bloom),
        color_grading_enabled: feature_enabled(BuiltinRenderFeature::ColorGrading),
        reflection_probes_enabled: render_feature_enabled(
            BuiltinRenderFeature::ReflectionProbes,
            "reflection_probes",
        ),
        baked_lighting_enabled: render_feature_enabled(
            BuiltinRenderFeature::BakedLighting,
            "baked_lighting",
        ),
        particle_rendering_enabled: render_feature_enabled(
            BuiltinRenderFeature::Particle,
            "particle",
        ),
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

    #[test]
    fn particle_rendering_runtime_flag_requires_particle_plugin_feature() {
        let default_compiled = RenderPipelineAsset::default_forward_plus()
            .compile(&test_extract())
            .unwrap();
        let default_flags = runtime_features_from_pipeline(&default_compiled);
        assert!(
            !default_flags.particle_rendering_enabled,
            "default runtime pipeline should not enable pluginized particle rendering"
        );

        let plugin_compiled = RenderPipelineAsset::default_forward_plus()
            .with_plugin_render_features([RenderFeatureDescriptor::new(
                "particle",
                vec!["particles".to_string()],
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Transparent,
                    "particle-render",
                    QueueLane::Graphics,
                )
                .with_executor_id("particle.transparent")
                .read_texture("scene-depth")
                .read_texture("scene-color")
                .write_texture("scene-color")],
            )])
            .compile(&test_extract())
            .unwrap();
        let plugin_flags = runtime_features_from_pipeline(&plugin_compiled);

        assert!(
            plugin_flags.particle_rendering_enabled,
            "particle rendering should be enabled only when the particle plugin contributes its render feature"
        );
    }

    #[test]
    fn pluginized_rendering_feature_names_drive_runtime_post_process_flags() {
        let plugin_compiled = RenderPipelineAsset::default_forward_plus()
            .with_plugin_render_features([
                RenderFeatureDescriptor::new(
                    "screen_space_ambient_occlusion",
                    vec![
                        "view".to_string(),
                        "geometry".to_string(),
                        "visibility".to_string(),
                    ],
                    vec![FrameHistoryBinding::read_write(
                        FrameHistorySlot::AmbientOcclusion,
                    )],
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::AmbientOcclusion,
                        "plugin-ssao-runtime-flag",
                        QueueLane::AsyncCompute,
                    )
                    .with_executor_id("plugin.ssao.runtime-flag")
                    .read_texture("scene-depth")
                    .write_texture("ambient-occlusion")],
                ),
                RenderFeatureDescriptor::new(
                    "reflection_probes",
                    vec![
                        "view".to_string(),
                        "lighting".to_string(),
                        "post_process".to_string(),
                    ],
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "plugin-reflection-probes-runtime-flag",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.reflection-probes.runtime-flag")
                    .read_texture("scene-color")
                    .write_texture("scene-color")],
                ),
                RenderFeatureDescriptor::new(
                    "baked_lighting",
                    vec!["lighting".to_string(), "post_process".to_string()],
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "plugin-baked-lighting-runtime-flag",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.baked-lighting.runtime-flag")
                    .read_texture("scene-color")
                    .write_texture("scene-color")],
                ),
            ])
            .compile(&test_extract())
            .unwrap();
        let flags = runtime_features_from_pipeline(&plugin_compiled);

        assert!(
            flags.ssao_enabled,
            "SSAO should follow the rendering plugin feature name"
        );
        assert!(
            flags.reflection_probes_enabled,
            "reflection probes should follow the rendering plugin feature name"
        );
        assert!(
            flags.baked_lighting_enabled,
            "baked lighting should follow the rendering plugin feature name"
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }
}
