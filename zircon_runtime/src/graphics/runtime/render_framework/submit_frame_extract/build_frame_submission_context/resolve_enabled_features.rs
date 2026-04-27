use crate::{CompiledRenderPipeline, RenderFeatureCapabilityRequirement};

pub(super) fn resolve_enabled_features(compiled_pipeline: &CompiledRenderPipeline) -> (bool, bool) {
    let hybrid_gi_enabled = compiled_pipeline.enabled_features.iter().any(|feature| {
        feature.requires_capability(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
    });
    let virtual_geometry_enabled = compiled_pipeline.enabled_features.iter().any(|feature| {
        feature.requires_capability(RenderFeatureCapabilityRequirement::VirtualGeometry)
    });

    (hybrid_gi_enabled, virtual_geometry_enabled)
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
    use crate::render_graph::QueueLane;
    use crate::scene::world::World;
    use crate::{
        RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
        RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
    };

    use super::resolve_enabled_features;

    #[test]
    fn advanced_runtime_submission_flags_follow_capability_metadata_only() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
                "legacy-virtual-geometry-without-submission-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::DepthPrepass,
                    "legacy-virtual-geometry-without-submission-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("legacy.virtual-geometry.without-submission-capability")
                .with_side_effects()],
            )));
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
                "legacy-hybrid-gi-without-submission-capability",
                Vec::new(),
                Vec::new(),
                vec![RenderFeaturePassDescriptor::new(
                    RenderPassStage::Lighting,
                    "legacy-hybrid-gi-without-submission-capability",
                    QueueLane::Graphics,
                )
                .with_executor_id("legacy.hybrid-gi.without-submission-capability")
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

        assert_eq!(resolve_enabled_features(&compiled), (false, false));
    }

    #[test]
    fn plugin_capability_metadata_enables_advanced_runtime_submission_flags() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.virtual_geometry.submission",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::DepthPrepass,
                        "plugin-virtual-geometry-submission",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.virtual-geometry.submission")
                    .with_side_effects()],
                )
                .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry),
            ));
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(
                RenderFeatureDescriptor::new(
                    "plugin.hybrid_gi.submission",
                    Vec::new(),
                    Vec::new(),
                    vec![RenderFeaturePassDescriptor::new(
                        RenderPassStage::Lighting,
                        "plugin-hybrid-gi-submission",
                        QueueLane::Graphics,
                    )
                    .with_executor_id("plugin.hybrid-gi.submission")
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

        assert_eq!(resolve_enabled_features(&compiled), (true, true));
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }
}
