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
pub struct UiLayoutMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UiWidgetMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UiStyleMarker;

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

impl ResourceMarker for UiLayoutMarker {
    const KIND: ResourceKind = ResourceKind::UiLayout;
}

impl ResourceMarker for UiWidgetMarker {
    const KIND: ResourceKind = ResourceKind::UiWidget;
}

impl ResourceMarker for UiStyleMarker {
    const KIND: ResourceKind = ResourceKind::UiStyle;
}
