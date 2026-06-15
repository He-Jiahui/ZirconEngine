use crate::asset::PrefabInstanceAsset;
use serde::{Deserialize, Serialize};

use super::animation::{
    SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset,
    SceneAnimationSkeletonAsset, SceneAnimationStateMachinePlayerAsset,
};
use super::camera::SceneCameraAsset;
use super::defaults::{default_render_layer_mask, default_scene_active};
use super::extensions::{SceneScriptBindingAsset, SceneTerrainAsset, SceneTileMapAsset};
use super::lighting::{
    SceneAmbientLightAsset, SceneDirectionalLightAsset, ScenePointLightAsset, SceneRectLightAsset,
    SceneSpotLightAsset,
};
use super::mesh::SceneMeshInstanceAsset;
use super::physics::{SceneColliderAsset, SceneJointAsset, SceneRigidBodyAsset};
use super::post_process::ScenePostProcessVolumeAsset;
use super::transform::TransformAsset;
use super::SceneMobilityAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityAsset {
    pub entity: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub transform: TransformAsset,
    #[serde(default = "default_scene_active")]
    pub active: bool,
    #[serde(default = "default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: SceneMobilityAsset,
    pub camera: Option<SceneCameraAsset>,
    pub mesh: Option<SceneMeshInstanceAsset>,
    #[serde(default)]
    pub ambient_light: Option<SceneAmbientLightAsset>,
    pub directional_light: Option<SceneDirectionalLightAsset>,
    #[serde(default)]
    pub point_light: Option<ScenePointLightAsset>,
    #[serde(default)]
    pub rect_light: Option<SceneRectLightAsset>,
    #[serde(default)]
    pub spot_light: Option<SceneSpotLightAsset>,
    #[serde(default)]
    pub post_process_volume: Option<ScenePostProcessVolumeAsset>,
    #[serde(default)]
    pub rigid_body: Option<SceneRigidBodyAsset>,
    #[serde(default)]
    pub collider: Option<SceneColliderAsset>,
    #[serde(default)]
    pub joint: Option<SceneJointAsset>,
    #[serde(default)]
    pub animation_skeleton: Option<SceneAnimationSkeletonAsset>,
    #[serde(default)]
    pub animation_player: Option<SceneAnimationPlayerAsset>,
    #[serde(default)]
    pub animation_sequence_player: Option<SceneAnimationSequencePlayerAsset>,
    #[serde(default)]
    pub animation_graph_player: Option<SceneAnimationGraphPlayerAsset>,
    #[serde(default)]
    pub animation_state_machine_player: Option<SceneAnimationStateMachinePlayerAsset>,
    #[serde(default)]
    pub terrain: Option<SceneTerrainAsset>,
    #[serde(default)]
    pub tilemap: Option<SceneTileMapAsset>,
    #[serde(default)]
    pub prefab_instance: Option<PrefabInstanceAsset>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub script_bindings: Vec<SceneScriptBindingAsset>,
}

// Read-only management DTOs keep scene authoring payloads stable while asset
// panels can inspect entity composition without walking every component type.
