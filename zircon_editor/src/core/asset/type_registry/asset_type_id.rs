use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use zircon_runtime_interface::resource::ResourceKind;

/// Stable, open identifier shared by runtime resource types and plugin-defined asset types.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetTypeId(String);

impl AssetTypeId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AssetTypeIdError> {
        let value = value.into();
        if is_valid_asset_type_id(&value) {
            Ok(Self(value))
        } else {
            Err(AssetTypeIdError::InvalidAssetTypeId { value })
        }
    }

    pub fn from_resource_kind(kind: ResourceKind) -> Self {
        Self(canonical_resource_kind_id(kind).to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for AssetTypeId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AssetTypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for AssetTypeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AssetTypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetTypeIdError {
    InvalidAssetTypeId { value: String },
}

impl fmt::Display for AssetTypeIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAssetTypeId { value } => {
                write!(formatter, "asset type id `{value}` is not canonical")
            }
        }
    }
}

impl std::error::Error for AssetTypeIdError {}

fn is_valid_asset_type_id(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_valid_segment)
}

fn is_valid_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '_')
}

pub(super) fn canonical_resource_kind_id(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Data => "data",
        ResourceKind::Model => "model",
        ResourceKind::Mesh => "mesh",
        ResourceKind::Material => "material",
        ResourceKind::MaterialGraph => "material.graph",
        ResourceKind::Texture => "texture",
        ResourceKind::Shader => "shader",
        ResourceKind::Scene => "scene",
        ResourceKind::Sound => "sound",
        ResourceKind::Font => "font",
        ResourceKind::PhysicsMaterial => "physics.material",
        ResourceKind::NavMesh => "navigation.mesh",
        ResourceKind::NavigationSettings => "navigation.settings",
        ResourceKind::Terrain => "terrain.heightfield",
        ResourceKind::TerrainLayerStack => "terrain.layer_stack",
        ResourceKind::TileSet => "tilemap_2d.tileset",
        ResourceKind::TileMap => "tilemap_2d.tilemap",
        ResourceKind::Prefab => "prefab.asset",
        ResourceKind::AnimationSkeleton => "animation.skeleton",
        ResourceKind::AnimationClip => "animation.clip",
        ResourceKind::AnimationSequence => "animation.sequence",
        ResourceKind::AnimationGraph => "animation.graph",
        ResourceKind::AnimationStateMachine => "animation.state_machine",
        ResourceKind::UiLayout => "ui.layout",
        ResourceKind::UiWidget => "ui.widget",
        ResourceKind::UiStyle => "ui.style",
    }
}
