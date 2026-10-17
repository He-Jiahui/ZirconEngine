//! Decode failures shared by desktop cache reads and portable Raw assets.
#[cfg(feature = "native-host")]
pub(crate) use crate::asset::AssetImportError as AssetCodecError;
#[cfg(not(feature = "native-host"))]
#[derive(Debug, thiserror::Error)]
pub(crate) enum AssetCodecError {
    #[error("invalid cooked asset: {0}")]
    Parse(String),
    #[error("cached JSON number must be finite: {value}")]
    CachedJsonNonFiniteNumber { value: String },
    #[error("invalid cached JSON number {value}: {source}")]
    CachedJsonNumberParse {
        value: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid cached TOML datetime {value}: {source}")]
    CachedTomlDatetime {
        value: String,
        #[source]
        source: toml::value::DatetimeParseError,
    },
    #[error("TOML serialization failed while {context}: {source}")]
    TomlSerialize {
        context: &'static str,
        #[source]
        source: toml::ser::Error,
    },
    #[error("UI decoding failed while {context}: {source}")]
    UiDocument {
        context: &'static str,
        #[source]
        source: crate::asset::assets::UiAssetDocumentError,
    },
    #[error("ZUI decoding failed while {context}: {source}")]
    UiV2Document {
        context: &'static str,
        #[source]
        source: crate::asset::assets::UiV2AssetDocumentError,
    },
}
