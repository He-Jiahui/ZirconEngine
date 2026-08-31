use crate::core::framework::render::{
    CorePipelineKind, PostProcessGraphResourceNames, RenderPhase, RenderPipelineHandle,
};
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
    RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};
use crate::render_graph::QueueLane;

pub(super) fn registry_material_pass_product_pipeline() -> RenderPipelineAsset {
    RenderPipelineAsset {
        handle: RenderPipelineHandle::new(811),
        revision: 1,
        name: "plan08-project-plugin-registry-material-pass-product".to_string(),
        core_pipeline: CorePipelineKind::Core3d,
        phase_mapping: vec![
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Deferred,
            RenderPhase::PostProcess,
        ],
        renderer: RendererAsset {
            name: "plan08-project-plugin-registry-material-pass-renderer".to_string(),
            stages: vec![
                RenderPassStage::DepthPrepass,
                RenderPassStage::Shadow,
                RenderPassStage::Deferred,
                RenderPassStage::Lighting,
                RenderPassStage::PostProcess,
            ],
            features: vec![
                RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredGeometry)
                    .with_descriptor_override(registry_material_pass_product_feature())
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::Temporal)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredLighting)
                    .with_descriptor_override(registry_material_pass_deferred_lighting_feature())
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess)
                    .without_quality_gate(),
                RendererFeatureAsset::builtin(BuiltinRenderFeature::AntiAlias)
                    .without_quality_gate(),
            ],
        },
    }
}

fn registry_material_pass_product_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.project_plugin_registry_material_pass_product",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plan08-project-plugin-registry-depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Shadow,
                "plan08-project-plugin-registry-shadow-atlas",
                QueueLane::Graphics,
            )
            .with_executor_id("shadow.atlas")
            .with_side_effects()
            .write_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Deferred,
                "plan08-project-plugin-registry-gbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.gbuffer")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE),
        ],
    )
}

fn registry_material_pass_deferred_lighting_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plan08.project_plugin_registry_material_pass_deferred_lighting",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
            "lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "plan08-project-plugin-registry-deferred-lighting",
                QueueLane::Graphics,
            )
            .with_executor_id("lighting.deferred")
            .read_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .read_external_texture(PostProcessGraphResourceNames::FINAL_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}
