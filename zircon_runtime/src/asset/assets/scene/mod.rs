mod animation;
mod asset;
mod camera;
mod defaults;
mod entity;
mod extensions;
mod lighting;
mod management;
mod mesh;
mod physics;
mod post_process;
mod transform;

pub use animation::{
    SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset,
    SceneAnimationSkeletonAsset, SceneAnimationStateMachinePlayerAsset,
};
pub use asset::SceneAsset;
pub use camera::{SceneCameraAsset, SceneCameraTargetAsset, SceneViewportRectAsset};
pub use entity::SceneEntityAsset;
pub use extensions::{SceneScriptBindingAsset, SceneTerrainAsset, SceneTileMapAsset};
pub use lighting::{
    SceneAmbientLightAsset, SceneDirectionalLightAsset, ScenePointLightAsset, SceneRectLightAsset,
    SceneSpotLightAsset,
};
pub use management::{
    SceneAssetManagementRecord, SceneAssetManagementRecordSet,
    SceneAssetManagementRecordSetSummary, SceneAssetOverview, SceneEntityManagementRecord,
    SceneEntityManagementRecordSet, SceneEntityManagementRecordSetSummary, SceneEntityOverview,
};
pub use mesh::{SceneMeshInstanceAsset, SceneMeshLodLevelAsset, SceneMeshPrimitiveBindingAsset};
pub use physics::{
    SceneColliderAsset, SceneColliderShapeAsset, SceneJointAsset, SceneJointKindAsset,
    SceneRigidBodyAsset, SceneRigidBodyTypeAsset,
};
pub use post_process::{
    SceneBloomSettingsAsset, SceneChromaticAberrationSettingsAsset, SceneColorGradingSettingsAsset,
    SceneDitherSettingsAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset,
    ScenePostProcessVolumeProfileAsset, SceneTonemapOperatorAsset, SceneTonemapSettingsAsset,
    SceneVignetteSettingsAsset,
};
pub use transform::TransformAsset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum SceneMobilityAsset {
    #[default]
    Dynamic,
    Static,
}
