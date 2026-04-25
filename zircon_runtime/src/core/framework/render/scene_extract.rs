use crate::core::math::{Real, Transform, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::core::framework::scene::{EntityId, Mobility};

use super::{FallbackSkyboxKind, RenderOverlayExtract, ViewportCameraSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: EntityId,
    pub transform: Transform,
    pub model: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub tint: Vec4,
    pub mobility: Mobility,
    pub render_layer_mask: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub node_id: EntityId,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderPointLightSnapshot {
    pub node_id: EntityId,
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSpotLightSnapshot {
    pub node_id: EntityId,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBloomSettings {
    pub threshold: Real,
    pub intensity: Real,
    pub radius: Real,
}

impl Default for RenderBloomSettings {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.0,
            radius: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderColorGradingSettings {
    pub exposure: Real,
    pub contrast: Real,
    pub saturation: Real,
    pub gamma: Real,
    pub tint: Vec3,
}

impl Default for RenderColorGradingSettings {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderReflectionProbeSnapshot {
    pub position: Vec3,
    pub radius: Real,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderReflectionProbeSnapshot {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.0,
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBakedLightingExtract {
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for RenderBakedLightingExtract {
    fn default() -> Self {
        Self {
            color: Vec3::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVirtualGeometryCluster {
    pub entity: EntityId,
    pub cluster_id: u32,
    pub hierarchy_node_id: Option<u32>,
    pub page_id: u32,
    pub lod_level: u8,
    pub parent_cluster_id: Option<u32>,
    pub bounds_center: Vec3,
    pub bounds_radius: Real,
    pub screen_space_error: Real,
}

impl Default for RenderVirtualGeometryCluster {
    fn default() -> Self {
        Self {
            entity: 0,
            cluster_id: 0,
            hierarchy_node_id: None,
            page_id: 0,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_space_error: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryHierarchyNode {
    pub instance_index: u32,
    pub node_id: u32,
    pub child_base: u32,
    pub child_count: u32,
    pub cluster_start: u32,
    pub cluster_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderVirtualGeometryPage {
    pub page_id: u32,
    pub resident: bool,
    pub size_bytes: u64,
}

impl Default for RenderVirtualGeometryPage {
    fn default() -> Self {
        Self {
            page_id: 0,
            resident: false,
            size_bytes: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugState {
    pub forced_mip: Option<u8>,
    pub freeze_cull: bool,
    pub visualize_bvh: bool,
    pub visualize_visbuffer: bool,
    pub print_leaf_clusters: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryInstance {
    pub entity: EntityId,
    pub source_model: Option<ResourceId>,
    pub transform: Transform,
    pub cluster_offset: u32,
    pub cluster_count: u32,
    pub page_offset: u32,
    pub page_count: u32,
    pub mesh_name: Option<String>,
    pub source_hint: Option<String>,
}

impl Default for RenderVirtualGeometryInstance {
    fn default() -> Self {
        Self {
            entity: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 0,
            page_offset: 0,
            page_count: 0,
            mesh_name: None,
            source_hint: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryExtract {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub clusters: Vec<RenderVirtualGeometryCluster>,
    pub hierarchy_nodes: Vec<RenderVirtualGeometryHierarchyNode>,
    pub hierarchy_child_ids: Vec<u32>,
    pub pages: Vec<RenderVirtualGeometryPage>,
    pub instances: Vec<RenderVirtualGeometryInstance>,
    pub debug: RenderVirtualGeometryDebugState,
}

impl Default for RenderVirtualGeometryExtract {
    fn default() -> Self {
        Self {
            cluster_budget: 0,
            page_budget: 0,
            clusters: Vec::new(),
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: Vec::new(),
            debug: RenderVirtualGeometryDebugState::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderHybridGiQuality {
    Low,
    Medium,
    High,
}

impl Default for RenderHybridGiQuality {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderHybridGiDebugView {
    None,
    Cards,
    SurfaceCache,
    VoxelClipmap,
    InputSet,
}

impl Default for RenderHybridGiDebugView {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderHybridGiProbe {
    pub entity: EntityId,
    pub probe_id: u32,
    pub position: Vec3,
    pub radius: Real,
    pub parent_probe_id: Option<u32>,
    pub resident: bool,
    pub ray_budget: u32,
}

impl Default for RenderHybridGiProbe {
    fn default() -> Self {
        Self {
            entity: 0,
            probe_id: 0,
            position: Vec3::ZERO,
            radius: 0.0,
            parent_probe_id: None,
            resident: false,
            ray_budget: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderHybridGiTraceRegion {
    pub entity: EntityId,
    pub region_id: u32,
    pub bounds_center: Vec3,
    pub bounds_radius: Real,
    pub screen_coverage: Real,
    pub rt_lighting_rgb: [u8; 3],
}

impl Default for RenderHybridGiTraceRegion {
    fn default() -> Self {
        Self {
            entity: 0,
            region_id: 0,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_coverage: 0.0,
            rt_lighting_rgb: [0, 0, 0],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderHybridGiExtract {
    pub enabled: bool,
    pub quality: RenderHybridGiQuality,
    pub trace_budget: u32,
    pub card_budget: u32,
    pub voxel_budget: u32,
    pub debug_view: RenderHybridGiDebugView,
    pub(crate) probe_budget: u32,
    pub(crate) tracing_budget: u32,
    pub(crate) probes: Vec<RenderHybridGiProbe>,
    pub(crate) trace_regions: Vec<RenderHybridGiTraceRegion>,
}

impl Default for RenderHybridGiExtract {
    fn default() -> Self {
        Self {
            enabled: false,
            quality: RenderHybridGiQuality::Medium,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            debug_view: RenderHybridGiDebugView::None,
            probe_budget: 0,
            tracing_budget: 0,
            probes: Vec::new(),
            trace_regions: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleSpriteSnapshot {
    pub entity: EntityId,
    pub position: Vec3,
    pub size: Real,
    pub color: Vec4,
    pub intensity: Real,
}

impl Default for RenderParticleSpriteSnapshot {
    fn default() -> Self {
        Self {
            entity: 0,
            position: Vec3::ZERO,
            size: 0.0,
            color: Vec4::ZERO,
            intensity: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewEnvironmentExtract {
    pub lighting_enabled: bool,
    pub skybox_enabled: bool,
    pub fallback_skybox: FallbackSkyboxKind,
    pub clear_color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSceneGeometryExtract {
    pub camera: ViewportCameraSnapshot,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub point_lights: Vec<RenderPointLightSnapshot>,
    pub spot_lights: Vec<RenderSpotLightSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportRenderPacket {
    pub scene: RenderSceneGeometryExtract,
    pub overlays: RenderOverlayExtract,
    pub preview: PreviewEnvironmentExtract,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

pub type RenderExtractPacket = SceneViewportRenderPacket;
pub type RenderSceneSnapshot = SceneViewportRenderPacket;
