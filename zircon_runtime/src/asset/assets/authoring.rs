use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::asset::{AssetReference, AssetUri};
use crate::core::math::Real;

use super::scene::{SceneAsset, TransformAsset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerrainAsset {
    pub uri: AssetUri,
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_terrain_sample_spacing")]
    pub sample_spacing: Real,
    #[serde(default = "default_terrain_height_scale")]
    pub height_scale: Real,
    #[serde(default)]
    pub height_samples: Vec<Real>,
    #[serde(default)]
    pub layers: Vec<TerrainLayerAsset>,
}

impl TerrainAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.layers
            .iter()
            .flat_map(TerrainLayerAsset::direct_references)
            .collect()
    }

    pub fn validate_dimensions(&self) -> Result<(), String> {
        let expected = self.width as usize * self.height as usize;
        if !self.height_samples.is_empty() && self.height_samples.len() != expected {
            return Err(format!(
                "terrain `{}` declares {}x{} samples but stores {}",
                self.name,
                self.width,
                self.height,
                self.height_samples.len()
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerrainLayerAsset {
    pub name: String,
    #[serde(default)]
    pub material: Option<AssetReference>,
    #[serde(default)]
    pub weightmap: Option<AssetReference>,
    #[serde(default = "default_terrain_layer_strength")]
    pub strength: Real,
}

impl TerrainLayerAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.material
            .iter()
            .chain(self.weightmap.iter())
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerrainLayerStackAsset {
    pub uri: AssetUri,
    #[serde(default)]
    pub layers: Vec<TerrainLayerAsset>,
}

impl TerrainLayerStackAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.layers
            .iter()
            .flat_map(TerrainLayerAsset::direct_references)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileMapProjectionAsset {
    #[default]
    Orthogonal,
    IsometricDiamond,
    IsometricStaggered,
    HexagonalStaggered,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileSetAsset {
    pub uri: AssetUri,
    pub tile_width: u32,
    pub tile_height: u32,
    pub image: AssetReference,
    #[serde(default)]
    pub tiles: Vec<TileSetTileAsset>,
}

impl TileSetAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        vec![self.image.clone()]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileSetTileAsset {
    pub id: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub collider: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileMapAsset {
    pub uri: AssetUri,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub projection: TileMapProjectionAsset,
    pub tile_set: AssetReference,
    #[serde(default)]
    pub layers: Vec<TileMapLayerAsset>,
}

impl TileMapAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        vec![self.tile_set.clone()]
    }

    pub fn validate_layers(&self) -> Result<(), String> {
        let expected = self.width as usize * self.height as usize;
        for layer in &self.layers {
            if layer.tiles.len() != expected {
                return Err(format!(
                    "tilemap layer `{}` stores {} tiles for {}x{} map",
                    layer.name,
                    layer.tiles.len(),
                    self.width,
                    self.height
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileMapLayerAsset {
    pub name: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_layer_opacity")]
    pub opacity: Real,
    #[serde(default)]
    pub tiles: Vec<Option<u32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabAsset {
    pub uri: AssetUri,
    pub name: String,
    pub scene: SceneAsset,
    #[serde(default)]
    pub exposed_properties: Vec<String>,
}

impl PrefabAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.scene.direct_references()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabInstanceAsset {
    pub prefab: AssetReference,
    #[serde(default)]
    pub local_transform: TransformAsset,
    #[serde(default)]
    pub overrides: Vec<PrefabPropertyOverrideAsset>,
}

impl PrefabInstanceAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        vec![self.prefab.clone()]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabPropertyOverrideAsset {
    pub entity_path: String,
    pub property_path: String,
    #[serde(default)]
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphAsset {
    pub uri: AssetUri,
    pub name: String,
    #[serde(default)]
    pub shader: Option<AssetReference>,
    #[serde(default)]
    pub nodes: Vec<MaterialGraphNodeAsset>,
    #[serde(default)]
    pub links: Vec<MaterialGraphLinkAsset>,
    #[serde(default)]
    pub parameters: BTreeMap<String, MaterialGraphParameterAsset>,
}

impl MaterialGraphAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.shader
            .iter()
            .cloned()
            .chain(self.nodes.iter().filter_map(|node| match &node.kind {
                MaterialGraphNodeKindAsset::TextureSample { texture } => Some(texture.clone()),
                _ => None,
            }))
            .collect()
    }

    pub fn validate_output_node(&self) -> Result<(), String> {
        if self
            .nodes
            .iter()
            .any(|node| matches!(node.kind, MaterialGraphNodeKindAsset::Output))
        {
            return Ok(());
        }
        Err(format!("material graph `{}` has no output node", self.name))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphNodeAsset {
    pub id: String,
    #[serde(default)]
    pub position: [Real; 2],
    pub kind: MaterialGraphNodeKindAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialGraphNodeKindAsset {
    Output,
    TextureSample { texture: AssetReference },
    ScalarParameter { name: String, default: Real },
    VectorParameter { name: String, default: [Real; 4] },
    Add,
    Multiply,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphLinkAsset {
    pub from_node: String,
    pub from_pin: String,
    pub to_node: String,
    pub to_pin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum MaterialGraphParameterAsset {
    Scalar(Real),
    Vector([Real; 4]),
}

const fn default_terrain_sample_spacing() -> Real {
    1.0
}

const fn default_terrain_height_scale() -> Real {
    1.0
}

const fn default_terrain_layer_strength() -> Real {
    1.0
}

const fn default_layer_opacity() -> Real {
    1.0
}

const fn default_true() -> bool {
    true
}
