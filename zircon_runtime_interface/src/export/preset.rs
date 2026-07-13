use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project::{AssetRef, RelPath};
use crate::serialization::{
    load_versioned, Format, LoadError, MigrationChain, SchemaId, VersionedSchema,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTargetMode {
    ClientRuntime,
    ServerRuntime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFileMode {
    #[default]
    Default,
    Include,
    Exclude,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportCookCompression {
    None,
    #[default]
    Zstd,
    Lz4,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCookOptions {
    #[serde(default = "default_true")]
    pub deterministic: bool,
    #[serde(default = "default_true")]
    pub binary_assets: bool,
    #[serde(default)]
    pub compression: ExportCookCompression,
}

impl Default for ExportCookOptions {
    fn default() -> Self {
        Self {
            deterministic: true,
            binary_assets: true,
            compression: ExportCookCompression::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportPluginSubset {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportPreset {
    pub profile_ref: String,
    pub target_mode: ExportTargetMode,
    #[serde(default)]
    pub debug: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub include_filter: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub exclude_filter: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry_scenes: Vec<AssetRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep_list: Vec<AssetRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_subset: Option<ExportPluginSubset>,
    #[serde(default)]
    pub cook: ExportCookOptions,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub customized_files: BTreeMap<RelPath, ExportFileMode>,
}

impl ExportPreset {
    pub fn new(profile_ref: impl Into<String>, target_mode: ExportTargetMode) -> Self {
        Self {
            profile_ref: profile_ref.into(),
            target_mode,
            debug: false,
            include_filter: String::new(),
            exclude_filter: String::new(),
            entry_scenes: Vec::new(),
            keep_list: Vec::new(),
            plugin_subset: None,
            cook: ExportCookOptions::default(),
            customized_files: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ExportPresetValidationError> {
        if self.profile_ref.trim().is_empty() {
            return Err(ExportPresetValidationError::EmptyProfileRef);
        }
        if let Some(subset) = &self.plugin_subset {
            let mut package_ids = BTreeSet::new();
            for package_id in &subset.package_ids {
                if package_id.trim().is_empty() {
                    return Err(ExportPresetValidationError::EmptyPluginPackageId);
                }
                if !package_ids.insert(package_id) {
                    return Err(ExportPresetValidationError::DuplicatePluginPackageId {
                        package_id: package_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl VersionedSchema for ExportPreset {
    const SCHEMA: SchemaId = SchemaId::new("zircon.export-preset");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<ExportPreset> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportPresetValidationError {
    EmptyProfileRef,
    EmptyPluginPackageId,
    DuplicatePluginPackageId { package_id: String },
}

impl fmt::Display for ExportPresetValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfileRef => formatter.write_str("export preset profile_ref is empty"),
            Self::EmptyPluginPackageId => {
                formatter.write_str("export preset plugin subset contains an empty package id")
            }
            Self::DuplicatePluginPackageId { package_id } => write!(
                formatter,
                "export preset plugin subset contains duplicate package id `{package_id}`"
            ),
        }
    }
}

impl Error for ExportPresetValidationError {}

/// Decodes the public `.zpreset` wire contract.
///
/// Presets are a hard-cutover format: unlike the generic versioned loader, this
/// entry point never accepts an unwrapped version-zero payload.
pub fn load_export_preset(bytes: &[u8]) -> Result<ExportPreset, ExportPresetLoadError> {
    let document = serde_json::from_slice::<StrictPresetDocument>(bytes)
        .map_err(ExportPresetLoadError::Envelope)?;
    if document.envelope.header.schema_id != ExportPreset::SCHEMA.as_str() {
        return Err(ExportPresetLoadError::Schema {
            actual: document.envelope.header.schema_id,
        });
    }
    if document.envelope.header.schema_version != ExportPreset::VERSION {
        return Err(ExportPresetLoadError::Version {
            actual: document.envelope.header.schema_version,
        });
    }
    let loaded = load_versioned::<ExportPreset>(bytes, Format::Text)
        .map_err(ExportPresetLoadError::Payload)?;
    loaded
        .value
        .validate()
        .map_err(ExportPresetLoadError::Validation)?;
    Ok(loaded.value)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictPresetDocument {
    #[serde(rename = "$zircon")]
    envelope: StrictPresetEnvelope,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictPresetEnvelope {
    header: StrictPresetHeader,
    #[allow(dead_code)]
    payload: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictPresetHeader {
    schema_id: String,
    schema_version: u32,
}

#[derive(Debug)]
pub enum ExportPresetLoadError {
    Envelope(serde_json::Error),
    Schema { actual: String },
    Version { actual: u32 },
    Payload(LoadError),
    Validation(ExportPresetValidationError),
}

impl fmt::Display for ExportPresetLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope(source) => write!(formatter, "invalid export preset envelope: {source}"),
            Self::Schema { actual } => write!(
                formatter,
                "export preset schema_id must be `{}`, got `{actual}`",
                ExportPreset::SCHEMA.as_str()
            ),
            Self::Version { actual } => write!(
                formatter,
                "export preset schema_version must be {}, got {actual}",
                ExportPreset::VERSION
            ),
            Self::Payload(source) => write!(formatter, "invalid export preset payload: {source}"),
            Self::Validation(source) => write!(formatter, "invalid export preset: {source}"),
        }
    }
}

impl Error for ExportPresetLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope(source) => Some(source),
            Self::Payload(source) => Some(source),
            Self::Validation(source) => Some(source),
            Self::Schema { .. } | Self::Version { .. } => None,
        }
    }
}

const fn default_true() -> bool {
    true
}
