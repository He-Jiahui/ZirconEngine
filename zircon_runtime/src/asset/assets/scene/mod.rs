#[path = "animation.rs"]
mod animation;
#[path = "asset.rs"]
mod asset;
#[path = "camera.rs"]
mod camera;
#[path = "component.rs"]
mod component;
#[path = "defaults.rs"]
mod defaults;
#[path = "entity.rs"]
mod entity;
#[path = "extensions.rs"]
mod extensions;
#[path = "lighting.rs"]
mod lighting;
#[path = "management.rs"]
mod management;
#[path = "mesh.rs"]
mod mesh;
#[path = "physics.rs"]
mod physics;
#[path = "post_process.rs"]
mod post_process;
#[path = "render2d.rs"]
mod render2d;
#[path = "transform.rs"]
mod transform;

pub use animation::{
    SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset,
    SceneAnimationSkeletonAsset, SceneAnimationStateMachinePlayerAsset,
};
pub use asset::SceneAsset;
pub use camera::{SceneCameraAsset, SceneCameraTargetAsset, SceneViewportRectAsset};
pub use component::SceneComponentAssetRecord;
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
    SceneAmbientOcclusionSettingsAsset, SceneAoQualityTierAsset, SceneBloomSettingsAsset,
    SceneChromaticAberrationSettingsAsset, SceneColorGradingSettingsAsset,
    SceneDitherSettingsAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset,
    ScenePostProcessVolumeProfileAsset, SceneTonemapOperatorAsset, SceneTonemapSettingsAsset,
    SceneVignetteSettingsAsset, SceneVolumetricFogSettingsAsset,
};
pub use render2d::{SceneMesh2dAsset, SceneSprite2dAsset};
pub use transform::TransformAsset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum SceneMobilityAsset {
    #[default]
    Dynamic,
    Static,
}
