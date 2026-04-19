use serde::{Deserialize, Serialize};

use super::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, MaterialAsset, ModelAsset, PhysicsMaterialAsset, SceneAsset,
    ShaderAsset, TextureAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportedAsset {
    Texture(TextureAsset),
    Shader(ShaderAsset),
    Material(MaterialAsset),
    PhysicsMaterial(PhysicsMaterialAsset),
    Scene(SceneAsset),
    Model(ModelAsset),
    AnimationSkeleton(AnimationSkeletonAsset),
    AnimationClip(AnimationClipAsset),
    AnimationSequence(AnimationSequenceAsset),
    AnimationGraph(AnimationGraphAsset),
    AnimationStateMachine(AnimationStateMachineAsset),
    UiLayout(UiLayoutAsset),
    UiWidget(UiWidgetAsset),
    UiStyle(UiStyleAsset),
}
