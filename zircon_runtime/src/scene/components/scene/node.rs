use crate::core::framework::scene::Mobility;
use crate::core::math::Transform;
use crate::scene::EntityId;
use serde::{Deserialize, Serialize};

use super::super::{Mesh2dComponent, Sprite2dComponent};
use super::{
    AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    JointComponent, MeshRenderer, NodeKind, PointLight, RectLight, RigidBodyComponent, SpotLight,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    #[serde(default)]
    pub sprite_2d: Option<Sprite2dComponent>,
    #[serde(default)]
    pub mesh_2d: Option<Mesh2dComponent>,
    #[serde(default)]
    pub ambient_light: Option<AmbientLight>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub rect_light: Option<RectLight>,
    #[serde(default)]
    pub spot_light: Option<SpotLight>,
    pub rigid_body: Option<RigidBodyComponent>,
    pub collider: Option<ColliderComponent>,
    pub joint: Option<JointComponent>,
    pub animation_skeleton: Option<AnimationSkeletonComponent>,
    pub animation_player: Option<AnimationPlayerComponent>,
    pub animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    pub animation_graph_player: Option<AnimationGraphPlayerComponent>,
    pub animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
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
    #[serde(default)]
    pub sprite_2d: Option<Sprite2dComponent>,
    #[serde(default)]
    pub mesh_2d: Option<Mesh2dComponent>,
    #[serde(default)]
    pub ambient_light: Option<AmbientLight>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub rect_light: Option<RectLight>,
    #[serde(default)]
    pub spot_light: Option<SpotLight>,
    #[serde(default)]
    pub active: bool,
    #[serde(default = "super::default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: Mobility,
    #[serde(default)]
    pub rigid_body: Option<RigidBodyComponent>,
    #[serde(default)]
    pub collider: Option<ColliderComponent>,
    #[serde(default)]
    pub joint: Option<JointComponent>,
    #[serde(default)]
    pub animation_skeleton: Option<AnimationSkeletonComponent>,
    #[serde(default)]
    pub animation_player: Option<AnimationPlayerComponent>,
    #[serde(default)]
    pub animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    #[serde(default)]
    pub animation_graph_player: Option<AnimationGraphPlayerComponent>,
    #[serde(default)]
    pub animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
}
