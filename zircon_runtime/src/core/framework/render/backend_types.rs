use crate::core::math::UVec2;

use super::{
    RenderFrameExtract, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderViewportHandle(u64);

impl RenderViewportHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineHandle(u64);

impl RenderPipelineHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FrameHistoryHandle(u64);

impl FrameHistoryHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderingBackendInfo {
    pub backend_name: String,
    pub supports_runtime_preview: bool,
    pub supports_shared_texture_viewports: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderQueueCapability {
    Graphics,
    Compute,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCapabilityKind {
    VirtualGeometry,
    HybridGlobalIllumination,
    AccelerationStructures,
    InlineRayQuery,
    RayTracingPipeline,
    AsyncCompute,
    AsyncCopy,
}

impl RenderCapabilityKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
            Self::AccelerationStructures => "acceleration_structures",
            Self::InlineRayQuery => "inline_ray_query",
            Self::RayTracingPipeline => "ray_tracing_pipeline",
            Self::AsyncCompute => "async_compute",
            Self::AsyncCopy => "async_copy",
        }
    }

    pub fn is_satisfied_by(self, capabilities: &RenderCapabilitySummary) -> bool {
        match self {
            Self::VirtualGeometry => capabilities.virtual_geometry_supported,
            Self::HybridGlobalIllumination => capabilities.hybrid_global_illumination_supported,
            Self::AccelerationStructures => capabilities.acceleration_structures_supported,
            Self::InlineRayQuery => capabilities.inline_ray_query,
            Self::RayTracingPipeline => capabilities.ray_tracing_pipeline,
            Self::AsyncCompute => capabilities.supports_async_compute,
            Self::AsyncCopy => capabilities.supports_async_copy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCapabilityMismatchDetail {
    pub capability: RenderCapabilityKind,
}

impl RenderCapabilityMismatchDetail {
    // Keep mismatch payloads backend-neutral so framework consumers never need graphics enums.
    pub const fn new(capability: RenderCapabilityKind) -> Self {
        Self { capability }
    }

    pub const fn label(self) -> &'static str {
        self.capability.label()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderCapabilitySummary {
    pub backend_name: String,
    pub queue_classes: Vec<RenderQueueCapability>,
    pub supports_surface: bool,
    pub supports_offscreen: bool,
    pub supports_async_compute: bool,
    pub supports_async_copy: bool,
    pub supports_pipeline_cache: bool,
    pub acceleration_structures_supported: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
    pub virtual_geometry_supported: bool,
    pub hybrid_global_illumination_supported: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    SubmitFrameExtract {
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    },
    ReloadPipeline {
        pipeline: RenderPipelineHandle,
    },
    SetQualityProfile {
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderQuery {
    Stats,
    CaptureFrame(RenderViewportHandle),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderViewportDescriptor {
    pub size: UVec2,
    pub label: Option<String>,
}

impl RenderViewportDescriptor {
    pub fn new(size: UVec2) -> Self {
        Self { size, label: None }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureQualitySettings {
    pub clustered_lighting: bool,
    pub screen_space_ambient_occlusion: bool,
    pub history_resolve: bool,
    pub bloom: bool,
    pub color_grading: bool,
    pub reflection_probes: bool,
    pub baked_lighting: bool,
    pub particle_rendering: bool,
    pub virtual_geometry: bool,
    pub hybrid_global_illumination: bool,
    pub allow_async_compute: bool,
}

impl Default for RenderFeatureQualitySettings {
    fn default() -> Self {
        Self {
            clustered_lighting: true,
            screen_space_ambient_occlusion: true,
            history_resolve: true,
            bloom: true,
            color_grading: true,
            reflection_probes: true,
            baked_lighting: true,
            particle_rendering: true,
            virtual_geometry: false,
            hybrid_global_illumination: false,
            allow_async_compute: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderQualityProfile {
    pub name: String,
    pub pipeline_override: Option<RenderPipelineHandle>,
    pub features: RenderFeatureQualitySettings,
}

impl RenderQualityProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pipeline_override: None,
            features: RenderFeatureQualitySettings::default(),
        }
    }

    pub fn with_pipeline_asset(mut self, pipeline: RenderPipelineHandle) -> Self {
        self.pipeline_override = Some(pipeline);
        self
    }

    pub fn with_clustered_lighting(mut self, enabled: bool) -> Self {
        self.features.clustered_lighting = enabled;
        self
    }

    pub fn with_screen_space_ambient_occlusion(mut self, enabled: bool) -> Self {
        self.features.screen_space_ambient_occlusion = enabled;
        self
    }

    pub fn with_history_resolve(mut self, enabled: bool) -> Self {
        self.features.history_resolve = enabled;
        self
    }

    pub fn with_bloom(mut self, enabled: bool) -> Self {
        self.features.bloom = enabled;
        self
    }

    pub fn with_color_grading(mut self, enabled: bool) -> Self {
        self.features.color_grading = enabled;
        self
    }

    pub fn with_reflection_probes(mut self, enabled: bool) -> Self {
        self.features.reflection_probes = enabled;
        self
    }

    pub fn with_baked_lighting(mut self, enabled: bool) -> Self {
        self.features.baked_lighting = enabled;
        self
    }

    pub fn with_particle_rendering(mut self, enabled: bool) -> Self {
        self.features.particle_rendering = enabled;
        self
    }

    pub fn with_virtual_geometry(mut self, enabled: bool) -> Self {
        self.features.virtual_geometry = enabled;
        self
    }

    pub fn with_hybrid_global_illumination(mut self, enabled: bool) -> Self {
        self.features.hybrid_global_illumination = enabled;
        self
    }

    pub fn with_async_compute(mut self, enabled: bool) -> Self {
        self.features.allow_async_compute = enabled;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub active_viewports: usize,
    pub submitted_frames: u64,
    pub captured_frames: u64,
    pub last_generation: Option<u64>,
    pub last_pipeline: Option<RenderPipelineHandle>,
    pub last_frame_history: Option<FrameHistoryHandle>,
    pub last_quality_profile: Option<String>,
    pub last_effective_features: Vec<String>,
    pub last_graph_pass_count: usize,
    pub last_graph_culled_pass_count: usize,
    pub last_graph_queue_fallback_pass_count: usize,
    pub last_graph_resource_lifetime_count: usize,
    pub last_graph_planned_resource_access_count: usize,
    pub last_graph_planned_dependency_count: usize,
    pub last_graph_transient_texture_slot_count: usize,
    pub last_graph_transient_buffer_slot_count: usize,
    pub last_graph_executed_pass_count: usize,
    pub last_graph_executed_passes: Vec<String>,
    pub last_graph_executed_executor_ids: Vec<String>,
    pub last_graph_executed_resource_access_count: usize,
    pub last_graph_executed_dependency_count: usize,
    pub last_virtual_geometry_graph_executed_pass_count: usize,
    pub last_hybrid_gi_graph_executed_pass_count: usize,
    pub last_async_compute_pass_count: usize,
    pub last_ui_command_count: usize,
    pub last_ui_quad_count: usize,
    pub last_ui_text_payload_count: usize,
    pub last_ui_image_payload_count: usize,
    pub last_ui_clipped_command_count: usize,
    pub last_virtual_geometry_cluster_budget: usize,
    pub last_virtual_geometry_page_budget: usize,
    pub last_virtual_geometry_input_cluster_count: usize,
    pub last_virtual_geometry_input_page_count: usize,
    pub last_virtual_geometry_visible_cluster_count: usize,
    pub last_virtual_geometry_visible_entity_count: usize,
    pub last_virtual_geometry_instance_count: usize,
    pub last_virtual_geometry_requested_page_count: usize,
    pub last_virtual_geometry_dirty_page_count: usize,
    pub last_virtual_geometry_forced_mip: Option<u8>,
    pub last_virtual_geometry_freeze_cull: bool,
    pub last_virtual_geometry_visualize_bvh: bool,
    pub last_virtual_geometry_visualize_visbuffer: bool,
    pub last_virtual_geometry_print_leaf_clusters: bool,
    pub last_virtual_geometry_page_table_entry_count: usize,
    pub last_virtual_geometry_resident_page_count: usize,
    pub last_virtual_geometry_pending_request_count: usize,
    pub last_virtual_geometry_page_dependency_count: usize,
    pub last_virtual_geometry_completed_page_count: usize,
    pub last_virtual_geometry_replaced_page_count: usize,
    pub last_virtual_geometry_indirect_draw_count: usize,
    pub last_virtual_geometry_indirect_buffer_count: usize,
    pub last_virtual_geometry_indirect_args_count: usize,
    pub last_virtual_geometry_indirect_segment_count: usize,
    pub last_virtual_geometry_execution_segment_count: usize,
    pub last_virtual_geometry_execution_page_count: usize,
    pub last_virtual_geometry_execution_resident_segment_count: usize,
    pub last_virtual_geometry_execution_pending_segment_count: usize,
    pub last_virtual_geometry_execution_missing_segment_count: usize,
    pub last_virtual_geometry_execution_repeated_draw_count: usize,
    pub last_virtual_geometry_cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub last_virtual_geometry_node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub last_virtual_geometry_node_and_cluster_cull_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_dispatch_group_count: [usize; 3],
    pub last_virtual_geometry_node_and_cluster_cull_instance_seed_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_instance_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_child_work_item_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_traversal_record_count: usize,
    pub last_virtual_geometry_node_and_cluster_cull_page_request_count: usize,
    pub last_virtual_geometry_selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    pub last_virtual_geometry_selected_cluster_count: usize,
    pub last_virtual_geometry_visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub last_virtual_geometry_visbuffer64_entry_count: usize,
    pub last_virtual_geometry_hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub last_virtual_geometry_hardware_rasterization_record_count: usize,
    pub last_hybrid_gi_active_probe_count: usize,
    pub last_hybrid_gi_requested_probe_count: usize,
    pub last_hybrid_gi_dirty_probe_count: usize,
    pub last_hybrid_gi_cache_entry_count: usize,
    pub last_hybrid_gi_resident_probe_count: usize,
    pub last_hybrid_gi_pending_update_count: usize,
    pub last_hybrid_gi_scheduled_trace_region_count: usize,
    pub last_hybrid_gi_scene_card_count: usize,
    pub last_hybrid_gi_scene_screen_probe_count: usize,
    pub last_hybrid_gi_scene_radiance_cache_entry_count: usize,
    pub last_hybrid_gi_surface_cache_resident_page_count: usize,
    pub last_hybrid_gi_surface_cache_dirty_page_count: usize,
    pub last_hybrid_gi_surface_cache_feedback_card_count: usize,
    pub last_hybrid_gi_surface_cache_capture_slot_count: usize,
    pub last_hybrid_gi_surface_cache_invalidated_page_count: usize,
    pub last_hybrid_gi_voxel_resident_clipmap_count: usize,
    pub last_hybrid_gi_voxel_dirty_clipmap_count: usize,
    pub last_hybrid_gi_voxel_invalidated_clipmap_count: usize,
    pub capabilities: RenderCapabilitySummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
}

impl CapturedFrame {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>, generation: u64) -> Self {
        Self {
            width,
            height,
            rgba,
            generation,
        }
    }
}
