use std::collections::{BTreeMap, BTreeSet};
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
        self.validate_version()?;
        validate_renderer_data_name(&self.name)?;
        validate_non_empty_stage_list(self.stages.len())?;
        validate_non_empty_feature_list(self.features.len())?;
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
        validate_unique_stages(&stages)?;
        validate_unique_features(&features)?;

        Ok(RendererAsset {
            name: self.name.clone(),
            stages,
            features,
        })
    }

    fn validate_version(&self) -> Result<(), RendererDataDocumentError> {
        if self.version == RENDERER_DATA_DOCUMENT_VERSION {
            Ok(())
        } else {
            Err(RendererDataDocumentError::UnsupportedDocumentVersion {
                version: self.version,
                supported: RENDERER_DATA_DOCUMENT_VERSION,
            })
        }
    }

    pub fn from_renderer_asset(
        renderer: &RendererAsset,
    ) -> Result<Self, RendererDataDocumentError> {
        validate_renderer_data_name(&renderer.name)?;
        validate_non_empty_stage_list(renderer.stages.len())?;
        validate_non_empty_feature_list(renderer.features.len())?;
        let stages = renderer
            .stages
            .iter()
            .map(|stage| {
                if RenderPassStage::RENDERER_DATA_AUTHORING_STAGES.contains(stage) {
                    Ok(stage.authoring_name().to_string())
                } else {
                    Err(RendererDataDocumentError::UnsupportedRendererAssetStage { stage: *stage })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let features = renderer
            .features
            .iter()
            .map(RendererFeatureDocument::from_renderer_feature_asset)
            .collect::<Result<Vec<_>, _>>()?;
        validate_unique_stages(&renderer.stages)?;
        validate_unique_feature_names(&features)?;

        Ok(Self {
            version: RENDERER_DATA_DOCUMENT_VERSION,
            name: renderer.name.clone(),
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
        let source_name = source.authoring_name();
        if self.name.as_str() != source_name {
            return Err(RendererDataDocumentError::MismatchedRenderFeatureName {
                name: self.name.clone(),
                source: source_name.to_string(),
            });
        }
        validate_unique_reference_names(
            source_name,
            RendererFeatureReferenceListKind::RequiredEntryPoints,
            &self.required_entry_points,
        )?;
        validate_unique_reference_names(
            source_name,
            RendererFeatureReferenceListKind::ExpectedProperties,
            &self.expected_properties,
        )?;
        validate_unique_reference_names(
            source_name,
            RendererFeatureReferenceListKind::ExpectedTextureSlots,
            &self.expected_texture_slots,
        )?;
        validate_shader_reference_for_contract_lists(
            source_name,
            self.shader.as_ref(),
            &self.required_entry_points,
            &self.expected_properties,
            &self.expected_texture_slots,
        )?;
        validate_local_config_keys(source_name, &self.local_config)?;
        validate_quality_gate(source_name, self.quality_gate.as_deref())?;
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

    pub fn from_renderer_feature_asset(
        asset: &RendererFeatureAsset,
    ) -> Result<Self, RendererDataDocumentError> {
        let source = asset.builtin_feature().ok_or_else(|| {
            RendererDataDocumentError::UnsupportedRendererAssetFeatureSource {
                value: asset.feature_name(),
            }
        })?;
        let source_name = source.authoring_name().to_string();

        if asset.descriptor_override.is_some() {
            return Err(
                RendererDataDocumentError::UnsupportedRendererAssetDescriptorOverride {
                    feature: source_name,
                },
            );
        }
        if !asset.capability_requirements.is_empty() {
            return Err(
                RendererDataDocumentError::UnsupportedRendererAssetCapabilityRequirements {
                    feature: source_name,
                },
            );
        }
        validate_unique_reference_names(
            &source_name,
            RendererFeatureReferenceListKind::RequiredEntryPoints,
            &asset.asset_references.required_entry_points,
        )?;
        validate_unique_reference_names(
            &source_name,
            RendererFeatureReferenceListKind::ExpectedProperties,
            &asset.asset_references.expected_properties,
        )?;
        validate_unique_reference_names(
            &source_name,
            RendererFeatureReferenceListKind::ExpectedTextureSlots,
            &asset.asset_references.expected_texture_slots,
        )?;
        validate_shader_reference_for_contract_lists(
            &source_name,
            asset.asset_references.shader.as_ref(),
            &asset.asset_references.required_entry_points,
            &asset.asset_references.expected_properties,
            &asset.asset_references.expected_texture_slots,
        )?;
        validate_local_config_keys(&source_name, &asset.local_config)?;

        Ok(Self {
            name: source_name.clone(),
            source: source_name,
            enabled: asset.enabled,
            quality_gate: asset
                .quality_gate
                .map(|feature| feature.authoring_name().to_string()),
            shader: asset.asset_references.shader.clone(),
            material: asset.asset_references.material.clone(),
            required_entry_points: asset.asset_references.required_entry_points.clone(),
            expected_properties: asset.asset_references.expected_properties.clone(),
            expected_texture_slots: asset.asset_references.expected_texture_slots.clone(),
            local_config: asset.local_config.clone(),
        })
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

impl TryFrom<&RendererAsset> for RendererDataDocument {
    type Error = RendererDataDocumentError;

    fn try_from(renderer: &RendererAsset) -> Result<Self, Self::Error> {
        Self::from_renderer_asset(renderer)
    }
}

impl TryFrom<RendererAsset> for RendererDataDocument {
    type Error = RendererDataDocumentError;

    fn try_from(renderer: RendererAsset) -> Result<Self, Self::Error> {
        Self::from_renderer_asset(&renderer)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererDataDocumentError {
    UnsupportedDocumentVersion {
        version: u32,
        supported: u32,
    },
    EmptyRendererDataName,
    PaddedRendererDataName {
        name: String,
    },
    EmptyRenderPassStageList,
    EmptyRenderFeatureList,
    UnknownRenderPassStage {
        value: String,
    },
    UnknownRenderFeatureSource {
        value: String,
    },
    MismatchedRenderFeatureName {
        name: String,
        source: String,
    },
    DuplicateRenderPassStage {
        stage: RenderPassStage,
    },
    DuplicateRenderFeature {
        feature: String,
    },
    DuplicateRenderFeatureReference {
        feature: String,
        list: RendererFeatureReferenceListKind,
        value: String,
    },
    EmptyRenderFeatureReference {
        feature: String,
        list: RendererFeatureReferenceListKind,
    },
    PaddedRenderFeatureReference {
        feature: String,
        list: RendererFeatureReferenceListKind,
        value: String,
    },
    MissingRenderFeatureShaderReference {
        feature: String,
        list: RendererFeatureReferenceListKind,
    },
    EmptyRenderFeatureLocalConfigKey {
        feature: String,
    },
    PaddedRenderFeatureLocalConfigKey {
        feature: String,
        key: String,
    },
    EmptyRenderFeatureQualityGate {
        feature: String,
    },
    PaddedRenderFeatureQualityGate {
        feature: String,
        gate: String,
    },
    UnknownQualityGate {
        value: String,
    },
    UnsupportedRendererAssetStage {
        stage: RenderPassStage,
    },
    UnsupportedRendererAssetFeatureSource {
        value: String,
    },
    UnsupportedRendererAssetDescriptorOverride {
        feature: String,
    },
    UnsupportedRendererAssetCapabilityRequirements {
        feature: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererFeatureReferenceListKind {
    RequiredEntryPoints,
    ExpectedProperties,
    ExpectedTextureSlots,
}

impl RendererFeatureReferenceListKind {
    fn authoring_name(self) -> &'static str {
        match self {
            Self::RequiredEntryPoints => "required_entry_points",
            Self::ExpectedProperties => "expected_properties",
            Self::ExpectedTextureSlots => "expected_texture_slots",
        }
    }
}

impl Display for RendererDataDocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDocumentVersion { version, supported } => write!(
                f,
                "renderer data document version {version} is not supported; expected version {supported}"
            ),
            Self::EmptyRendererDataName => {
                write!(f, "renderer data document name cannot be empty")
            }
            Self::PaddedRendererDataName { name } => write!(
                f,
                "renderer data document name `{name}` has leading or trailing whitespace"
            ),
            Self::EmptyRenderPassStageList => write!(
                f,
                "renderer data document must declare at least one render pass stage"
            ),
            Self::EmptyRenderFeatureList => write!(
                f,
                "renderer data document must declare at least one render feature"
            ),
            Self::UnknownRenderPassStage { value } => {
                write!(f, "unknown renderer data render pass stage `{value}`")
            }
            Self::UnknownRenderFeatureSource { value } => {
                write!(f, "unknown renderer data feature source `{value}`")
            }
            Self::MismatchedRenderFeatureName { name, source } => write!(
                f,
                "renderer data feature name `{name}` must match canonical source `{source}`"
            ),
            Self::DuplicateRenderPassStage { stage } => write!(
                f,
                "renderer data document declares duplicate stage `{}`",
                stage.authoring_name()
            ),
            Self::DuplicateRenderFeature { feature } => write!(
                f,
                "renderer data document declares duplicate feature `{feature}`"
            ),
            Self::DuplicateRenderFeatureReference {
                feature,
                list,
                value,
            } => write!(
                f,
                "renderer data feature `{feature}` declares duplicate `{}` reference `{value}`",
                list.authoring_name()
            ),
            Self::EmptyRenderFeatureReference { feature, list } => write!(
                f,
                "renderer data feature `{feature}` declares empty `{}` reference",
                list.authoring_name()
            ),
            Self::PaddedRenderFeatureReference {
                feature,
                list,
                value,
            } => write!(
                f,
                "renderer data feature `{feature}` declares `{}` reference `{value}` with leading or trailing whitespace",
                list.authoring_name()
            ),
            Self::MissingRenderFeatureShaderReference { feature, list } => write!(
                f,
                "renderer data feature `{feature}` declares `{}` references without a shader reference",
                list.authoring_name()
            ),
            Self::EmptyRenderFeatureLocalConfigKey { feature } => write!(
                f,
                "renderer data feature `{feature}` declares empty local config key"
            ),
            Self::PaddedRenderFeatureLocalConfigKey { feature, key } => write!(
                f,
                "renderer data feature `{feature}` declares local config key `{key}` with leading or trailing whitespace"
            ),
            Self::EmptyRenderFeatureQualityGate { feature } => write!(
                f,
                "renderer data feature `{feature}` declares empty quality gate"
            ),
            Self::PaddedRenderFeatureQualityGate { feature, gate } => write!(
                f,
                "renderer data feature `{feature}` declares quality gate `{gate}` with leading or trailing whitespace"
            ),
            Self::UnknownQualityGate { value } => {
                write!(f, "unknown renderer data quality gate `{value}`")
            }
            Self::UnsupportedRendererAssetStage { stage } => write!(
                f,
                "renderer asset stage `{}` cannot be written to renderer data",
                stage.authoring_name()
            ),
            Self::UnsupportedRendererAssetFeatureSource { value } => write!(
                f,
                "renderer asset feature source `{value}` cannot be written to renderer data"
            ),
            Self::UnsupportedRendererAssetDescriptorOverride { feature } => write!(
                f,
                "renderer asset feature `{feature}` descriptor override cannot be written to renderer data"
            ),
            Self::UnsupportedRendererAssetCapabilityRequirements { feature } => write!(
                f,
                "renderer asset feature `{feature}` capability requirements cannot be written to renderer data"
            ),
        }
    }
}

impl std::error::Error for RendererDataDocumentError {}

fn parse_render_pass_stage(value: &str) -> Result<RenderPassStage, RendererDataDocumentError> {
    RenderPassStage::from_renderer_data_authoring_name(value).ok_or_else(|| {
        RendererDataDocumentError::UnknownRenderPassStage {
            value: value.to_string(),
        }
    })
}

fn parse_builtin_render_feature(value: &str) -> Result<BuiltinRenderFeature, String> {
    BuiltinRenderFeature::from_authoring_name(value).ok_or_else(|| value.to_string())
}

fn validate_renderer_data_name(name: &str) -> Result<(), RendererDataDocumentError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RendererDataDocumentError::EmptyRendererDataName);
    }
    if trimmed != name {
        return Err(RendererDataDocumentError::PaddedRendererDataName {
            name: name.to_string(),
        });
    }

    Ok(())
}

fn validate_non_empty_stage_list(stage_count: usize) -> Result<(), RendererDataDocumentError> {
    if stage_count == 0 {
        Err(RendererDataDocumentError::EmptyRenderPassStageList)
    } else {
        Ok(())
    }
}

fn validate_non_empty_feature_list(feature_count: usize) -> Result<(), RendererDataDocumentError> {
    if feature_count == 0 {
        Err(RendererDataDocumentError::EmptyRenderFeatureList)
    } else {
        Ok(())
    }
}

fn validate_unique_stages(stages: &[RenderPassStage]) -> Result<(), RendererDataDocumentError> {
    let mut seen_stages = BTreeSet::new();
    for stage in stages {
        if !seen_stages.insert(*stage) {
            return Err(RendererDataDocumentError::DuplicateRenderPassStage { stage: *stage });
        }
    }

    Ok(())
}

fn validate_unique_features(
    features: &[RendererFeatureAsset],
) -> Result<(), RendererDataDocumentError> {
    let mut seen_features = BTreeSet::new();
    for feature in features {
        let feature_name = feature.feature_name();
        if !seen_features.insert(feature_name.clone()) {
            return Err(RendererDataDocumentError::DuplicateRenderFeature {
                feature: feature_name,
            });
        }
    }

    Ok(())
}

fn validate_unique_feature_names(
    features: &[RendererFeatureDocument],
) -> Result<(), RendererDataDocumentError> {
    let mut seen_features = BTreeSet::new();
    for feature in features {
        if !seen_features.insert(feature.name.clone()) {
            return Err(RendererDataDocumentError::DuplicateRenderFeature {
                feature: feature.name.clone(),
            });
        }
    }

    Ok(())
}

fn validate_unique_reference_names(
    feature: &str,
    list: RendererFeatureReferenceListKind,
    values: &[String],
) -> Result<(), RendererDataDocumentError> {
    let mut seen_values = BTreeSet::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(RendererDataDocumentError::EmptyRenderFeatureReference {
                feature: feature.to_string(),
                list,
            });
        }
        if trimmed != value.as_str() {
            return Err(RendererDataDocumentError::PaddedRenderFeatureReference {
                feature: feature.to_string(),
                list,
                value: value.clone(),
            });
        }
        if !seen_values.insert(value.as_str()) {
            return Err(RendererDataDocumentError::DuplicateRenderFeatureReference {
                feature: feature.to_string(),
                list,
                value: value.clone(),
            });
        }
    }

    Ok(())
}

fn validate_shader_reference_for_contract_lists(
    feature: &str,
    shader: Option<&AssetReference>,
    required_entry_points: &[String],
    expected_properties: &[String],
    expected_texture_slots: &[String],
) -> Result<(), RendererDataDocumentError> {
    if shader.is_some() {
        return Ok(());
    }
    if !required_entry_points.is_empty() {
        return Err(
            RendererDataDocumentError::MissingRenderFeatureShaderReference {
                feature: feature.to_string(),
                list: RendererFeatureReferenceListKind::RequiredEntryPoints,
            },
        );
    }
    if !expected_properties.is_empty() {
        return Err(
            RendererDataDocumentError::MissingRenderFeatureShaderReference {
                feature: feature.to_string(),
                list: RendererFeatureReferenceListKind::ExpectedProperties,
            },
        );
    }
    if !expected_texture_slots.is_empty() {
        return Err(
            RendererDataDocumentError::MissingRenderFeatureShaderReference {
                feature: feature.to_string(),
                list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
            },
        );
    }

    Ok(())
}

fn validate_local_config_keys(
    feature: &str,
    local_config: &BTreeMap<String, String>,
) -> Result<(), RendererDataDocumentError> {
    for key in local_config.keys() {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(
                RendererDataDocumentError::EmptyRenderFeatureLocalConfigKey {
                    feature: feature.to_string(),
                },
            );
        }
        if trimmed != key.as_str() {
            return Err(
                RendererDataDocumentError::PaddedRenderFeatureLocalConfigKey {
                    feature: feature.to_string(),
                    key: key.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_quality_gate(
    feature: &str,
    quality_gate: Option<&str>,
) -> Result<(), RendererDataDocumentError> {
    let Some(gate) = quality_gate else {
        return Ok(());
    };

    let trimmed = gate.trim();
    if trimmed.is_empty() {
        return Err(RendererDataDocumentError::EmptyRenderFeatureQualityGate {
            feature: feature.to_string(),
        });
    }
    if trimmed != gate {
        return Err(RendererDataDocumentError::PaddedRenderFeatureQualityGate {
            feature: feature.to_string(),
            gate: gate.to_string(),
        });
    }

    Ok(())
}

const fn default_renderer_data_document_version() -> u32 {
    RENDERER_DATA_DOCUMENT_VERSION
}
