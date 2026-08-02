use crate::graphics::CompiledRenderPipeline;

use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn runtime_features_from_pipeline(
    pipeline: &CompiledRenderPipeline,
) -> SceneRuntimeFeatureFlags {
    let flags = pipeline.runtime_feature_flags();
    SceneRuntimeFeatureFlags {
        deferred_lighting_enabled: flags.deferred_lighting_enabled,
        ssao_enabled: flags.ssao_enabled,
        contact_shadow_enabled: flags.contact_shadow_enabled,
        clustered_lighting_enabled: flags.clustered_lighting_enabled,
        hybrid_global_illumination_enabled: flags.hybrid_global_illumination_enabled,
        temporal_history_enabled: flags.temporal_history_enabled,
        bloom_enabled: flags.bloom_enabled,
        color_grading_enabled: flags.color_grading_enabled,
        anti_alias_enabled: flags.anti_alias_enabled,
        reflection_probes_enabled: flags.reflection_probes_enabled,
        baked_lighting_enabled: flags.baked_lighting_enabled,
        sprite_rendering_enabled: flags.sprite_rendering_enabled,
        particle_rendering_enabled: flags.particle_rendering_enabled,
        virtual_geometry_enabled: flags.virtual_geometry_enabled,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        PostProcessGraphResourceNames, RenderFrameExtract, RenderWorldSnapshotHandle,
    };
    use crate::graphics::{
        BuiltinRenderFeature, FrameHistoryBinding, FrameHistorySlot,
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
    };
    use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};
    use crate::scene::world::World;

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
                "fallback-virtual-geometry-without-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "fallback-virtual-geometry-without-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("fallback.virtual-geometry.without-capability")
                .with_side_effects()],
            )));
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
                "fallback-hybrid-gi-without-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "fallback-hybrid-gi-without-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("fallback.hybrid-gi.without-capability")
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
                    RenderPassStage::Transparent3d,
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
    fn descriptor_only_builtin_tokens_do_not_enable_pluginized_runtime_flags() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        for feature in [
            BuiltinRenderFeature::ReflectionProbes,
            BuiltinRenderFeature::BakedLighting,
            BuiltinRenderFeature::Particle,
        ] {
            pipeline
                .renderer
                .features
                .push(RendererFeatureAsset::builtin(feature));
        }
        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_feature_enabled(BuiltinRenderFeature::ReflectionProbes)
                    .with_feature_enabled(BuiltinRenderFeature::BakedLighting)
                    .with_feature_enabled(BuiltinRenderFeature::Particle),
            )
            .unwrap();
        let flags = runtime_features_from_pipeline(&compiled);

        assert!(compiled
            .enabled_features()
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::ReflectionProbes)));
        assert!(compiled
            .required_extract_sections
            .contains(&"reflection_probes".to_string()));
        assert!(
            !flags.reflection_probes_enabled,
            "descriptor-only built-in reflection probes should not enable executable runtime state"
        );
        assert!(
            !flags.baked_lighting_enabled,
            "descriptor-only built-in baked lighting should not enable executable runtime state"
        );
        assert!(
            !flags.particle_rendering_enabled,
            "descriptor-only built-in particles should not enable executable runtime state"
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
                    .with_compute_workload(RenderGraphComputeWorkload::viewport(
                        "ssao-evaluate",
                        [8, 8, 1],
                    ))
                    .read_texture("scene-depth")
                    .write_storage_external_texture(
                        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                    )],
                ),
                RenderFeatureDescriptor::new(
                    "contact_shadow",
                    vec![
                        "view".to_string(),
                        "geometry".to_string(),
                        "visibility".to_string(),
                    ],
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::PostProcess,
                        "plugin-contact-shadow-runtime-flag",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.contact-shadow.runtime-flag")
                    .read_texture("scene-color")
                    .write_texture("scene-color")],
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
            flags.contact_shadow_enabled,
            "contact shadow should follow the rendering plugin feature name"
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
