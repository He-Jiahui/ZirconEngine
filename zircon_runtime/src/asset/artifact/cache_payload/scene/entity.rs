use serde::{Deserialize, Serialize};

use super::{
    physics::{
        ArtifactCacheSceneColliderAsset, ArtifactCacheSceneJointAsset,
        ArtifactCacheSceneRigidBodyAsset,
    },
    rendering::{ArtifactCacheSceneCameraAsset, ArtifactCacheSceneMeshInstanceAsset},
    script::ArtifactCacheSceneScriptBindingAsset,
};
use crate::asset::AssetImportError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneEntityAsset {
    entity: u64,
    name: String,
    parent: Option<u64>,
    transform: crate::asset::TransformAsset,
    active: bool,
    render_layer_mask: u32,
    mobility: crate::asset::SceneMobilityAsset,
    camera: Option<ArtifactCacheSceneCameraAsset>,
    mesh: Option<ArtifactCacheSceneMeshInstanceAsset>,
    ambient_light: Option<crate::asset::SceneAmbientLightAsset>,
    directional_light: Option<crate::asset::SceneDirectionalLightAsset>,
    point_light: Option<crate::asset::ScenePointLightAsset>,
    rect_light: Option<crate::asset::SceneRectLightAsset>,
    spot_light: Option<crate::asset::SceneSpotLightAsset>,
    post_process_volume: Option<crate::asset::ScenePostProcessVolumeAsset>,
    rigid_body: Option<ArtifactCacheSceneRigidBodyAsset>,
    collider: Option<ArtifactCacheSceneColliderAsset>,
    joint: Option<ArtifactCacheSceneJointAsset>,
    animation_skeleton: Option<crate::asset::SceneAnimationSkeletonAsset>,
    animation_player: Option<crate::asset::SceneAnimationPlayerAsset>,
    animation_sequence_player: Option<crate::asset::SceneAnimationSequencePlayerAsset>,
    animation_graph_player: Option<crate::asset::SceneAnimationGraphPlayerAsset>,
    animation_state_machine_player: Option<crate::asset::SceneAnimationStateMachinePlayerAsset>,
    terrain: Option<crate::asset::SceneTerrainAsset>,
    tilemap: Option<crate::asset::SceneTileMapAsset>,
    prefab_instance: Option<crate::asset::PrefabInstanceAsset>,
    script_bindings: Vec<ArtifactCacheSceneScriptBindingAsset>,
}

impl From<&crate::asset::SceneEntityAsset> for ArtifactCacheSceneEntityAsset {
    fn from(asset: &crate::asset::SceneEntityAsset) -> Self {
        Self {
            entity: asset.entity,
            name: asset.name.clone(),
            parent: asset.parent,
            transform: asset.transform,
            active: asset.active,
            render_layer_mask: asset.render_layer_mask,
            mobility: asset.mobility,
            camera: asset
                .camera
                .as_ref()
                .map(ArtifactCacheSceneCameraAsset::from),
            mesh: asset
                .mesh
                .as_ref()
                .map(ArtifactCacheSceneMeshInstanceAsset::from),
            ambient_light: asset.ambient_light.clone(),
            directional_light: asset.directional_light.clone(),
            point_light: asset.point_light.clone(),
            rect_light: asset.rect_light.clone(),
            spot_light: asset.spot_light.clone(),
            post_process_volume: asset.post_process_volume,
            rigid_body: asset
                .rigid_body
                .as_ref()
                .map(ArtifactCacheSceneRigidBodyAsset::from),
            collider: asset
                .collider
                .as_ref()
                .map(ArtifactCacheSceneColliderAsset::from),
            joint: asset.joint.as_ref().map(ArtifactCacheSceneJointAsset::from),
            animation_skeleton: asset.animation_skeleton.clone(),
            animation_player: asset.animation_player.clone(),
            animation_sequence_player: asset.animation_sequence_player.clone(),
            animation_graph_player: asset.animation_graph_player.clone(),
            animation_state_machine_player: asset.animation_state_machine_player.clone(),
            terrain: asset.terrain.clone(),
            tilemap: asset.tilemap.clone(),
            prefab_instance: asset.prefab_instance.clone(),
            script_bindings: asset
                .script_bindings
                .iter()
                .map(ArtifactCacheSceneScriptBindingAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneEntityAsset {
    pub(super) fn into_asset(self) -> Result<crate::asset::SceneEntityAsset, AssetImportError> {
        Ok(crate::asset::SceneEntityAsset {
            entity: self.entity,
            name: self.name,
            parent: self.parent,
            transform: self.transform,
            active: self.active,
            render_layer_mask: self.render_layer_mask,
            mobility: self.mobility,
            camera: self.camera.map(ArtifactCacheSceneCameraAsset::into_asset),
            mesh: self
                .mesh
                .map(ArtifactCacheSceneMeshInstanceAsset::into_asset),
            ambient_light: self.ambient_light,
            directional_light: self.directional_light,
            point_light: self.point_light,
            rect_light: self.rect_light,
            spot_light: self.spot_light,
            post_process_volume: self.post_process_volume,
            rigid_body: self
                .rigid_body
                .map(ArtifactCacheSceneRigidBodyAsset::into_asset),
            collider: self
                .collider
                .map(ArtifactCacheSceneColliderAsset::into_asset),
            joint: self.joint.map(ArtifactCacheSceneJointAsset::into_asset),
            animation_skeleton: self.animation_skeleton,
            animation_player: self.animation_player,
            animation_sequence_player: self.animation_sequence_player,
            animation_graph_player: self.animation_graph_player,
            animation_state_machine_player: self.animation_state_machine_player,
            terrain: self.terrain,
            tilemap: self.tilemap,
            prefab_instance: self.prefab_instance,
            script_bindings: self
                .script_bindings
                .into_iter()
                .map(ArtifactCacheSceneScriptBindingAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
