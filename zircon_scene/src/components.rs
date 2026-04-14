//! ECS-style components, scheduling, and render extraction snapshots.

use serde::{Deserialize, Serialize};
use zircon_math::{Transform, Vec3, Vec4};
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
pub struct Active(pub bool);

impl Default for Active {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
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
    pub intensity: f32,
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
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCameraSnapshot {
    pub node_id: EntityId,
    pub transform: Transform,
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: EntityId,
    pub transform: Transform,
    pub model: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub tint: Vec4,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGizmoSnapshot {
    pub target_node: EntityId,
    pub origin: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderExtractPacket {
    pub camera: RenderCameraSnapshot,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub light: RenderDirectionalLightSnapshot,
    pub selected_node: Option<EntityId>,
    pub gizmo: Option<RenderGizmoSnapshot>,
    pub show_grid: bool,
}

pub type RenderSceneSnapshot = RenderExtractPacket;
