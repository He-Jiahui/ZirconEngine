use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::SolariRuntimeStatus;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::graphics::{
    FrameHistoryBinding, FrameHistorySlot, HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput,
    HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider, HybridGiRuntimeProviderRegistration,
    HybridGiRuntimeState, HybridGiRuntimeUpdate, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage, SolariRuntimeProvider,
    SolariRuntimeProviderRegistration, VirtualGeometryRuntimeFeedback,
    VirtualGeometryRuntimePrepareInput, VirtualGeometryRuntimePrepareOutput,
    VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeProviderRegistration,
    VirtualGeometryRuntimeState, VirtualGeometryRuntimeUpdate,
};
use crate::render_graph::QueueLane;

pub(super) fn pluginized_wgpu_render_framework() -> WgpuRenderFramework {
    pluginized_wgpu_render_framework_with_asset_manager(Arc::new(ProjectAssetManager::default()))
}

pub(super) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
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
    WgpuRenderFramework::new_with_plugin_render_extensions(
        asset_manager,
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        Vec::new(),
        Vec::new(),
        [test_hybrid_gi_runtime_provider()],
        [test_virtual_geometry_runtime_provider()],
    )
    .unwrap()
}

pub(super) fn pluginized_wgpu_render_framework_with_solari_provider(
    status: SolariRuntimeStatus,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_extensions_and_solari(
        Arc::new(ProjectAssetManager::default()),
        [
            virtual_geometry_render_feature_descriptor(),
            hybrid_gi_render_feature_descriptor(),
        ],
        Vec::new(),
        Vec::new(),
        [test_hybrid_gi_runtime_provider()],
        [test_solari_runtime_provider(status)],
        [test_virtual_geometry_runtime_provider()],
    )
    .unwrap()
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
            .read_texture("scene-depth")
            .write_buffer("hybrid-gi-scene"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .read_buffer("hybrid-gi-scene")
            .write_buffer("hybrid-gi-trace"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer("hybrid-gi-trace")
            .write_texture("hybrid-gi-lighting"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "hybrid-gi-history",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.history")
            .read_texture("scene-color")
            .write_external("history-global-illumination"),
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

#[derive(Debug)]
struct TestVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for TestVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::new(TestVirtualGeometryRuntimeState)
    }
}

#[derive(Debug)]
struct TestVirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState for TestVirtualGeometryRuntimeState {
    fn prepare_frame(
        &mut self,
        _input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        VirtualGeometryRuntimePrepareOutput::default()
    }

    fn update_after_render(
        &mut self,
        _feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        VirtualGeometryRuntimeUpdate::default()
    }
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

fn test_virtual_geometry_runtime_provider() -> VirtualGeometryRuntimeProviderRegistration {
    VirtualGeometryRuntimeProviderRegistration::new(
        "test.virtual-geometry",
        Arc::new(TestVirtualGeometryRuntimeProvider),
    )
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
        .read_texture("scene-color")
        .write_texture("scene-color")],
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
        .read_texture("scene-depth")
        .write_texture("ambient-occlusion")],
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
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "post-process",
            QueueLane::Graphics,
        )
        .with_executor_id("post.stack")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}
