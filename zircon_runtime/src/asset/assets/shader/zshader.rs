use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    MaterialPropertyKind, ShaderAssetKind, ShaderQueueDescriptor, ShaderRenderStateDescriptor,
    ShaderResourceDescriptor,
};

pub type ZShaderV2Result<T> = std::result::Result<T, ZShaderV2Error>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZShaderV2Error {
    #[error("zshader v2 document is not a TOML table")]
    RootTable,
    #[error("zshader v2 document is missing required field `{field}`")]
    MissingDocumentField { field: String },
    #[error("zshader v2 document has unsupported kind `{kind}`")]
    UnsupportedKind { kind: String },
    #[error("zshader v2 {kind} document is missing required field `{field}`")]
    MissingRequiredField { kind: String, field: String },
    #[error("zshader v2 {kind} document forbids field `{field}`")]
    ForbiddenField { kind: String, field: String },
    #[error("zshader v2 {kind} document field `{field}` must not be empty")]
    EmptyField { kind: String, field: String },
    #[error("zshader v2 document version `{version}` is unsupported")]
    UnsupportedVersion { version: u32 },
    #[error(
        "zshader v2 {kind} entry point `{entry}` uses stage `{stage}` but expected `{expected}`"
    )]
    InvalidEntryStage {
        kind: String,
        entry: String,
        stage: String,
        expected: String,
    },
    #[error("zshader v2 TOML parse failed: {message}")]
    Toml { message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZShaderDocumentV2 {
    Surface(ZShaderSurfaceDocumentV2),
    Include(ZShaderIncludeDocumentV2),
    Compute(ZShaderComputeDocumentV2),
    Fullscreen(ZShaderFullscreenDocumentV2),
}

impl ZShaderDocumentV2 {
    pub fn from_toml_str(document: &str) -> ZShaderV2Result<Self> {
        let table = zshader_v2_table(document)?;
        let kind = zshader_v2_kind(&table)?;
        validate_zshader_v2_version(&table)?;
        match kind {
            ShaderAssetKind::Surface => {
                validate_zshader_v2_keys(&table, kind, SURFACE_V2_FIELDS, SURFACE_V2_REQUIRED)?;
                validate_required_string(&table, kind, "shading_model")?;
                validate_optional_string(&table, kind, "import_path")?;
                Ok(Self::Surface(deserialize_zshader_v2(table)?))
            }
            ShaderAssetKind::Include => {
                validate_zshader_v2_keys(&table, kind, INCLUDE_V2_FIELDS, INCLUDE_V2_REQUIRED)?;
                validate_optional_string(&table, kind, "import_path")?;
                Ok(Self::Include(deserialize_zshader_v2(table)?))
            }
            ShaderAssetKind::Compute => {
                validate_zshader_v2_keys(&table, kind, COMPUTE_V2_FIELDS, COMPUTE_V2_REQUIRED)?;
                validate_non_empty_array(&table, kind, "entry_points")?;
                let document: ZShaderComputeDocumentV2 = deserialize_zshader_v2(table)?;
                validate_entry_point_stages(
                    kind,
                    &document.entry_points,
                    "compute",
                    stage_is_compute,
                )?;
                Ok(Self::Compute(document))
            }
            ShaderAssetKind::Fullscreen => {
                validate_zshader_v2_keys(&table, kind, FULLSCREEN_V2_FIELDS, &[])?;
                let document: ZShaderFullscreenDocumentV2 = deserialize_zshader_v2(table)?;
                validate_entry_point_stages(
                    kind,
                    &document.entry_points,
                    "fragment",
                    stage_is_fragment,
                )?;
                Ok(Self::Fullscreen(document))
            }
        }
    }

    pub const fn kind(&self) -> ShaderAssetKind {
        match self {
            Self::Surface(_) => ShaderAssetKind::Surface,
            Self::Include(_) => ShaderAssetKind::Include,
            Self::Compute(_) => ShaderAssetKind::Compute,
            Self::Fullscreen(_) => ShaderAssetKind::Fullscreen,
        }
    }

    pub fn import_path(&self) -> Option<&str> {
        match self {
            Self::Surface(document) => document.import_path.as_deref(),
            Self::Compute(_) | Self::Fullscreen(_) => None,
            Self::Include(document) => document.import_path.as_deref(),
        }
    }

    pub fn wgsl_files(&self) -> &[String] {
        match self {
            Self::Surface(document) => &document.wgsl_files,
            Self::Include(document) => &document.wgsl_files,
            Self::Compute(document) => &document.wgsl_files,
            Self::Fullscreen(document) => &document.wgsl_files,
        }
    }

    pub fn entry_points(&self) -> &[ZShaderEntryPointDocument] {
        match self {
            Self::Surface(_) | Self::Include(_) => &[],
            Self::Compute(document) => &document.entry_points,
            Self::Fullscreen(document) => &document.entry_points,
        }
    }

    pub fn imports(&self) -> &[ZShaderImportDocument] {
        match self {
            Self::Surface(document) => &document.imports,
            Self::Include(document) => &document.imports,
            Self::Compute(document) => &document.imports,
            Self::Fullscreen(document) => &document.imports,
        }
    }

    pub fn properties(&self) -> &[ShaderMaterialPropertyAsset] {
        match self {
            Self::Surface(document) => &document.properties,
            Self::Include(_) => &[],
            Self::Compute(document) => &document.properties,
            Self::Fullscreen(document) => &document.properties,
        }
    }

    pub fn options(&self) -> &[ZShaderOptionDocument] {
        match self {
            Self::Surface(document) => &document.options,
            Self::Include(_) => &[],
            Self::Compute(document) => &document.options,
            Self::Fullscreen(document) => &document.options,
        }
    }

    pub fn texture_slots(&self) -> &[ZShaderTextureSlotDocument] {
        match self {
            Self::Surface(document) => &document.texture_slots,
            Self::Include(_) | Self::Compute(_) | Self::Fullscreen(_) => &[],
        }
    }

    pub fn resources(&self) -> &[ShaderResourceDescriptor] {
        match self {
            Self::Surface(_) | Self::Include(_) => &[],
            Self::Compute(document) => &document.resources,
            Self::Fullscreen(document) => &document.resources,
        }
    }

    pub fn render_state(&self) -> ShaderRenderStateDescriptor {
        match self {
            Self::Surface(document) => document.render_state.clone(),
            Self::Fullscreen(document) => document.render_state.clone(),
            Self::Include(_) | Self::Compute(_) => ShaderRenderStateDescriptor::default(),
        }
    }

    pub fn queue(&self) -> Option<ShaderQueueDescriptor> {
        match self {
            Self::Surface(document) => document.queue,
            Self::Include(_) | Self::Compute(_) | Self::Fullscreen(_) => None,
        }
    }

    pub fn disabled_passes(&self) -> &[String] {
        match self {
            Self::Surface(document) => &document.disabled_passes,
            Self::Include(_) | Self::Compute(_) | Self::Fullscreen(_) => &[],
        }
    }

    pub fn shading_model(&self) -> Option<&str> {
        match self {
            Self::Surface(document) => Some(document.shading_model.as_str()),
            Self::Include(_) | Self::Compute(_) | Self::Fullscreen(_) => None,
        }
    }

    pub fn editor(&self) -> &toml::Table {
        match self {
            Self::Surface(document) => &document.editor,
            Self::Include(document) => &document.editor,
            Self::Compute(document) => &document.editor,
            Self::Fullscreen(document) => &document.editor,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderSurfaceDocumentV2 {
    pub kind: ShaderAssetKind,
    #[serde(default = "default_zshader_v2_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    pub shading_model: String,
    #[serde(default)]
    pub properties: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub options: Vec<ZShaderOptionDocument>,
    #[serde(default)]
    pub texture_slots: Vec<ZShaderTextureSlotDocument>,
    #[serde(default)]
    pub render_state: ShaderRenderStateDescriptor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<ShaderQueueDescriptor>,
    #[serde(default)]
    pub disabled_passes: Vec<String>,
    #[serde(default)]
    pub editor: toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderIncludeDocumentV2 {
    pub kind: ShaderAssetKind,
    #[serde(default = "default_zshader_v2_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    #[serde(default)]
    pub editor: toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderComputeDocumentV2 {
    pub kind: ShaderAssetKind,
    #[serde(default = "default_zshader_v2_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default)]
    pub entry_points: Vec<ZShaderEntryPointDocument>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    #[serde(default)]
    pub properties: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub options: Vec<ZShaderOptionDocument>,
    #[serde(default)]
    pub resources: Vec<ShaderResourceDescriptor>,
    #[serde(default)]
    pub editor: toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderFullscreenDocumentV2 {
    pub kind: ShaderAssetKind,
    #[serde(default = "default_zshader_v2_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default)]
    pub entry_points: Vec<ZShaderEntryPointDocument>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    #[serde(default)]
    pub properties: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub options: Vec<ZShaderOptionDocument>,
    #[serde(default)]
    pub resources: Vec<ShaderResourceDescriptor>,
    #[serde(default)]
    pub render_state: ShaderRenderStateDescriptor,
    #[serde(default)]
    pub editor: toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZShaderOptionDocument {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderEntryPointDocument {
    pub name: String,
    pub stage: String,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderImportDocument {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<AssetReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderSourceFileAsset {
    pub path: String,
    pub url: AssetUri,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderImportRedirectAsset {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<AssetReference>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderMaterialPropertyAsset {
    pub name: String,
    pub kind: MaterialPropertyKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderOptionAsset {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderTextureSlotDocument {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    #[serde(default)]
    pub st: bool,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderTextureSlotAsset {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    #[serde(default)]
    pub st: bool,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

impl From<&ZShaderTextureSlotDocument> for ShaderTextureSlotAsset {
    fn from(slot: &ZShaderTextureSlotDocument) -> Self {
        Self {
            name: slot.name.clone(),
            kind: slot.kind.clone(),
            required: slot.required,
            default: slot.default.clone(),
            sampler: slot.sampler.clone(),
            group: slot.group.clone(),
            label: slot.label.clone(),
            option: slot.option.clone(),
            st: slot.st,
            editor: slot.editor.clone(),
        }
    }
}

impl From<&ZShaderOptionDocument> for ShaderOptionAsset {
    fn from(option: &ZShaderOptionDocument) -> Self {
        Self {
            name: option.name.clone(),
            kind: option.kind.clone(),
            default: option.default.clone(),
            editor: option.editor.clone(),
        }
    }
}

impl ShaderMaterialPropertyAsset {
    pub fn accepts_value(&self, value: &toml::Value) -> bool {
        match self.kind {
            MaterialPropertyKind::Bool => value.as_bool().is_some(),
            MaterialPropertyKind::Float => {
                value.as_float().is_some() || value.as_integer().is_some()
            }
            MaterialPropertyKind::Int => value
                .as_integer()
                .and_then(|value| i32::try_from(value).ok())
                .is_some(),
            MaterialPropertyKind::UInt => value
                .as_integer()
                .and_then(|value| u32::try_from(value).ok())
                .is_some(),
            MaterialPropertyKind::Color | MaterialPropertyKind::Vec4 => numeric_array_len(value, 4),
            MaterialPropertyKind::Vec3 => numeric_array_len(value, 3),
            MaterialPropertyKind::Vec2 => numeric_array_len(value, 2),
        }
    }
}

fn numeric_array_len(value: &toml::Value, len: usize) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == len
            && items
                .iter()
                .all(|item| item.as_float().is_some() || item.as_integer().is_some())
    })
}

const SURFACE_V2_FIELDS: &[&str] = &[
    "kind",
    "version",
    "name",
    "wgsl_files",
    "import_path",
    "imports",
    "shading_model",
    "properties",
    "options",
    "texture_slots",
    "render_state",
    "queue",
    "disabled_passes",
    "editor",
];
const SURFACE_V2_REQUIRED: &[&str] = &["kind", "shading_model"];
const INCLUDE_V2_FIELDS: &[&str] = &[
    "kind",
    "version",
    "name",
    "import_path",
    "wgsl_files",
    "imports",
    "editor",
];
const INCLUDE_V2_REQUIRED: &[&str] = &["kind"];
const COMPUTE_V2_FIELDS: &[&str] = &[
    "kind",
    "version",
    "name",
    "wgsl_files",
    "entry_points",
    "imports",
    "properties",
    "options",
    "resources",
    "editor",
];
const COMPUTE_V2_REQUIRED: &[&str] = &["kind", "entry_points"];
const FULLSCREEN_V2_FIELDS: &[&str] = &[
    "kind",
    "version",
    "name",
    "wgsl_files",
    "entry_points",
    "imports",
    "properties",
    "options",
    "resources",
    "render_state",
    "editor",
];

fn zshader_v2_table(document: &str) -> ZShaderV2Result<toml::Table> {
    let value = toml::from_str::<toml::Value>(document).map_err(|error| ZShaderV2Error::Toml {
        message: error.to_string(),
    })?;
    match value {
        toml::Value::Table(table) => Ok(table),
        _ => Err(ZShaderV2Error::RootTable),
    }
}

fn zshader_v2_kind(table: &toml::Table) -> ZShaderV2Result<ShaderAssetKind> {
    let kind = table
        .get("kind")
        .ok_or_else(|| ZShaderV2Error::MissingDocumentField {
            field: "kind".to_string(),
        })?
        .as_str()
        .ok_or_else(|| ZShaderV2Error::Toml {
            message: "zshader v2 `kind` must be a string".to_string(),
        })?
        .trim()
        .to_ascii_lowercase();
    match kind.as_str() {
        "surface" => Ok(ShaderAssetKind::Surface),
        "include" => Ok(ShaderAssetKind::Include),
        "compute" => Ok(ShaderAssetKind::Compute),
        "fullscreen" => Ok(ShaderAssetKind::Fullscreen),
        _ => Err(ZShaderV2Error::UnsupportedKind { kind }),
    }
}

fn validate_zshader_v2_version(table: &toml::Table) -> ZShaderV2Result<()> {
    let Some(version) = table.get("version") else {
        return Ok(());
    };
    let Some(version) = version
        .as_integer()
        .and_then(|value| u32::try_from(value).ok())
    else {
        return Err(ZShaderV2Error::Toml {
            message: "zshader v2 `version` must be a positive integer".to_string(),
        });
    };
    if version == default_zshader_v2_version() {
        Ok(())
    } else {
        Err(ZShaderV2Error::UnsupportedVersion { version })
    }
}

fn validate_zshader_v2_keys(
    table: &toml::Table,
    kind: ShaderAssetKind,
    allowed: &[&str],
    required: &[&str],
) -> ZShaderV2Result<()> {
    let kind = kind.token();
    for field in table.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(ZShaderV2Error::ForbiddenField {
                kind: kind.to_string(),
                field: field.clone(),
            });
        }
    }
    for field in required {
        if !table.contains_key(*field) {
            return Err(ZShaderV2Error::MissingRequiredField {
                kind: kind.to_string(),
                field: (*field).to_string(),
            });
        }
    }
    Ok(())
}

fn validate_required_string(
    table: &toml::Table,
    kind: ShaderAssetKind,
    field: &str,
) -> ZShaderV2Result<()> {
    let value = table
        .get(field)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .trim();
    if value.is_empty() {
        Err(ZShaderV2Error::EmptyField {
            kind: kind.token().to_string(),
            field: field.to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_optional_string(
    table: &toml::Table,
    kind: ShaderAssetKind,
    field: &str,
) -> ZShaderV2Result<()> {
    if table.contains_key(field) {
        validate_required_string(table, kind, field)
    } else {
        Ok(())
    }
}

fn validate_non_empty_array(
    table: &toml::Table,
    kind: ShaderAssetKind,
    field: &str,
) -> ZShaderV2Result<()> {
    let is_non_empty = table
        .get(field)
        .and_then(toml::Value::as_array)
        .is_some_and(|items| !items.is_empty());
    if is_non_empty {
        Ok(())
    } else {
        Err(ZShaderV2Error::EmptyField {
            kind: kind.token().to_string(),
            field: field.to_string(),
        })
    }
}

fn deserialize_zshader_v2<T>(table: toml::Table) -> ZShaderV2Result<T>
where
    T: DeserializeOwned,
{
    toml::Value::Table(table)
        .try_into()
        .map_err(|error| ZShaderV2Error::Toml {
            message: error.to_string(),
        })
}

fn validate_entry_point_stages(
    kind: ShaderAssetKind,
    entry_points: &[ZShaderEntryPointDocument],
    expected: &str,
    predicate: fn(&str) -> bool,
) -> ZShaderV2Result<()> {
    for entry_point in entry_points {
        if !predicate(&entry_point.stage) {
            return Err(ZShaderV2Error::InvalidEntryStage {
                kind: kind.token().to_string(),
                entry: entry_point.name.clone(),
                stage: entry_point.stage.clone(),
                expected: expected.to_string(),
            });
        }
    }
    Ok(())
}

fn stage_is_compute(stage: &str) -> bool {
    matches!(
        stage.trim().to_ascii_lowercase().as_str(),
        "compute" | "comp" | "cs"
    )
}

fn stage_is_fragment(stage: &str) -> bool {
    matches!(
        stage.trim().to_ascii_lowercase().as_str(),
        "fragment" | "frag" | "fs"
    )
}

fn default_zshader_v2_version() -> u32 {
    2
}
