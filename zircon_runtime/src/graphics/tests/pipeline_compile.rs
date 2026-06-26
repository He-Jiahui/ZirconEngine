use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, PreviewEnvironmentExtract, ProjectionMode, RenderCameraTarget,
    RenderDynamicResolutionSettings, RenderFrameExtract, RenderPhase,
    RenderPostProcessEffectStackSettings, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderScreenSpaceReflectionSettings, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphComputeWorkload, RenderGraphResourceAccessKind, RenderGraphResourceDesc,
    RenderGraphResourceKind,
};
use crate::rhi::TextureFormat;

use crate::graphics::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};

mod compile_options;
mod default_pipelines;
mod dynamic_resolution;
mod feature_descriptors;
mod plugin_features;
mod temporal_and_ops;
mod validation_core;
mod validation_descriptors;

fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

fn rendering_ssao_descriptor() -> RenderFeatureDescriptor {
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
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("ao.ssao-evaluate")
        .with_compute_workload(RenderGraphComputeWorkload::viewport(
            "zircon-ssao-pipeline",
            [8, 8, 1],
        ))
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
        .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)],
    )
}

fn rendering_reflection_probes_descriptor() -> RenderFeatureDescriptor {
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
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_baked_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "baked_lighting",
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.baked-composite")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_post_process_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-tile-max",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-tile-max")
            .read_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-tile-max-coarse",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-tile-max-coarse")
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "motion-vector-neighbor-max",
                QueueLane::Graphics,
            )
            .with_executor_id("post.motion-vector-neighbor-max")
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "depth-of-field-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("post.depth-of-field-prepare")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_texture_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-reflection-pyramid",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-reflection-pyramid")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-reflection-pyramid-coarse",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-reflection-pyramid-coarse")
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-specular-occlusion",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-specular-occlusion")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "screen-space-reflection-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("post.screen-space-reflection-resolve")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
            .read_texture(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
            )
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION)
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "uber",
                QueueLane::Graphics,
            )
            .with_executor_id("post.uber")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .read_texture(PostProcessGraphResourceNames::BLOOM)
            .read_texture(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)
            .read_texture(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)
            .read_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_LIST)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TONEMAPPED,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_texture(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "output-transfer",
                QueueLane::Graphics,
            )
            .with_executor_id("post.output-transfer")
            .read_texture(PostProcessGraphResourceNames::TONEMAPPED)
            .write_external_texture_with_ops(
                PostProcessGraphResourceNames::FINAL_COLOR,
                RenderGraphAttachmentOps::clear_store(),
            ),
        ],
    )
}

fn test_extract() -> RenderFrameExtract {
    extract_with_camera(ViewportCameraSnapshot::default())
}

fn extract_with_camera(camera: ViewportCameraSnapshot) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn pass_resource_access<'a>(
    compiled: &'a crate::graphics::CompiledRenderPipeline,
    pass_name: &str,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> &'a crate::render_graph::RenderGraphPassResourceAccess {
    compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == pass_name)
        .and_then(|pass| {
            pass.resources
                .iter()
                .find(|resource| resource.name == resource_name && resource.access == access)
        })
        .unwrap_or_else(|| panic!("pass `{pass_name}` should {access:?} `{resource_name}`"))
}

fn graph_resource_lifetime<'a>(
    compiled: &'a crate::graphics::CompiledRenderPipeline,
    resource_name: &str,
) -> &'a crate::render_graph::RenderGraphResourceLifetime {
    compiled
        .graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == resource_name)
        .unwrap_or_else(|| panic!("compiled graph should contain resource `{resource_name}`"))
}

fn orthographic_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Orthographic,
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
