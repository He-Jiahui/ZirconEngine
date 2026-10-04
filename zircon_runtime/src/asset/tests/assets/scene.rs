//! 场景资产测试使用仅供测试的直接 TOML 往返检查组件形状；项目级持久化的引用解析由 project_document 路径另行覆盖。

use crate::asset::{
    AssetReference, AssetUri, AssetUuid, SceneAmbientLightAsset, SceneAnimationGraphPlayerAsset,
    SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneBloomSettingsAsset, SceneCameraAsset,
    SceneCameraTargetAsset, SceneChromaticAberrationSettingsAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneColorGradingSettingsAsset, SceneDirectionalLightAsset,
    SceneDitherSettingsAsset, SceneEntityAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    SceneJointAsset, SceneJointKindAsset, SceneMeshInstanceAsset, SceneMobilityAsset,
    ScenePointLightAsset, ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset,
    ScenePostProcessVolumeAsset, ScenePostProcessVolumeProfileAsset, SceneRectLightAsset,
    SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneScriptBindingAsset, SceneSpotLightAsset,
    SceneTonemapOperatorAsset, SceneTonemapSettingsAsset, SceneViewportRectAsset,
    SceneVignetteSettingsAsset, SceneVolumetricFogSettingsAsset, TransformAsset,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::render::{CorePipelineKind, ProjectionMode, RenderCameraClearColor};
use crate::core::framework::scene::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};

mod camera;
mod foundation;
mod lights;
mod management;
mod physics_animation;
mod post_process;
mod script_bindings;
