use crate::core::resource::{AssetReference, ResourceLocator};
use crate::ui::template::UiAssetLoader;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};

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

pub fn ui_asset_references(document: &UiAssetDocument) -> Vec<AssetReference> {
    let mut references = Vec::new();
    for reference in document
        .imports
        .widgets
        .iter()
        .chain(document.imports.styles.iter())
    {
        if let Ok(locator) = ResourceLocator::parse(reference) {
            if let Ok(asset_locator) =
                ResourceLocator::new(locator.scheme(), locator.path().to_string(), None)
            {
                references.push(AssetReference::from_locator(asset_locator));
            }
        }
    }
    references
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
