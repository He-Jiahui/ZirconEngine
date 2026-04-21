use serde::{Deserialize, Serialize};

use super::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, FontAsset, MaterialAsset, ModelAsset, PhysicsMaterialAsset,
    SceneAsset, ShaderAsset, SoundAsset, TextureAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportedAsset {
    Texture(TextureAsset),
    Shader(ShaderAsset),
    Material(MaterialAsset),
    Sound(SoundAsset),
    Font(FontAsset),
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
