use crate::asset::AssetReference;

/// Shader/material expectations declared by a renderer feature asset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RendererFeatureAssetReferences {
    pub shader: Option<AssetReference>,
    pub material: Option<AssetReference>,
    pub required_entry_points: Vec<String>,
    pub expected_properties: Vec<String>,
    pub expected_texture_slots: Vec<String>,
}
