use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    GeometrySourceBindingKind, GeometrySourceBindingRequirement, GeometrySourceDescriptor,
    GeometrySourceId, GeometrySourceVertexAttribute, PostProcessGraphResourceNames,
    RenderShaderDefinitionValue, SolariRuntimeStatus, GEOMETRY_SOURCE_PLUGIN_ID_START,
};
use crate::graphics::runtime::WgpuRenderFramework;
use crate::graphics::{
    FrameHistoryBinding, FrameHistorySlot, HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput,
    HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration,
    HybridGiRuntimeState, HybridGiRuntimeUpdate, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage, SolariRuntimeProvider,
    SolariRuntimeProviderRegistration,
};
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload};

mod virtual_geometry_provider;

use virtual_geometry_provider::test_virtual_geometry_runtime_provider;

const HYBRID_GI_SCENE_PACKET_MINIMUM_SIZE_BYTES: u64 = 710 * 4;
const HYBRID_GI_TRACE_PACKET_MINIMUM_SIZE_BYTES: u64 = 448 * 4;

pub(super) fn pluginized_wgpu_render_framework() -> WgpuRenderFramework {
    pluginized_wgpu_render_framework_with_asset_manager(Arc::new(ProjectAssetManager::default()))
}

pub(super) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_extensions_and_shading_models(
        asset_manager,
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        advanced_render_pass_executor_registrations(),
        Vec::new(),
        virtual_geometry_geometry_source_descriptors(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

pub(super) fn pluginized_wgpu_render_framework_with_advanced_providers() -> WgpuRenderFramework {
    pluginized_wgpu_render_framework_with_advanced_providers_and_asset_manager(Arc::new(
        ProjectAssetManager::default(),
    ))
}

pub(super) fn pluginized_wgpu_render_framework_with_advanced_providers_and_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_extensions_and_shading_models(
        asset_manager,
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        advanced_render_pass_executor_registrations(),
        Vec::new(),
        virtual_geometry_geometry_source_descriptors(),
        Vec::new(),
        [test_hybrid_gi_runtime_provider()],
        [test_virtual_geometry_runtime_provider()],
    )
    .unwrap()
}

pub(super) fn pluginized_wgpu_render_framework_with_solari_provider(
    status: SolariRuntimeStatus,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_extensions_and_solari_and_shading_models(
        Arc::new(ProjectAssetManager::default()),
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        advanced_render_pass_executor_registrations(),
        Vec::new(),
        [test_hybrid_gi_runtime_provider()],
        [test_solari_runtime_provider(status)],
        [test_virtual_geometry_runtime_provider()],
        virtual_geometry_geometry_source_descriptors(),
        Vec::new(),
    )
    .unwrap()
}

fn virtual_geometry_geometry_source_descriptors() -> Vec<GeometrySourceDescriptor> {
    vec![GeometrySourceDescriptor {
        id: GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
        token: "custom:virtual_geometry".to_string(),
        wgsl_include: "zr_geometry_virtual_geometry.wgsl".to_string(),
        vertex_attributes: vec![
            GeometrySourceVertexAttribute::Position,
            GeometrySourceVertexAttribute::Normal,
            GeometrySourceVertexAttribute::Tangent,
            GeometrySourceVertexAttribute::Uv0,
        ],
        required_bindings: vec![
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryPages,
                "virtual_geometry.pages",
            ),
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryClusters,
                "virtual_geometry.clusters",
            ),
        ],
        shader_defines: vec![RenderShaderDefinitionValue::bool(
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            true,
        )],
    }]
}

pub(super) fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "virtual_geometry",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .write_buffer("virtual-geometry-page-requests"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-node-cluster-cull",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("virtual-geometry.node-cluster-cull")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-virtual-geometry-node-cluster-cull",
                [64, 1, 1],
                [1, 1, 1],
            ))
            .read_buffer("virtual-geometry-page-requests")
            .write_buffer("virtual-geometry-visible-clusters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-page-feedback",
                QueueLane::AsyncCopy,
            )
            .with_executor_id("virtual-geometry.page-feedback")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_external("virtual-geometry-feedback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-visbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.visbuffer")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "virtual-geometry-debug-overlay",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.debug-overlay")
            .read_buffer("virtual-geometry-visible-clusters")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

pub(super) fn hybrid_gi_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-scene-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.scene-prepare")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .write_buffer_with_minimum_size(
                PostProcessGraphResourceNames::HYBRID_GI_SCENE,
                HYBRID_GI_SCENE_PACKET_MINIMUM_SIZE_BYTES,
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .with_compute_workload(RenderGraphComputeWorkload::fixed(
                "zircon-hybrid-gi-trace-schedule",
                [8, 8, 1],
                [1, 1, 1],
            ))
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .read_buffer(PostProcessGraphResourceNames::HYBRID_GI_SCENE)
            .write_buffer_with_minimum_size(
                PostProcessGraphResourceNames::HYBRID_GI_TRACE,
                HYBRID_GI_TRACE_PACKET_MINIMUM_SIZE_BYTES,
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer(PostProcessGraphResourceNames::HYBRID_GI_TRACE)
            .write_texture(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "hybrid-gi-history",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.history")
            .read_texture(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING)
            .write_external_texture("history-global-illumination"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

pub(super) fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

fn advanced_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    [
        "virtual-geometry.prepare",
        "virtual-geometry.node-cluster-cull",
        "virtual-geometry.page-feedback",
        "virtual-geometry.visbuffer",
        "virtual-geometry.debug-overlay",
        "hybrid-gi.scene-prepare",
        "hybrid-gi.trace-schedule",
        "hybrid-gi.resolve",
        "hybrid-gi.history",
    ]
    .into_iter()
    .map(|executor_id| {
        RenderPassExecutorRegistration::new(executor_id, test_advanced_render_pass_executor)
    })
    .collect()
}

fn test_advanced_render_pass_executor(
    _context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    Ok(())
}

#[derive(Debug)]
struct TestHybridGiRuntimeProvider;

impl HybridGiRuntimeProvider for TestHybridGiRuntimeProvider {
    fn create_state(&self) -> Box<dyn HybridGiRuntimeState> {
        Box::new(TestHybridGiRuntimeState)
    }
}

#[derive(Debug)]
struct TestSolariRuntimeProvider {
    status: SolariRuntimeStatus,
}

impl SolariRuntimeProvider for TestSolariRuntimeProvider {
    fn runtime_status(&self) -> SolariRuntimeStatus {
        self.status
    }

    fn runtime_status_message(&self) -> Option<&str> {
        (self.status == SolariRuntimeStatus::Unavailable)
            .then_some("test solari provider unavailable")
    }
}

struct TestHybridGiRuntimeState;

impl HybridGiRuntimeState for TestHybridGiRuntimeState {
    fn prepare_frame(
        &mut self,
        _input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        HybridGiRuntimePrepareOutput::default()
    }

    fn update_after_render(&mut self, _feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        HybridGiRuntimeUpdate::default()
    }
}

fn test_hybrid_gi_runtime_provider() -> HybridGiRuntimeProviderRegistration {
    HybridGiRuntimeProviderRegistration::new(
        "test.hybrid-gi",
        Arc::new(TestHybridGiRuntimeProvider),
    )
}

fn test_solari_runtime_provider(status: SolariRuntimeStatus) -> SolariRuntimeProviderRegistration {
    SolariRuntimeProviderRegistration::new(
        "test.solari",
        Arc::new(TestSolariRuntimeProvider { status }),
    )
}

pub(super) fn particle_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent3d,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .write_texture("scene-color")],
    )
}

pub(super) fn particle_render_feature_descriptor_with_velocity() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-velocity",
                QueueLane::Graphics,
            )
            .with_executor_id("particle.velocity")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "particle-render",
                QueueLane::Graphics,
            )
            .with_executor_id("particle.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
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
