use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;
use crate::graphics::feature::BuiltinRenderFeature;

use super::{RenderPassStage, RendererAsset, RendererFeatureAsset, RendererFeatureAssetReferences};

pub const RENDERER_DATA_DOCUMENT_VERSION: u32 = 1;

/// TOML-facing SRP renderer data asset shape before graph compile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RendererDataDocument {
    #[serde(default = "default_renderer_data_document_version")]
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub features: Vec<RendererFeatureDocument>,
}

impl RendererDataDocument {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn to_renderer_asset(&self) -> Result<RendererAsset, RendererDataDocumentError> {
        let stages = self
            .stages
            .iter()
            .map(|stage| parse_render_pass_stage(stage))
            .collect::<Result<Vec<_>, _>>()?;
        let features = self
            .features
            .iter()
            .map(RendererFeatureDocument::to_renderer_feature_asset)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RendererAsset {
            name: self.name.clone(),
            stages,
            features,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RendererFeatureDocument {
    pub name: String,
    pub source: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_gate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shader: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_entry_points: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_properties: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_texture_slots: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub local_config: BTreeMap<String, String>,
}

impl RendererFeatureDocument {
    pub fn to_renderer_feature_asset(
        &self,
    ) -> Result<RendererFeatureAsset, RendererDataDocumentError> {
        let source = parse_builtin_render_feature(&self.source)
            .map_err(|value| RendererDataDocumentError::UnknownRenderFeatureSource { value })?;
        let quality_gate = self
            .quality_gate
            .as_deref()
            .map(parse_builtin_render_feature)
            .transpose()
            .map_err(|value| RendererDataDocumentError::UnknownQualityGate { value })?;
        let mut asset = RendererFeatureAsset::builtin(source).with_enabled(self.enabled);

        asset.quality_gate = quality_gate;
        asset.local_config = self.local_config.clone();
        asset.asset_references = RendererFeatureAssetReferences {
            shader: self.shader.clone(),
            material: self.material.clone(),
            required_entry_points: self.required_entry_points.clone(),
            expected_properties: self.expected_properties.clone(),
            expected_texture_slots: self.expected_texture_slots.clone(),
        };

        Ok(asset)
    }
}

impl TryFrom<&RendererDataDocument> for RendererAsset {
    type Error = RendererDataDocumentError;

    fn try_from(document: &RendererDataDocument) -> Result<Self, Self::Error> {
        document.to_renderer_asset()
    }
}

impl TryFrom<RendererDataDocument> for RendererAsset {
    type Error = RendererDataDocumentError;

    fn try_from(document: RendererDataDocument) -> Result<Self, Self::Error> {
        document.to_renderer_asset()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererDataDocumentError {
    UnknownRenderPassStage { value: String },
    UnknownRenderFeatureSource { value: String },
    UnknownQualityGate { value: String },
}

impl Display for RendererDataDocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRenderPassStage { value } => {
                write!(f, "unknown renderer data render pass stage `{value}`")
            }
            Self::UnknownRenderFeatureSource { value } => {
                write!(f, "unknown renderer data feature source `{value}`")
            }
            Self::UnknownQualityGate { value } => {
                write!(f, "unknown renderer data quality gate `{value}`")
            }
        }
    }
}

impl std::error::Error for RendererDataDocumentError {}

fn parse_render_pass_stage(value: &str) -> Result<RenderPassStage, RendererDataDocumentError> {
    match value {
        "DepthPrepass" => Ok(RenderPassStage::DepthPrepass),
        "Shadow" => Ok(RenderPassStage::Shadow),
        "Deferred" => Ok(RenderPassStage::Deferred),
        "AmbientOcclusion" => Ok(RenderPassStage::AmbientOcclusion),
        "Lighting" => Ok(RenderPassStage::Lighting),
        "Opaque2d" => Ok(RenderPassStage::Opaque2d),
        "AlphaMask2d" => Ok(RenderPassStage::AlphaMask2d),
        "Transparent2d" => Ok(RenderPassStage::Transparent2d),
        "Opaque3d" => Ok(RenderPassStage::Opaque3d),
        "AlphaMask3d" => Ok(RenderPassStage::AlphaMask3d),
        "Transparent3d" => Ok(RenderPassStage::Transparent3d),
        "PostProcess" => Ok(RenderPassStage::PostProcess),
        "Ui" => Ok(RenderPassStage::Ui),
        "Overlay" => Ok(RenderPassStage::Overlay),
        "Debug" => Ok(RenderPassStage::Debug),
        _ => Err(RendererDataDocumentError::UnknownRenderPassStage {
            value: value.to_string(),
        }),
    }
}

fn parse_builtin_render_feature(value: &str) -> Result<BuiltinRenderFeature, String> {
    match value {
        "Mesh" => Ok(BuiltinRenderFeature::Mesh),
        "Sprite" => Ok(BuiltinRenderFeature::Sprite),
        "DeferredGeometry" => Ok(BuiltinRenderFeature::DeferredGeometry),
        "DeferredLighting" => Ok(BuiltinRenderFeature::DeferredLighting),
        "ClusteredLighting" => Ok(BuiltinRenderFeature::ClusteredLighting),
        "ScreenSpaceAmbientOcclusion" => Ok(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion),
        "Bloom" => Ok(BuiltinRenderFeature::Bloom),
        "ColorGrading" => Ok(BuiltinRenderFeature::ColorGrading),
        "ReflectionProbes" => Ok(BuiltinRenderFeature::ReflectionProbes),
        "BakedLighting" => Ok(BuiltinRenderFeature::BakedLighting),
        "HistoryResolve" => Ok(BuiltinRenderFeature::HistoryResolve),
        "AntiAlias" => Ok(BuiltinRenderFeature::AntiAlias),
        "Shadows" => Ok(BuiltinRenderFeature::Shadows),
        "PostProcess" => Ok(BuiltinRenderFeature::PostProcess),
        "Ui" => Ok(BuiltinRenderFeature::Ui),
        "DebugOverlay" => Ok(BuiltinRenderFeature::DebugOverlay),
        "Particle" => Ok(BuiltinRenderFeature::Particle),
        "GlobalIllumination" => Ok(BuiltinRenderFeature::GlobalIllumination),
        "RayTracing" => Ok(BuiltinRenderFeature::RayTracing),
        "VirtualGeometry" => Ok(BuiltinRenderFeature::VirtualGeometry),
        _ => Err(value.to_string()),
    }
}

const fn default_renderer_data_document_version() -> u32 {
    RENDERER_DATA_DOCUMENT_VERSION
}
