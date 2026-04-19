//! ECS-style scene components plus schedule and scene-domain mobility glue.

mod scene;
mod schedule;

pub use scene::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf, CameraComponent,
    DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Name, NodeKind, NodeRecord,
    RenderLayerMask, SceneNode, WorldMatrix, WorldTransform,
};
pub use schedule::{Schedule, SystemStage};
pub use crate::core::framework::scene::Mobility;
