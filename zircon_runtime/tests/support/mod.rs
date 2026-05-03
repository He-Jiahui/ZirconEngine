use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutionContext, RenderPassExecutorRegistration, RenderPassStage,
    VirtualGeometryRuntimeExtractOutput, VirtualGeometryRuntimeFeedback,
    VirtualGeometryRuntimePrepareInput, VirtualGeometryRuntimePrepareOutput,
    VirtualGeometryRuntimeProvider, VirtualGeometryRuntimeProviderRegistration,
    VirtualGeometryRuntimeState, VirtualGeometryRuntimeStats, VirtualGeometryRuntimeUpdate,
    WgpuRenderFramework,
};
use zircon_runtime::render_graph::QueueLane;

pub fn virtual_geometry_wgpu_render_framework(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
        virtual_geometry_render_pass_executor_registrations(),
        [virtual_geometry_runtime_provider_registration()],
    )
    .expect("pluginized virtual geometry framework should initialize")
}

fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
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

fn virtual_geometry_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new("virtual-geometry.prepare", test_render_pass_executor),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.node-cluster-cull",
            test_render_pass_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.page-feedback",
            test_render_pass_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.visbuffer",
            test_render_pass_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.debug-overlay",
            test_render_pass_executor,
        ),
    ]
}

fn test_render_pass_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
}

fn virtual_geometry_runtime_provider_registration() -> VirtualGeometryRuntimeProviderRegistration {
    VirtualGeometryRuntimeProviderRegistration::new(
        "virtual_geometry",
        Arc::new(TestVirtualGeometryRuntimeProvider),
    )
}

#[derive(Debug, Default)]
struct TestVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for TestVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn VirtualGeometryRuntimeState> {
        Box::<TestVirtualGeometryRuntimeState>::default()
    }

    fn build_extract_from_meshes(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug: Option<RenderVirtualGeometryDebugState>,
        load_model: &mut dyn FnMut(ResourceId) -> Option<zircon_runtime::asset::ModelAsset>,
    ) -> Option<VirtualGeometryRuntimeExtractOutput> {
        let mesh = meshes.first()?;
        let model = load_model(mesh.model.id())?;
        if !model
            .primitives
            .iter()
            .any(|primitive| primitive.virtual_geometry.is_some())
        {
            return None;
        }
        let debug = debug.unwrap_or_default();
        let lod_level = debug.forced_mip.unwrap_or(0);
        Some(VirtualGeometryRuntimeExtractOutput::new(
            RenderVirtualGeometryExtract {
                cluster_budget: 1,
                page_budget: 1,
                clusters: vec![RenderVirtualGeometryCluster {
                    entity: mesh.node_id,
                    cluster_id: 1,
                    hierarchy_node_id: Some(0),
                    page_id: 1,
                    lod_level,
                    parent_cluster_id: None,
                    bounds_center: mesh.transform.translation,
                    bounds_radius: 100.0,
                    screen_space_error: 1.0,
                }],
                hierarchy_nodes: Vec::new(),
                hierarchy_child_ids: Vec::new(),
                pages: vec![RenderVirtualGeometryPage {
                    page_id: 1,
                    resident: true,
                    size_bytes: 4096,
                }],
                page_dependencies: vec![RenderVirtualGeometryPageDependency {
                    page_id: 1,
                    parent_page_id: None,
                    child_page_ids: Vec::new(),
                }],
                instances: vec![RenderVirtualGeometryInstance {
                    entity: mesh.node_id,
                    source_model: Some(mesh.model.id()),
                    transform: mesh.transform,
                    cluster_offset: 0,
                    cluster_count: 1,
                    page_offset: 0,
                    page_count: 1,
                    mesh_name: Some("TestProviderAutomaticMesh".to_string()),
                    source_hint: Some("runtime-test-provider".to_string()),
                }],
                debug,
            },
            Vec::new(),
            Vec::new(),
        ))
    }
}

#[derive(Debug, Default)]
struct TestVirtualGeometryRuntimeState {
    page_table_entry_count: usize,
    resident_page_count: usize,
    pending_request_count: usize,
}

impl VirtualGeometryRuntimeState for TestVirtualGeometryRuntimeState {
    fn prepare_frame(
        &mut self,
        input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        if let Some(plan) = input.page_upload_plan() {
            self.page_table_entry_count = plan.resident_pages.len();
            self.resident_page_count = plan.resident_pages.len();
            self.pending_request_count = plan.requested_pages.len();
            return VirtualGeometryRuntimePrepareOutput::new(plan.evictable_pages.clone());
        }

        *self = Self::default();
        VirtualGeometryRuntimePrepareOutput::default()
    }

    fn update_after_render(
        &mut self,
        feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        if let Some(feedback) = feedback.visibility_feedback() {
            self.pending_request_count = feedback.requested_pages.len();
        }

        VirtualGeometryRuntimeUpdate::new(VirtualGeometryRuntimeStats::new(
            self.page_table_entry_count,
            self.resident_page_count,
            self.pending_request_count,
            0,
            0,
            0,
        ))
    }
}
