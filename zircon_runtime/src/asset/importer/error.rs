use thiserror::Error;

use super::AssetImporterRegistryError;
use crate::asset::ReferenceResolutionError;
use crate::asset::assets::ProjectDocumentError;
use crate::asset::assets::{
    FontAssetError, UiAssetDocumentError, UiIconAssetDocumentError, UiThemeAssetDocumentError,
    UiV2AssetDocumentError,
};
#[cfg(feature = "text")]
use crate::asset::assets::{FontMetadataParseError, FontSourceBudgetError, FontSourceDecodeError};
use crate::asset::project::{ProjectManifestError, ProjectPaths};
use crate::asset::registry::AssetRegistryError;
use crate::core::framework::animation::AnimationAssetError;
use crate::core::resource::{ResourceLocator, ResourceLocatorError};

#[derive(Debug, Error)]
pub enum AssetImportError {
    #[error(transparent)]
    ProjectDocument(#[from] ProjectDocumentError),
    #[error(transparent)]
    ReferenceResolution(#[from] ReferenceResolutionError),
    #[error(transparent)]
    ProjectManifest(#[from] ProjectManifestError),
    #[error("asset I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("asset uri error: {0}")]
    Uri(#[from] ResourceLocatorError),
    #[error(
        "asset source {} is not valid UTF-8: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    SourceTextDecode {
        path: std::path::PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("asset parse failed: {0}")]
    Parse(String),
    #[error(
        "artifact raw payload requires {raw_bytes} bytes, exceeding the {limit_bytes}-byte read limit"
    )]
    ArtifactRawPayloadLimitExceeded { raw_bytes: u64, limit_bytes: u64 },
    #[error(
        "authoring asset {} requires an explicit project registry resolver",
        ProjectPaths::display_path(.path).display()
    )]
    ProjectContextRequired { path: std::path::PathBuf },
    #[error("project has no registered manifest asset roots")]
    MissingProjectAssetRoot,
    #[error(
        "project asset root {} escapes project root {}",
        ProjectPaths::display_path(.root).display(),
        ProjectPaths::display_path(.project_root).display()
    )]
    ProjectAssetRootOutsideProject {
        project_root: std::path::PathBuf,
        root: std::path::PathBuf,
    },
    #[error(
        "canonicalize project root {} failed: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    CanonicalProjectRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "canonicalize project asset root {} failed: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    CanonicalProjectAssetRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "canonical project asset root {} escapes project root {}",
        ProjectPaths::display_path(.asset_root).display(),
        ProjectPaths::display_path(.project_root).display()
    )]
    CanonicalProjectAssetRootEscape {
        project_root: std::path::PathBuf,
        asset_root: std::path::PathBuf,
    },
    #[error(
        "project asset root {} is registered more than once",
        ProjectPaths::display_path(.root).display()
    )]
    DuplicateProjectAssetRoot { root: std::path::PathBuf },
    #[error(
        "project asset uri {uri} resolves from both {} and {}",
        ProjectPaths::display_path(.first).display(),
        ProjectPaths::display_path(.second).display()
    )]
    DuplicateProjectAssetUri {
        uri: ResourceLocator,
        first: std::path::PathBuf,
        second: std::path::PathBuf,
    },
    #[error("project asset uri {uri} does not exist in any registered manifest root")]
    MissingProjectAssetUri { uri: ResourceLocator },
    #[error(
        "project asset uri {uri} exists in multiple registered manifest roots: {display_paths:?}"
    )]
    AmbiguousProjectAssetUri {
        uri: ResourceLocator,
        paths: Vec<std::path::PathBuf>,
        display_paths: Vec<std::path::PathBuf>,
    },
    #[error("project import batch must contain at least one source")]
    EmptyProjectImportBatch,
    #[error("targeted import for {uri} requires a full generation scan: {reason}")]
    TargetedImportRequiresFullScan {
        uri: ResourceLocator,
        reason: String,
    },
    #[error(
        "source path {} is outside all registered manifest asset roots",
        ProjectPaths::display_path(.path).display()
    )]
    SourceOutsideProjectAssetRoots { path: std::path::PathBuf },
    #[error(
        "source path {display_path} belongs to overlapping registered project roots: {display_roots:?}"
    )]
    AmbiguousProjectSourcePath {
        path: std::path::PathBuf,
        roots: Vec<std::path::PathBuf>,
        display_path: std::path::PathBuf,
        display_roots: Vec<std::path::PathBuf>,
    },
    #[error(
        "project asset scan rejected symbolic link or Windows reparse point at {}",
        ProjectPaths::display_path(.path).display()
    )]
    UnsafeProjectAssetLink { path: std::path::PathBuf },
    #[error("font asset document failed: {0}")]
    FontDocument(#[source] FontAssetError),
    #[error(
        "font source {} could not be read: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    FontSourceIo {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[cfg(feature = "text")]
    #[error(
        "font source {} exceeds an import budget: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    FontSourceBudget {
        path: std::path::PathBuf,
        #[source]
        source: FontSourceBudgetError,
    },
    #[cfg(feature = "text")]
    #[error(
        "font source {} could not be decoded: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    FontSourceDecode {
        path: std::path::PathBuf,
        #[source]
        source: FontSourceDecodeError,
    },
    #[cfg(feature = "text")]
    #[error(
        "font source {} metadata is invalid: {source}",
        ProjectPaths::display_path(.path).display()
    )]
    FontMetadata {
        path: std::path::PathBuf,
        #[source]
        source: FontMetadataParseError,
    },
    #[error(
        "font asset {} has invalid source path: {reason}",
        ProjectPaths::display_path(.manifest_path).display()
    )]
    FontSourcePath {
        manifest_path: std::path::PathBuf,
        reason: &'static str,
    },
    #[error("unsupported asset format: {0}")]
    UnsupportedFormat(String),
    #[error("wgsl validation failed: {0}")]
    ShaderValidation(String),
    #[error("asset schema migration failed: {0}")]
    SchemaMigration(String),
    #[error("animation asset decode failed: {0}")]
    AnimationAsset(#[from] AnimationAssetError),
    #[error("native asset importer failed: {0}")]
    Native(String),
    #[error("duplicate asset label {label} for source {source_uri}")]
    DuplicateAssetLabel {
        source_uri: ResourceLocator,
        label: String,
    },
    #[error("missing asset label {label} for source {source_uri}")]
    MissingAssetLabel {
        source_uri: ResourceLocator,
        label: String,
    },
    #[error("asset importer registry failed: {0}")]
    Registry(#[from] AssetImporterRegistryError),
    #[error("asset registry index failed: {0}")]
    RegistryIndex(#[from] AssetRegistryError),
    #[error("asset TOML serialization failed while {context}: {source}")]
    TomlSerialize {
        context: &'static str,
        #[source]
        source: toml::ser::Error,
    },
    #[error("asset TOML deserialization failed while {context}: {source}")]
    TomlDeserialize {
        context: &'static str,
        #[source]
        source: toml::de::Error,
    },
    #[error("asset JSON deserialization failed while {context}: {source}")]
    JsonDeserialize {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("cached TOML datetime `{value}` is invalid: {source}")]
    CachedTomlDatetime {
        value: String,
        #[source]
        source: toml::value::DatetimeParseError,
    },
    #[error("cached JSON number `{value}` is invalid because JSON numbers must be finite")]
    CachedJsonNonFiniteNumber { value: String },
    #[error("cached JSON number `{value}` is invalid: {source}")]
    CachedJsonNumberParse {
        value: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("ui asset document failed while {context}: {source}")]
    UiDocument {
        context: &'static str,
        #[source]
        source: UiAssetDocumentError,
    },
    #[error(".zui asset document failed while {context}: {source}")]
    UiV2Document {
        context: &'static str,
        #[source]
        source: UiV2AssetDocumentError,
    },
    #[error("ui theme asset document failed while {context}: {source}")]
    UiThemeDocument {
        context: &'static str,
        #[source]
        source: UiThemeAssetDocumentError,
    },
    #[error("ui icon asset document failed while {context}: {source}")]
    UiIconDocument {
        context: &'static str,
        #[source]
        source: UiIconAssetDocumentError,
    },
    #[error("artifact cache serialization failed: {0}")]
    ArtifactCacheSerialize(#[source] bincode::Error),
    #[error("artifact cache deserialization failed: {0}")]
    ArtifactCacheDeserialize(#[source] bincode::Error),
    #[error("asset serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl AssetImportError {
    pub(crate) fn ambiguous_project_source_path(
        path: std::path::PathBuf,
        roots: Vec<std::path::PathBuf>,
    ) -> Self {
        let display_path = ProjectPaths::display_path(&path);
        let display_roots = roots.iter().map(ProjectPaths::display_path).collect();
        Self::AmbiguousProjectSourcePath {
            path,
            roots,
            display_path,
            display_roots,
        }
    }

    pub(crate) fn ambiguous_project_asset_uri(
        uri: ResourceLocator,
        paths: Vec<std::path::PathBuf>,
    ) -> Self {
        let display_paths = paths.iter().map(ProjectPaths::display_path).collect();
        Self::AmbiguousProjectAssetUri {
            uri,
            paths,
            display_paths,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::path::PathBuf;

    #[cfg(windows)]
    use crate::asset::AssetUri;

    #[cfg(windows)]
    use super::AssetImportError;

    #[cfg(windows)]
    #[test]
    fn ambiguous_project_asset_uri_keeps_operation_paths_but_displays_virtual_paths() {
        let operation_paths = vec![
            PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj"),
            PathBuf::from(r"\\?\C:\projects\forest\shared-assets\cube.obj"),
        ];
        let error = AssetImportError::ambiguous_project_asset_uri(
            AssetUri::parse("res://models/cube.obj").unwrap(),
            operation_paths.clone(),
        );

        assert!(matches!(
            &error,
            AssetImportError::AmbiguousProjectAssetUri {
                paths,
                display_paths,
                ..
            } if paths == &operation_paths
                && display_paths == &vec![
                    PathBuf::from(r"C:\projects\forest\assets\cube.obj"),
                    PathBuf::from(r"C:\projects\forest\shared-assets\cube.obj"),
                ]
        ));
        let diagnostic = error.to_string();
        assert!(!diagnostic.contains(r"\\?\"));
        assert!(diagnostic.contains(r"C:\projects\forest\assets\cube.obj"));
        assert!(diagnostic.contains(r"C:\projects\forest\shared-assets\cube.obj"));
    }

    #[cfg(windows)]
    #[test]
    fn duplicate_project_asset_uri_displays_windows_virtual_paths() {
        let error = AssetImportError::DuplicateProjectAssetUri {
            uri: AssetUri::parse("res://models/cube.obj").unwrap(),
            first: PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj"),
            second: PathBuf::from(r"\\?\C:\projects\forest\shared-assets\cube.obj"),
        };

        let diagnostic = error.to_string();
        assert!(!diagnostic.contains(r"\\?\"));
        assert!(diagnostic.contains(r"C:\projects\forest\assets\cube.obj"));
        assert!(diagnostic.contains(r"C:\projects\forest\shared-assets\cube.obj"));
    }

    #[cfg(windows)]
    #[test]
    fn ambiguous_project_source_path_keeps_operation_roots_but_displays_virtual_paths() {
        let operation_path = PathBuf::from(r"\\?\C:\projects\forest\assets\cube.obj");
        let operation_roots = vec![
            PathBuf::from(r"\\?\C:\projects\forest\assets"),
            PathBuf::from(r"\\?\C:\projects\forest\shared-assets"),
        ];
        let error = AssetImportError::ambiguous_project_source_path(
            operation_path.clone(),
            operation_roots.clone(),
        );

        assert!(matches!(
            &error,
            AssetImportError::AmbiguousProjectSourcePath {
                path,
                roots,
                display_path,
                display_roots,
            } if path == &operation_path
                && roots == &operation_roots
                && display_path == &PathBuf::from(r"C:\projects\forest\assets\cube.obj")
                && display_roots == &vec![
                    PathBuf::from(r"C:\projects\forest\assets"),
                    PathBuf::from(r"C:\projects\forest\shared-assets"),
                ]
        ));
        assert!(!error.to_string().contains(r"\\?\"));
    }
}
