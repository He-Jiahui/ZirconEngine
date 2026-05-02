use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetImportError, AssetKind, AssetUri, ImportedAsset};
use crate::core::resource::ResourceDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImporterDescriptor {
    pub id: String,
    pub plugin_id: String,
    pub priority: i32,
    #[serde(default)]
    pub source_extensions: Vec<String>,
    #[serde(default)]
    pub full_suffixes: Vec<String>,
    pub output_kind: AssetKind,
    #[serde(default)]
    pub additional_output_kinds: Vec<AssetKind>,
    pub importer_version: u32,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

impl AssetImporterDescriptor {
    pub fn new(
        id: impl Into<String>,
        plugin_id: impl Into<String>,
        output_kind: AssetKind,
        importer_version: u32,
    ) -> Self {
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            priority: 0,
            source_extensions: Vec::new(),
            full_suffixes: Vec::new(),
            output_kind,
            additional_output_kinds: Vec::new(),
            importer_version,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_source_extensions(
        mut self,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.source_extensions = extensions
            .into_iter()
            .map(|extension| normalize_extension(&extension.into()))
            .collect();
        self
    }

    pub fn with_full_suffixes(
        mut self,
        suffixes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.full_suffixes = suffixes
            .into_iter()
            .map(|suffix| normalize_full_suffix(&suffix.into()))
            .collect();
        self
    }

    pub fn with_required_capabilities(
        mut self,
        capabilities: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.required_capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_additional_output_kinds(
        mut self,
        kinds: impl IntoIterator<Item = AssetKind>,
    ) -> Self {
        self.additional_output_kinds = kinds.into_iter().collect();
        self
    }

    pub fn allows_output_kind(&self, kind: AssetKind) -> bool {
        self.output_kind == kind || self.additional_output_kinds.contains(&kind)
    }
}

#[derive(Clone, Debug)]
pub struct AssetImportContext {
    pub source_path: PathBuf,
    pub uri: AssetUri,
    pub source_bytes: Vec<u8>,
    pub import_settings: toml::Table,
}

impl AssetImportContext {
    pub fn new(
        source_path: PathBuf,
        uri: AssetUri,
        source_bytes: Vec<u8>,
        import_settings: toml::Table,
    ) -> Self {
        Self {
            source_path,
            uri,
            source_bytes,
            import_settings,
        }
    }

    pub fn source_text(&self) -> Result<String, AssetImportError> {
        String::from_utf8(self.source_bytes.clone()).map_err(|error| {
            AssetImportError::Parse(format!(
                "source {} is not valid utf-8: {error}",
                self.source_path.display()
            ))
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSchemaMigrationReport {
    pub source_schema_version: Option<u32>,
    pub target_schema_version: u32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetImportOutcome {
    pub imported_asset: ImportedAsset,
    #[serde(default)]
    pub dependencies: Vec<AssetUri>,
    #[serde(default)]
    pub migration_report: Option<AssetSchemaMigrationReport>,
    #[serde(default)]
    pub diagnostics: Vec<ResourceDiagnostic>,
}

impl AssetImportOutcome {
    pub fn new(imported_asset: ImportedAsset) -> Self {
        Self {
            imported_asset,
            dependencies: Vec::new(),
            migration_report: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_migration_report(mut self, migration_report: AssetSchemaMigrationReport) -> Self {
        self.migration_report = Some(migration_report);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: ResourceDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}

pub trait AssetImporterHandler: fmt::Debug + Send + Sync {
    fn descriptor(&self) -> &AssetImporterDescriptor;

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError>;
}

#[derive(Clone)]
pub struct FunctionAssetImporter {
    descriptor: AssetImporterDescriptor,
    import_fn: fn(&AssetImportContext) -> Result<AssetImportOutcome, AssetImportError>,
}

impl FunctionAssetImporter {
    pub fn new(
        descriptor: AssetImporterDescriptor,
        import_fn: fn(&AssetImportContext) -> Result<AssetImportOutcome, AssetImportError>,
    ) -> Self {
        Self {
            descriptor,
            import_fn,
        }
    }
}

impl fmt::Debug for FunctionAssetImporter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FunctionAssetImporter")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl AssetImporterHandler for FunctionAssetImporter {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        (self.import_fn)(context)
    }
}

#[derive(Clone)]
pub struct DiagnosticOnlyAssetImporter {
    descriptor: AssetImporterDescriptor,
    message: String,
}

impl DiagnosticOnlyAssetImporter {
    pub fn new(descriptor: AssetImporterDescriptor, message: impl Into<String>) -> Self {
        Self {
            descriptor,
            message: message.into(),
        }
    }
}

impl fmt::Debug for DiagnosticOnlyAssetImporter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DiagnosticOnlyAssetImporter")
            .field("descriptor", &self.descriptor)
            .field("message", &self.message)
            .finish()
    }
}

impl AssetImporterHandler for DiagnosticOnlyAssetImporter {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(
        &self,
        _context: &AssetImportContext,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        Err(AssetImportError::UnsupportedFormat(self.message.clone()))
    }
}

pub(crate) fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

pub(crate) fn normalize_full_suffix(suffix: &str) -> String {
    let trimmed = suffix.trim().to_ascii_lowercase();
    if trimmed.starts_with('.') {
        trimmed
    } else {
        format!(".{trimmed}")
    }
}
