use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetImportError, AssetKind, AssetUri, ImportedAsset, MeshSdfCookRequest,
    VirtualGeometryCookRequest,
};
use crate::core::resource::ResourceDiagnostic;

use super::{AssetImportBuildContext, AssetImportBuildIdentity, AssetImportRecipe};

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AssetImporterCapabilityStatus {
    Available,
    DiagnosticOnly { message: String },
}

impl AssetImporterCapabilityStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImporterCapabilityReport {
    pub descriptor: AssetImporterDescriptor,
    pub status: AssetImporterCapabilityStatus,
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
            .map(|extension| normalize_extension_owned(extension.into()))
            .collect();
        self
    }

    pub fn with_full_suffixes(
        mut self,
        suffixes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.full_suffixes = suffixes
            .into_iter()
            .map(|suffix| normalize_full_suffix_owned(suffix.into()))
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
    import_settings: toml::Table,
    import_recipe: AssetImportRecipe,
    build_context: Option<AssetImportBuildContext>,
    build_action_key: Option<String>,
    source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
    source_file_snapshots_authoritative: bool,
    project_resolver: Option<ProjectImportResolver>,
    reference_repairs: std::sync::Arc<std::sync::Mutex<Vec<crate::asset::ReferenceRepair>>>,
}

#[derive(Clone, Debug)]
struct ProjectImportResolver {
    registry: std::sync::Arc<crate::asset::registry::AssetRegistryIndex>,
    roots: std::sync::Arc<Vec<(zircon_runtime_interface::project::RelPath, PathBuf)>>,
}

impl AssetImportContext {
    pub fn new(
        source_path: PathBuf,
        uri: AssetUri,
        source_bytes: Vec<u8>,
        import_settings: toml::Table,
    ) -> Self {
        let import_recipe = AssetImportRecipe::from_legacy_settings(import_settings.clone());
        Self {
            source_path,
            uri,
            source_bytes,
            import_settings,
            import_recipe,
            build_context: None,
            build_action_key: None,
            source_file_snapshots: BTreeMap::new(),
            source_file_snapshots_authoritative: false,
            project_resolver: None,
            reference_repairs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn with_build_identity(mut self, identity: AssetImportBuildIdentity) -> Self {
        let settings_recipe = AssetImportRecipe::from_legacy_settings(self.import_settings.clone());
        assert!(
            identity.recipe().canonical_bytes() == settings_recipe.canonical_bytes(),
            "asset import build identity recipe must match the context settings"
        );
        let (recipe, build_context, action_key) = identity.into_context_parts();
        self.import_recipe = recipe;
        self.build_context = Some(build_context);
        self.build_action_key = Some(action_key);
        self
    }

    pub fn import_settings(&self) -> &toml::Table {
        &self.import_settings
    }

    pub fn import_recipe(&self) -> &AssetImportRecipe {
        &self.import_recipe
    }

    pub fn build_context(&self) -> Option<&AssetImportBuildContext> {
        self.build_context.as_ref()
    }

    pub fn build_action_key(&self) -> Option<&str> {
        self.build_action_key.as_deref()
    }

    pub(crate) fn with_project_resolver(
        mut self,
        registry: std::sync::Arc<crate::asset::registry::AssetRegistryIndex>,
        roots: std::sync::Arc<Vec<(zircon_runtime_interface::project::RelPath, PathBuf)>>,
    ) -> Self {
        self.project_resolver = Some(ProjectImportResolver { registry, roots });
        self
    }

    /// Supplies authoritative transaction-owned companion files without exposing the destination.
    pub fn with_source_file_snapshots(
        mut self,
        source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
    ) -> Self {
        self.source_file_snapshots = source_file_snapshots;
        self.source_file_snapshots_authoritative = true;
        self
    }

    pub(crate) fn source_file_snapshot(&self, path: &std::path::Path) -> Option<&[u8]> {
        self.source_file_snapshots
            .get(path)
            .or_else(|| {
                #[cfg(windows)]
                {
                    self.source_file_snapshots.iter().find_map(|(key, bytes)| {
                        crate::asset::project::ProjectPaths::same_lexical_path(key, path)
                            .unwrap_or(false)
                            .then_some(bytes)
                    })
                }
                #[cfg(not(windows))]
                {
                    None
                }
            })
            .map(Vec::as_slice)
    }

    pub(crate) fn has_source_file_snapshots(&self) -> bool {
        self.source_file_snapshots_authoritative
    }

    pub(crate) fn source_file_snapshot_paths(&self) -> impl Iterator<Item = &std::path::Path> {
        self.source_file_snapshots.keys().map(PathBuf::as_path)
    }

    pub fn resolve_project_asset_ref(
        &self,
        reference: &zircon_runtime_interface::project::PersistedAssetReference,
    ) -> Result<crate::asset::AssetReference, crate::asset::ReferenceResolutionError> {
        let Some(reference) = reference.project_ref() else {
            let locator = reference
                .builtin_locator()
                .ok_or_else(|| crate::asset::ReferenceResolutionError::MissingPayload)?;
            if locator.scheme() != zircon_runtime_interface::resource::ResourceScheme::Builtin {
                return Err(crate::asset::ReferenceResolutionError::UnsupportedScheme {
                    locator: locator.clone(),
                });
            }
            return Ok(crate::asset::AssetReference::from_locator(locator.clone()));
        };
        let resolver = self.project_resolver.as_ref().ok_or_else(|| {
            crate::asset::ReferenceResolutionError::ProjectContextRequired {
                path: self.source_path.clone(),
            }
        })?;
        let resolved = crate::asset::resolve_project_reference(
            &resolver.registry,
            &resolver.roots,
            reference,
        )?;
        if let Some(repair) = resolved.repair {
            self.reference_repairs
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(repair);
        }
        Ok(resolved.reference)
    }

    pub(crate) fn reference_repairs(&self) -> Vec<crate::asset::ReferenceRepair> {
        self.reference_repairs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn source_text(&self) -> Result<String, AssetImportError> {
        self.source_str().map(str::to_owned)
    }

    pub fn source_str(&self) -> Result<&str, AssetImportError> {
        match std::str::from_utf8(&self.source_bytes) {
            Ok(source) => Ok(source),
            Err(_) => {
                let source = String::from_utf8(self.source_bytes.clone())
                    .expect_err("the borrowed UTF-8 check already rejected these bytes");
                Err(AssetImportError::SourceTextDecode {
                    path: self.source_path.clone(),
                    source,
                })
            }
        }
    }

    pub fn virtual_geometry_cook_request(
        &self,
    ) -> Result<VirtualGeometryCookRequest, AssetImportError> {
        VirtualGeometryCookRequest::from_import_settings(self.import_settings())
            .map_err(AssetImportError::Parse)
    }

    pub fn mesh_sdf_cook_request(&self) -> Result<MeshSdfCookRequest, AssetImportError> {
        MeshSdfCookRequest::from_import_settings(self.import_settings())
            .map_err(AssetImportError::Parse)
    }

    pub(crate) fn has_project_resolver(&self) -> bool {
        self.project_resolver.is_some()
    }
}

#[cfg(test)]
#[path = "tests/contract.rs"]
mod tests;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSchemaMigrationReport {
    pub source_schema_version: Option<u32>,
    pub target_schema_version: u32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportedAssetEntry {
    pub locator: AssetUri,
    pub asset: ImportedAsset,
    #[serde(default)]
    pub dependencies: Vec<AssetUri>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_report: Option<AssetSchemaMigrationReport>,
    #[serde(default)]
    pub diagnostics: Vec<ResourceDiagnostic>,
}

impl ImportedAssetEntry {
    pub fn new(locator: AssetUri, asset: ImportedAsset) -> Self {
        Self {
            locator,
            asset,
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

    pub fn with_dependency(mut self, dependency: AssetUri) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetImportOutcome {
    #[serde(default)]
    pub entries: Vec<ImportedAssetEntry>,
    /// Canonical persisted references discovered while importing this source.
    /// Callers may surface or persist these repairs instead of silently losing them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_repairs: Vec<crate::asset::ReferenceRepair>,
}

impl AssetImportOutcome {
    pub fn new(locator: AssetUri, imported_asset: ImportedAsset) -> Self {
        Self {
            entries: vec![ImportedAssetEntry::new(locator, imported_asset)],
            reference_repairs: Vec::new(),
        }
    }

    pub fn with_reference_repairs(
        mut self,
        repairs: impl IntoIterator<Item = crate::asset::ReferenceRepair>,
    ) -> Self {
        self.reference_repairs.extend(repairs);
        self
    }

    pub fn with_entry(mut self, entry: ImportedAssetEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn root_entry(&self) -> Option<&ImportedAssetEntry> {
        self.entries
            .iter()
            .find(|entry| entry.locator.label().is_none())
    }

    pub fn with_migration_report(mut self, migration_report: AssetSchemaMigrationReport) -> Self {
        if let Some(entry) = self.entries.first_mut() {
            entry.migration_report = Some(migration_report);
        }
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: ResourceDiagnostic) -> Self {
        if let Some(entry) = self.entries.first_mut() {
            entry.diagnostics.push(diagnostic);
        }
        self
    }

    pub fn with_dependency(mut self, dependency: AssetUri) -> Self {
        if let Some(entry) = self.entries.first_mut() {
            entry.dependencies.push(dependency);
        }
        self
    }
}

pub trait AssetImporterHandler: fmt::Debug + Send + Sync {
    fn descriptor(&self) -> &AssetImporterDescriptor;

    fn capability_status(&self) -> AssetImporterCapabilityStatus {
        AssetImporterCapabilityStatus::Available
    }

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

    fn capability_status(&self) -> AssetImporterCapabilityStatus {
        AssetImporterCapabilityStatus::DiagnosticOnly {
            message: self.message.clone(),
        }
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

fn normalize_extension_owned(mut extension: String) -> String {
    let (start, len) = {
        let trimmed_start = extension.trim_start();
        let without_dots = trimmed_start.trim_start_matches('.');
        (
            extension.len() - without_dots.len(),
            without_dots.trim_end().len(),
        )
    };
    if start > 0 {
        extension.replace_range(..start, "");
    }
    extension.truncate(len);
    extension.make_ascii_lowercase();
    extension
}

pub(crate) fn normalize_full_suffix(suffix: &str) -> String {
    let trimmed = suffix.trim();
    let mut normalized =
        String::with_capacity(trimmed.len() + usize::from(!trimmed.starts_with('.')));
    if !trimmed.starts_with('.') {
        normalized.push('.');
    }
    normalized.push_str(trimmed);
    normalized.make_ascii_lowercase();
    normalized
}

fn normalize_full_suffix_owned(mut suffix: String) -> String {
    let (start, len) = {
        let trimmed_start = suffix.trim_start();
        (
            suffix.len() - trimmed_start.len(),
            trimmed_start.trim_end().len(),
        )
    };
    if start > 0 {
        suffix.replace_range(..start, "");
    }
    suffix.truncate(len);
    suffix.make_ascii_lowercase();
    if !suffix.starts_with('.') {
        suffix.insert(0, '.');
    }
    suffix
}

#[cfg(test)]
#[path = "tests/contract_plugins07_descriptor_normalization_hotpath_tests.rs"]
mod plugins07_descriptor_normalization_hotpath_tests;
