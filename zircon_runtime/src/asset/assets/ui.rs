use std::collections::{BTreeMap, HashSet};

use crate::core::resource::{AssetReference, ResourceLocator, ResourceLocatorError};
use crate::ui::template::{collect_document_resource_dependencies, UiAssetLoader};
use crate::ui::v2::{UiV2AssetLoader, UiZuiAssetLoader};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::ui::style::UiThemeDocument;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiAssetKind};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetError, UiV2AssetKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiLayoutAsset {
    pub document: UiAssetDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiWidgetAsset {
    pub document: UiAssetDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiStyleAsset {
    pub document: UiAssetDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiThemeAsset {
    pub document: UiThemeDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiIconAsset {
    pub source: UiIconSource,
    #[serde(default = "default_ui_icon_size")]
    pub default_size: f32,
    #[serde(default)]
    pub semantic_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiIconSource {
    pub kind: UiIconSourceKind,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiIconSourceKind {
    Svg,
    SvgAsset,
    Bitmap,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiV2ViewAsset {
    pub document: UiV2AssetDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiV2ComponentAsset {
    pub document: UiV2AssetDocument,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiV2StyleAsset {
    pub document: UiV2AssetDocument,
}

pub type UiAssetDocumentResult<T> = std::result::Result<T, UiAssetDocumentError>;
pub type UiThemeAssetDocumentResult<T> = std::result::Result<T, UiThemeAssetDocumentError>;
pub type UiIconAssetDocumentResult<T> = std::result::Result<T, UiIconAssetDocumentError>;
pub type UiV2AssetDocumentResult<T> = std::result::Result<T, UiV2AssetDocumentError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiAssetDocumentError {
    #[error("failed to parse ui asset document: {0}")]
    Parse(#[from] UiAssetError),
    #[error("expected ui asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
}

#[derive(Debug, Error)]
pub enum UiThemeAssetDocumentError {
    #[error("failed to parse ui theme asset document: {0}")]
    Parse(#[source] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum UiIconAssetDocumentError {
    #[error("failed to parse ui icon asset document: {0}")]
    Parse(#[source] toml::de::Error),
    #[error("ui icon default_size must be a positive finite value")]
    InvalidDefaultSize,
    #[error("ui icon semantic_id must not be empty")]
    EmptySemanticId,
    #[error("inline svg source must not be empty")]
    EmptyInlineSvgSource,
    #[error("external icon source uri must not be empty")]
    EmptyExternalSourceUri,
    #[error("source uri `{uri}` is not a valid resource locator: {source}")]
    InvalidSourceUri {
        uri: String,
        #[source]
        source: ResourceLocatorError,
    },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiV2AssetDocumentError {
    #[error("failed to parse ui v2 asset document: {0}")]
    Parse(#[from] UiV2AssetError),
    #[error("expected ui v2 asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiV2AssetKind,
        actual: UiV2AssetKind,
    },
    #[error("ui v2 component documents must use `.zui`, not `.v2.ui.toml`")]
    ComponentRequiresZui,
}

impl UiLayoutAsset {
    pub fn from_toml_str(document: &str) -> UiAssetDocumentResult<Self> {
        parse_typed(document, UiAssetKind::Layout).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiWidgetAsset {
    pub fn from_toml_str(document: &str) -> UiAssetDocumentResult<Self> {
        parse_typed(document, UiAssetKind::Widget).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiStyleAsset {
    pub fn from_toml_str(document: &str) -> UiAssetDocumentResult<Self> {
        parse_typed(document, UiAssetKind::Style).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiThemeAsset {
    pub fn from_toml_str(document: &str) -> UiThemeAssetDocumentResult<Self> {
        toml::from_str::<UiThemeDocument>(document)
            .map(|document| Self { document })
            .map_err(UiThemeAssetDocumentError::Parse)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiIconAsset {
    pub fn from_toml_str(document: &str) -> UiIconAssetDocumentResult<Self> {
        let asset = toml::from_str::<Self>(document).map_err(UiIconAssetDocumentError::Parse)?;
        asset.validate()?;
        Ok(asset)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::new();
        let mut seen = HashSet::new();
        match self.source.kind {
            UiIconSourceKind::Svg => {}
            UiIconSourceKind::SvgAsset | UiIconSourceKind::Bitmap => {
                let Some(uri) = self.source.uri.as_deref() else {
                    return references;
                };
                push_reference(uri, &mut references, &mut seen);
            }
        }
        references
    }

    fn validate(&self) -> UiIconAssetDocumentResult<()> {
        if !self.default_size.is_finite() || self.default_size <= 0.0 {
            return Err(UiIconAssetDocumentError::InvalidDefaultSize);
        }
        if self.semantic_id.trim().is_empty() {
            return Err(UiIconAssetDocumentError::EmptySemanticId);
        }
        match self.source.kind {
            UiIconSourceKind::Svg => match self.source.text.as_deref() {
                Some(text) if !text.trim().is_empty() => Ok(()),
                _ => Err(UiIconAssetDocumentError::EmptyInlineSvgSource),
            },
            UiIconSourceKind::SvgAsset | UiIconSourceKind::Bitmap => {
                let Some(uri) = self.source.uri.as_deref() else {
                    return Err(UiIconAssetDocumentError::EmptyExternalSourceUri);
                };
                ResourceLocator::parse(uri).map_err(|source| {
                    UiIconAssetDocumentError::InvalidSourceUri {
                        uri: uri.to_string(),
                        source,
                    }
                })?;
                Ok(())
            }
        }
    }
}

impl UiV2ViewAsset {
    pub fn from_toml_str(document: &str) -> UiV2AssetDocumentResult<Self> {
        parse_v2_typed(document, UiV2AssetKind::View).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiV2ComponentAsset {
    pub fn from_toml_str(document: &str) -> UiV2AssetDocumentResult<Self> {
        parse_v2_typed(document, UiV2AssetKind::Component).map(|document| Self { document })
    }

    pub fn from_zui_str(document: &str) -> UiV2AssetDocumentResult<Self> {
        let document =
            UiZuiAssetLoader::load_zui_str(document).map_err(UiV2AssetDocumentError::Parse)?;
        if document.asset.kind != UiV2AssetKind::Component {
            return Err(UiV2AssetDocumentError::UnexpectedKind {
                expected: UiV2AssetKind::Component,
                actual: document.asset.kind,
            });
        }
        Ok(Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiV2StyleAsset {
    pub fn from_toml_str(document: &str) -> UiV2AssetDocumentResult<Self> {
        parse_v2_typed(document, UiV2AssetKind::Style).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

pub fn ui_asset_references(document: &UiAssetDocument) -> Vec<AssetReference> {
    let mut references = Vec::new();
    let mut seen = HashSet::new();
    for reference in document
        .imports
        .widgets
        .iter()
        .chain(document.imports.styles.iter())
    {
        push_reference(reference, &mut references, &mut seen);
    }

    if let Ok(report) =
        collect_document_resource_dependencies(document, &BTreeMap::new(), &BTreeMap::new())
    {
        for dependency in report.dependencies {
            push_reference(&dependency.reference.uri, &mut references, &mut seen);
            if let Some(fallback_uri) = dependency.reference.fallback.uri.as_deref() {
                push_reference(fallback_uri, &mut references, &mut seen);
            }
        }
    }
    references
}

pub fn ui_v2_asset_references(document: &UiV2AssetDocument) -> Vec<AssetReference> {
    let mut references = Vec::new();
    let mut seen = HashSet::new();
    for reference in document
        .imports
        .widgets
        .iter()
        .chain(document.imports.styles.iter())
    {
        push_reference(reference, &mut references, &mut seen);
    }
    for reference in &document.imports.resources {
        push_reference(&reference.uri, &mut references, &mut seen);
        if let Some(fallback_uri) = reference.fallback.uri.as_deref() {
            push_reference(fallback_uri, &mut references, &mut seen);
        }
    }
    references
}

const fn default_ui_icon_size() -> f32 {
    16.0
}

fn push_reference(
    uri: &str,
    references: &mut Vec<AssetReference>,
    seen: &mut HashSet<ResourceLocator>,
) {
    let Ok(locator) = ResourceLocator::parse(uri) else {
        return;
    };
    let Ok(asset_locator) =
        ResourceLocator::new(locator.scheme(), locator.path().to_string(), None)
    else {
        return;
    };
    if seen.insert(asset_locator.clone()) {
        references.push(AssetReference::from_locator(asset_locator));
    }
}

fn parse_typed(document: &str, expected: UiAssetKind) -> UiAssetDocumentResult<UiAssetDocument> {
    let parsed = UiAssetLoader::load_toml_str(document)?;
    if parsed.asset.kind != expected {
        return Err(UiAssetDocumentError::UnexpectedKind {
            expected,
            actual: parsed.asset.kind,
        });
    }
    Ok(parsed)
}

fn parse_v2_typed(
    document: &str,
    expected: UiV2AssetKind,
) -> UiV2AssetDocumentResult<UiV2AssetDocument> {
    let parsed = UiV2AssetLoader::load_toml_str(document)?;
    let matches_style_import =
        expected == UiV2AssetKind::Style && parsed.asset.kind == UiV2AssetKind::ThemeTokens;
    if parsed.asset.kind != expected && !matches_style_import {
        return Err(UiV2AssetDocumentError::UnexpectedKind {
            expected,
            actual: parsed.asset.kind,
        });
    }
    Ok(parsed)
}
