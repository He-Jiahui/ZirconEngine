//! ECS-style components, scheduling, and render extraction snapshots.

use serde::{Deserialize, Serialize};
use zircon_math::{Mat4, Real, Transform, UVec2, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemStage {
    PreUpdate,
    Update,
    LateUpdate,
    FixedUpdate,
    RenderExtract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub stages: Vec<SystemStage>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            stages: vec![
                SystemStage::PreUpdate,
                SystemStage::Update,
                SystemStage::LateUpdate,
                SystemStage::FixedUpdate,
                SystemStage::RenderExtract,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Camera,
    Cube,
    Mesh,
    DirectionalLight,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Hierarchy {
    pub parent: Option<EntityId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub transform: Transform,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMatrix(pub Mat4);

impl Default for WorldMatrix {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldTransform {
    pub transform: Transform,
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSelf(pub bool);

impl Default for ActiveSelf {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveInHierarchy(pub bool);

impl Default for ActiveInHierarchy {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderLayerMask(pub u32);

impl Default for RenderLayerMask {
    fn default() -> Self {
        Self(default_render_layer_mask())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Mobility {
    Dynamic,
    Static,
}

impl Default for Mobility {
    fn default() -> Self {
        Self::Dynamic
    }
}

pub type Active = ActiveSelf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_y_radians: Real,
    pub z_near: Real,
    pub z_far: Real,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub model: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub tint: Vec4,
}

impl MeshRenderer {
    pub fn from_handles(
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            model,
            material,
            tint: Vec4::ONE,
        }
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::from_handles(
            ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
            ResourceHandle::new(ResourceId::from_stable_label("builtin://material/default")),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -1.0, -0.25).normalize_or_zero(),
            color: Vec3::splat(1.0),
            intensity: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    pub directional_light: Option<DirectionalLight>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub active: bool,
    #[serde(default = "default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: Mobility,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportCameraSnapshot {
    pub transform: Transform,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub aspect_ratio: Real,
}

impl ViewportCameraSnapshot {
    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.aspect_ratio = aspect_ratio_from_viewport_size(viewport_size);
    }
}

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
            page_id: 0,
            lod_level: 0,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_space_error: 0.0,
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryExtract {
    pub cluster_budget: u32,
    pub page_budget: u32,
    pub clusters: Vec<RenderVirtualGeometryCluster>,
    pub pages: Vec<RenderVirtualGeometryPage>,
}

impl Default for RenderVirtualGeometryExtract {
    fn default() -> Self {
        Self {
            cluster_budget: 0,
            page_budget: 0,
            clusters: Vec::new(),
            pages: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderHybridGiProbe {
    pub entity: EntityId,
    pub probe_id: u32,
    pub position: Vec3,
    pub radius: Real,
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
}

impl Default for RenderHybridGiTraceRegion {
    fn default() -> Self {
        Self {
            entity: 0,
            region_id: 0,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.0,
            screen_coverage: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderHybridGiExtract {
    pub probe_budget: u32,
    pub tracing_budget: u32,
    pub probes: Vec<RenderHybridGiProbe>,
    pub trace_regions: Vec<RenderHybridGiTraceRegion>,
}

impl Default for RenderHybridGiExtract {
    fn default() -> Self {
        Self {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneViewportTool {
    Drag,
    Move,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformSpace {
    Local,
    Global,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewOrientation {
    User,
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridMode {
    Hidden,
    VisibleNoSnap,
    VisibleAndSnap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Shaded,
    WireOverlay,
    WireOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackSkyboxKind {
    None,
    ProceduralGradient,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneViewportSettings {
    pub tool: SceneViewportTool,
    pub transform_space: TransformSpace,
    pub projection_mode: ProjectionMode,
    pub view_orientation: ViewOrientation,
    pub gizmos_enabled: bool,
    pub display_mode: DisplayMode,
    pub grid_mode: GridMode,
    pub translate_step: Real,
    pub rotate_step_deg: Real,
    pub scale_step: Real,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

impl Default for SceneViewportSettings {
    fn default() -> Self {
        Self {
            tool: SceneViewportTool::Move,
            transform_space: TransformSpace::Local,
            projection_mode: ProjectionMode::Perspective,
            view_orientation: ViewOrientation::User,
            gizmos_enabled: true,
            display_mode: DisplayMode::Shaded,
            grid_mode: GridMode::VisibleNoSnap,
            translate_step: 1.0,
            rotate_step_deg: 15.0,
            scale_step: 0.1,
            preview_lighting: true,
            preview_skybox: true,
        }
    }
}

impl Default for ViewportCameraSnapshot {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: 5.0,
            z_near: 0.1,
            z_far: 200.0,
            aspect_ratio: default_viewport_aspect_ratio(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportExtractRequest {
    pub settings: SceneViewportSettings,
    pub selection: Vec<EntityId>,
    pub active_camera_override: Option<EntityId>,
    pub camera: Option<ViewportCameraSnapshot>,
    pub viewport_size: Option<UVec2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneGizmoKind {
    Camera,
    DirectionalLight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewportIconId {
    Camera,
    DirectionalLight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayLineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayWireShape {
    Frustum {
        transform: Transform,
        fov_y_radians: Real,
        aspect_ratio: Real,
        z_near: Real,
        z_far: Real,
        color: Vec4,
    },
    Arrow {
        origin: Vec3,
        direction: Vec3,
        length: Real,
        color: Vec4,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayBillboardIcon {
    pub id: ViewportIconId,
    pub position: Vec3,
    pub tint: Vec4,
    pub size: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayPickShape {
    Sphere {
        center: Vec3,
        radius: Real,
    },
    Segment {
        start: Vec3,
        end: Vec3,
        thickness: Real,
    },
    Circle {
        center: Vec3,
        normal: Vec3,
        radius: Real,
        thickness: Real,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneGizmoOverlayExtract {
    pub owner: EntityId,
    pub kind: SceneGizmoKind,
    pub selected: bool,
    pub lines: Vec<OverlayLineSegment>,
    pub wire_shapes: Vec<OverlayWireShape>,
    pub icons: Vec<OverlayBillboardIcon>,
    pub pick_shapes: Vec<OverlayPickShape>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionHighlightExtract {
    pub owner: EntityId,
    pub outline: bool,
    pub tint: Option<Vec4>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionAnchorExtract {
    pub owner: EntityId,
    pub position: Vec3,
    pub size: Real,
    pub color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridOverlayExtract {
    pub visible: bool,
    pub snap_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandleElementExtract {
    AxisLine {
        axis: OverlayAxis,
        start: Vec3,
        end: Vec3,
        color: Vec4,
        pick_radius: Real,
    },
    AxisRing {
        axis: OverlayAxis,
        center: Vec3,
        normal: Vec3,
        radius: Real,
        color: Vec4,
        pick_radius: Real,
    },
    AxisScale {
        axis: OverlayAxis,
        start: Vec3,
        end: Vec3,
        color: Vec4,
        pick_radius: Real,
        handle_size: Real,
    },
    CenterAnchor {
        position: Vec3,
        size: Real,
        color: Vec4,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct HandleOverlayExtract {
    pub owner: EntityId,
    pub tool: SceneViewportTool,
    pub space: TransformSpace,
    pub origin: Transform,
    pub elements: Vec<HandleElementExtract>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderOverlayExtract {
    pub selection: Vec<SelectionHighlightExtract>,
    pub selection_anchors: Vec<SelectionAnchorExtract>,
    pub grid: Option<GridOverlayExtract>,
    pub handles: Vec<HandleOverlayExtract>,
    pub scene_gizmos: Vec<SceneGizmoOverlayExtract>,
    pub display_mode: DisplayMode,
}

impl Default for RenderOverlayExtract {
    fn default() -> Self {
        Self {
            selection: Vec::new(),
            selection_anchors: Vec::new(),
            grid: None,
            handles: Vec::new(),
            scene_gizmos: Vec::new(),
            display_mode: DisplayMode::Shaded,
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
    pub lights: Vec<RenderDirectionalLightSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportRenderPacket {
    pub scene: RenderSceneGeometryExtract,
    pub overlays: RenderOverlayExtract,
    pub preview: PreviewEnvironmentExtract,
}

pub type RenderExtractPacket = SceneViewportRenderPacket;
pub type RenderSceneSnapshot = SceneViewportRenderPacket;

pub const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}

pub const fn default_viewport_aspect_ratio() -> Real {
    16.0 / 9.0
}

pub fn aspect_ratio_from_viewport_size(viewport_size: UVec2) -> Real {
    viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real
}

pub struct SceneGizmoBuildContext<'a> {
    pub world: &'a crate::World,
    pub entity: EntityId,
    pub selected: bool,
    pub camera: &'a ViewportCameraSnapshot,
}

pub trait SceneGizmoProvider: Send + Sync {
    fn kind(&self) -> SceneGizmoKind;
    fn supports(&self, world: &crate::World, entity: EntityId) -> bool;
    fn build(&self, ctx: &SceneGizmoBuildContext<'_>, out: &mut SceneGizmoOverlayExtract);
}

pub struct SceneGizmoRegistry {
    pub providers: Vec<Box<dyn SceneGizmoProvider>>,
}

impl SceneGizmoRegistry {
    pub fn new(providers: Vec<Box<dyn SceneGizmoProvider>>) -> Self {
        Self { providers }
    }
}
