use serde::{Deserialize, Serialize};

pub trait ResourceMarker: Send + Sync + 'static {
    const KIND: ResourceKind;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    Model,
    Material,
    Texture,
    Shader,
    Scene,
    Sound,
    Font,
    PhysicsMaterial,
    AnimationSkeleton,
    AnimationClip,
    AnimationSequence,
    AnimationGraph,
    AnimationStateMachine,
    UiLayout,
    UiWidget,
    UiStyle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ModelMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MaterialMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextureMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SceneMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SoundMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FontMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UiLayoutMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UiWidgetMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UiStyleMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PhysicsMaterialMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimationSkeletonMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimationClipMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimationSequenceMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimationGraphMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimationStateMachineMarker;

impl ResourceMarker for ModelMarker {
    const KIND: ResourceKind = ResourceKind::Model;
}

impl ResourceMarker for MaterialMarker {
    const KIND: ResourceKind = ResourceKind::Material;
}

impl ResourceMarker for TextureMarker {
    const KIND: ResourceKind = ResourceKind::Texture;
}

impl ResourceMarker for ShaderMarker {
    const KIND: ResourceKind = ResourceKind::Shader;
}

impl ResourceMarker for SceneMarker {
    const KIND: ResourceKind = ResourceKind::Scene;
}

impl ResourceMarker for SoundMarker {
    const KIND: ResourceKind = ResourceKind::Sound;
}

impl ResourceMarker for FontMarker {
    const KIND: ResourceKind = ResourceKind::Font;
}

impl ResourceMarker for UiLayoutMarker {
    const KIND: ResourceKind = ResourceKind::UiLayout;
}

impl ResourceMarker for UiWidgetMarker {
    const KIND: ResourceKind = ResourceKind::UiWidget;
}

impl ResourceMarker for UiStyleMarker {
    const KIND: ResourceKind = ResourceKind::UiStyle;
}

impl ResourceMarker for PhysicsMaterialMarker {
    const KIND: ResourceKind = ResourceKind::PhysicsMaterial;
}

impl ResourceMarker for AnimationSkeletonMarker {
    const KIND: ResourceKind = ResourceKind::AnimationSkeleton;
}

impl ResourceMarker for AnimationClipMarker {
    const KIND: ResourceKind = ResourceKind::AnimationClip;
}

impl ResourceMarker for AnimationSequenceMarker {
    const KIND: ResourceKind = ResourceKind::AnimationSequence;
}

impl ResourceMarker for AnimationGraphMarker {
    const KIND: ResourceKind = ResourceKind::AnimationGraph;
}

impl ResourceMarker for AnimationStateMachineMarker {
    const KIND: ResourceKind = ResourceKind::AnimationStateMachine;
}
