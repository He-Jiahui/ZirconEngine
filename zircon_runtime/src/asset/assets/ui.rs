use std::collections::{BTreeMap, HashSet};

use crate::core::resource::{AssetReference, ResourceLocator};
use crate::ui::template::{collect_document_resource_dependencies, UiAssetLoader};
use crate::ui::v2::UiV2AssetLoader;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

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

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiAssetDocumentError {
    #[error("failed to parse ui asset document: {0}")]
    Parse(String),
    #[error("expected ui asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiV2AssetDocumentError {
    #[error("failed to parse ui v2 asset document: {0}")]
    Parse(String),
    #[error("expected ui v2 asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiV2AssetKind,
        actual: UiV2AssetKind,
    },
}

impl UiLayoutAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiAssetDocumentError> {
        parse_typed(document, UiAssetKind::Layout).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiWidgetAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiAssetDocumentError> {
        parse_typed(document, UiAssetKind::Widget).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiStyleAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiAssetDocumentError> {
        parse_typed(document, UiAssetKind::Style).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiV2ViewAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiV2AssetDocumentError> {
        parse_v2_typed(document, UiV2AssetKind::View).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiV2ComponentAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiV2AssetDocumentError> {
        parse_v2_typed(document, UiV2AssetKind::Component).map(|document| Self { document })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.document)
    }
}

impl UiV2StyleAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, UiV2AssetDocumentError> {
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

fn parse_typed(
    document: &str,
    expected: UiAssetKind,
) -> Result<UiAssetDocument, UiAssetDocumentError> {
    let parsed = UiAssetLoader::load_toml_str(document)
        .map_err(|error| UiAssetDocumentError::Parse(error.to_string()))?;
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
) -> Result<UiV2AssetDocument, UiV2AssetDocumentError> {
    let parsed = UiV2AssetLoader::load_toml_str(document)
        .map_err(|error| UiV2AssetDocumentError::Parse(error.to_string()))?;
    if parsed.asset.kind != expected {
        return Err(UiV2AssetDocumentError::UnexpectedKind {
            expected,
            actual: parsed.asset.kind,
        });
    }
    Ok(parsed)
}
