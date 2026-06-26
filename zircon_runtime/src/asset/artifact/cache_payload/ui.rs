use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetImportError, UiLayoutAsset, UiStyleAsset, UiV2ComponentAsset, UiV2StyleAsset,
    UiV2ViewAsset, UiWidgetAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheUiAssetDocument {
    document_toml: String,
}

impl ArtifactCacheUiAssetDocument {
    pub(super) fn from_document(
        document: &zircon_runtime_interface::ui::template::UiAssetDocument,
    ) -> Result<Self, AssetImportError> {
        toml::to_string(document)
            .map(|document_toml| Self { document_toml })
            .map_err(|source| AssetImportError::TomlSerialize {
                context: "serialize ui asset document cache",
                source,
            })
    }

    pub(super) fn into_layout_asset(self) -> Result<UiLayoutAsset, AssetImportError> {
        UiLayoutAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiDocument {
                context: "deserialize ui layout document cache",
                source,
            }
        })
    }

    pub(super) fn into_widget_asset(self) -> Result<UiWidgetAsset, AssetImportError> {
        UiWidgetAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiDocument {
                context: "deserialize ui widget document cache",
                source,
            }
        })
    }

    pub(super) fn into_style_asset(self) -> Result<UiStyleAsset, AssetImportError> {
        UiStyleAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiDocument {
                context: "deserialize ui style document cache",
                source,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheUiV2AssetDocument {
    document_toml: String,
}

impl ArtifactCacheUiV2AssetDocument {
    pub(super) fn from_document(
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
    ) -> Result<Self, AssetImportError> {
        toml::to_string(document)
            .map(|document_toml| Self { document_toml })
            .map_err(|source| AssetImportError::TomlSerialize {
                context: "serialize ui v2 asset document cache",
                source,
            })
    }

    pub(super) fn into_view_asset(self) -> Result<UiV2ViewAsset, AssetImportError> {
        UiV2ViewAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiV2Document {
                context: "deserialize ui v2 view document cache",
                source,
            }
        })
    }

    pub(super) fn into_component_asset(self) -> Result<UiV2ComponentAsset, AssetImportError> {
        UiV2ComponentAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiV2Document {
                context: "deserialize ui v2 component document cache",
                source,
            }
        })
    }

    pub(super) fn into_style_asset(self) -> Result<UiV2StyleAsset, AssetImportError> {
        UiV2StyleAsset::from_toml_str(&self.document_toml).map_err(|source| {
            AssetImportError::UiV2Document {
                context: "deserialize ui v2 style document cache",
                source,
            }
        })
    }
}
