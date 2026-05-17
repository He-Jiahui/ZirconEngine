//! ECS-style scene components plus schedule and scene-domain mobility glue.

mod render2d;
mod scene;

pub use crate::core::framework::scene::Mobility;
pub use render2d::{Mesh2dComponent, Sprite2dComponent};
pub use scene::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf,
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, CameraComponent,
    ColliderComponent, ColliderShape, DirectionalLight, Hierarchy, JointComponent, JointKind,
    LocalTransform, MeshRenderer, Name, NodeKind, NodeRecord, PointLight, RenderLayerMask,
    RigidBodyComponent, RigidBodyType, SceneNode, SpotLight, WorldMatrix, WorldTransform,
};
