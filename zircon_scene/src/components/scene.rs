use serde::{Deserialize, Serialize};
use zircon_math::{Mat4, Real, Transform, Vec3, Vec4};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::EntityId;

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

pub const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}
