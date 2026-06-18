use crate::core::math::{Real, Transform, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};

use crate::core::framework::scene::{EntityId, Mobility};

use super::light::{
    RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot, RenderPointLightSnapshot,
    RenderRectLightSnapshot, RenderSpotLightSnapshot,
};
use super::{FallbackSkyboxKind, RenderOverlayExtract, ViewportCameraSnapshot};

pub const RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS: u32 = 16;
pub const RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL: u32 =
    (1_u32 << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) - 1;

pub fn render_mesh_stable_instance_key(entity: EntityId, primitive_ordinal: u32) -> u64 {
    assert!(
        primitive_ordinal <= RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
        "render mesh primitive ordinal exceeds stable instance key packing range"
    );
    debug_assert!(
        entity <= (u64::MAX >> RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS),
        "entity id exceeds stable instance key packing range"
    );
    (entity << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) | u64::from(primitive_ordinal)
}

pub fn render_mesh_transform_revision(transform: &Transform) -> u64 {
    let mut revision = FNV_OFFSET_BASIS;
    for lane in transform.translation.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    for lane in transform.rotation.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    for lane in transform.scale.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    revision
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

const fn fnv1a_u32(mut hash: u64, value: u32) -> u64 {
    let bytes = value.to_le_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        index += 1;
    }
    hash
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderMeshStaticState {
    pub transform_static: bool,
    pub geometry_revision: u64,
    pub material_revision: u64,
}

impl RenderMeshStaticState {
    pub const fn new(
        transform_static: bool,
        geometry_revision: u64,
        material_revision: u64,
    ) -> Self {
        Self {
            transform_static,
            geometry_revision,
            material_revision,
        }
    }

    pub const fn from_transform_static(transform_static: bool) -> Self {
        Self {
            transform_static,
            geometry_revision: 0,
            material_revision: 0,
        }
    }

    pub const fn has_authoritative_revisions(self) -> bool {
        self.transform_static && self.geometry_revision != 0 && self.material_revision != 0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: EntityId,
    pub stable_instance_key: u64,
    pub transform_revision: u64,
    pub transform: Transform,
    pub model: ResourceHandle<ModelMarker>,
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    pub mesh_lod: Option<RenderMeshLodSelection>,
    pub morph_weights: Vec<Real>,
    pub tint: Vec4,
    pub mobility: Mobility,
    pub static_state: RenderMeshStaticState,
    pub render_layer_mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMeshLodSelection {
    pub level_index: u32,
    pub min_distance: Real,
}

impl RenderMeshLodSelection {
    pub fn new(level_index: u32, min_distance: Real) -> Self {
        Self {
            level_index,
            min_distance,
        }
    }
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageDependency {
    pub page_id: u32,
    pub parent_page_id: Option<u32>,
    /// Stable child list from cooked VG data; runtime may derive its parent map from either side.
    pub child_page_ids: Vec<u32>,
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
    pub page_dependencies: Vec<RenderVirtualGeometryPageDependency>,
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
            page_dependencies: Vec::new(),
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
pub struct RenderHybridGiProbe {
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
pub struct RenderHybridGiTraceRegion {
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
    pub probe_budget: u32,
    pub tracing_budget: u32,
    pub probes: Vec<RenderHybridGiProbe>,
    pub trace_regions: Vec<RenderHybridGiTraceRegion>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderParticleSpriteIdentity {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
}

impl RenderParticleSpriteIdentity {
    pub const fn new(entity: EntityId, stable_sprite_key: u64) -> Self {
        Self {
            entity,
            stable_sprite_key,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleSpriteSnapshot {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
    pub position: Vec3,
    pub size: Real,
    pub aspect_ratio: Real,
    pub billboard_offset: Vec2,
    pub rotation: Real,
    pub sort_order: i32,
    pub color: Vec4,
    pub intensity: Real,
    pub depth_test: bool,
    pub material: Option<ResourceHandle<MaterialMarker>>,
    pub texture: Option<ResourceHandle<TextureMarker>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleBillboardBasisSnapshot {
    pub right: Vec3,
    pub up: Vec3,
}

impl RenderParticleBillboardBasisSnapshot {
    pub const fn new(right: Vec3, up: Vec3) -> Self {
        Self { right, up }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticlePreviousSpriteSnapshot {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
    pub position: Vec3,
    pub size: Real,
    pub aspect_ratio: Real,
    pub billboard_offset: Vec2,
    pub rotation: Real,
    pub billboard_basis: Option<RenderParticleBillboardBasisSnapshot>,
}

impl RenderParticlePreviousSpriteSnapshot {
    pub fn from_current(sprite: &RenderParticleSpriteSnapshot) -> Self {
        Self {
            entity: sprite.entity,
            stable_sprite_key: sprite.stable_sprite_key,
            position: sprite.position,
            size: sprite.size,
            aspect_ratio: sprite.aspect_ratio,
            billboard_offset: sprite.billboard_offset,
            rotation: sprite.rotation,
            billboard_basis: None,
        }
    }

    pub fn from_current_with_billboard_basis(
        sprite: &RenderParticleSpriteSnapshot,
        right: Vec3,
        up: Vec3,
    ) -> Self {
        let mut previous = Self::from_current(sprite);
        previous.billboard_basis = Some(RenderParticleBillboardBasisSnapshot::new(right, up));
        previous
    }

    pub const fn identity(&self) -> RenderParticleSpriteIdentity {
        RenderParticleSpriteIdentity::new(self.entity, self.stable_sprite_key)
    }
}

impl RenderParticleSpriteSnapshot {
    pub const fn identity(&self) -> RenderParticleSpriteIdentity {
        RenderParticleSpriteIdentity::new(self.entity, self.stable_sprite_key)
    }
}

impl Default for RenderParticleSpriteSnapshot {
    fn default() -> Self {
        Self {
            entity: 0,
            stable_sprite_key: 0,
            position: Vec3::ZERO,
            size: 0.0,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::ZERO,
            intensity: 0.0,
            depth_test: true,
            material: None,
            texture: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleBoundsSnapshot {
    pub entity: EntityId,
    pub center: Vec3,
    pub radius: Real,
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
    pub ambient_lights: Vec<RenderAmbientLightSnapshot>,
    pub rect_lights: Vec<RenderRectLightSnapshot>,
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
