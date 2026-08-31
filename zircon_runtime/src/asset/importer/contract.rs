use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetImportError, AssetKind, AssetUri, ImportedAsset, MeshSdfCookRequest,
    VirtualGeometryCookRequest,
};
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
    pub import_settings: toml::Table,
    source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
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
        Self {
            source_path,
            uri,
            source_bytes,
            import_settings,
            source_file_snapshots: BTreeMap::new(),
            project_resolver: None,
            reference_repairs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn with_project_resolver(
        mut self,
        registry: std::sync::Arc<crate::asset::registry::AssetRegistryIndex>,
        roots: std::sync::Arc<Vec<(zircon_runtime_interface::project::RelPath, PathBuf)>>,
    ) -> Self {
        self.project_resolver = Some(ProjectImportResolver { registry, roots });
        self
    }

    /// Supplies transaction-owned companion files without exposing an uncommitted destination.
    pub(crate) fn with_source_file_snapshots(
        mut self,
        source_file_snapshots: BTreeMap<PathBuf, Vec<u8>>,
    ) -> Self {
        self.source_file_snapshots = source_file_snapshots;
        self
    }

    pub(crate) fn source_file_snapshot(&self, path: &std::path::Path) -> Option<&[u8]> {
        self.source_file_snapshots.get(path).map(Vec::as_slice)
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
        VirtualGeometryCookRequest::from_import_settings(&self.import_settings)
            .map_err(AssetImportError::Parse)
    }

    pub fn mesh_sdf_cook_request(&self) -> Result<MeshSdfCookRequest, AssetImportError> {
        MeshSdfCookRequest::from_import_settings(&self.import_settings)
            .map_err(AssetImportError::Parse)
    }

    pub(crate) fn has_project_resolver(&self) -> bool {
        self.project_resolver.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_str_borrows_the_context_utf8_buffer() {
        let context = AssetImportContext::new(
            PathBuf::from("assets/shaders/main.wgsl"),
            AssetUri::parse("res://shaders/main.wgsl").unwrap(),
            b"@compute @workgroup_size(1) fn main() {}".to_vec(),
            toml::Table::new(),
        );

        let source = context.source_str().unwrap();

        assert_eq!(source, "@compute @workgroup_size(1) fn main() {}");
        assert_eq!(source.as_ptr(), context.source_bytes.as_ptr());
        assert_eq!(source.len(), context.source_bytes.len());
    }

    fn context_with_settings(import_settings: toml::Table) -> AssetImportContext {
        AssetImportContext::new(
            PathBuf::from("assets/models/mesh.obj"),
            AssetUri::parse("res://models/mesh.obj").unwrap(),
            Vec::new(),
            import_settings,
        )
    }

    #[test]
    fn context_exposes_disabled_virtual_geometry_request_by_default() {
        assert_eq!(
            context_with_settings(toml::Table::new())
                .virtual_geometry_cook_request()
                .unwrap(),
            VirtualGeometryCookRequest::Disabled
        );
    }

    #[test]
    fn context_rejects_malformed_virtual_geometry_request() {
        let settings = toml::from_str(
            r#"
                [virtual_geometry]
                enabled = "yes"
            "#,
        )
        .unwrap();

        assert!(matches!(
            context_with_settings(settings).virtual_geometry_cook_request(),
            Err(AssetImportError::Parse(message)) if message.contains("virtual_geometry.enabled")
        ));
    }
}

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
mod plugins07_descriptor_normalization_hotpath_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 160_000;
    const EXTENSIONS: [&str; 4] = [" PNG ", " .KtX2 ", " DDS", ".AstC "];
    const SUFFIXES: [&str; 4] = [
        " .SKELETON.ZRANIM ",
        ".Clip.ZRANIM ",
        " .Scene.TOML",
        ".PREFAB.TOML ",
    ];

    #[test]
    fn importer_descriptor_normalization_contract_in_place_extensions() {
        assert_eq!(normalize_extension_owned("  .PnG  ".to_string()), "png");
        let descriptor =
            AssetImporterDescriptor::new("test.normalized", "test.plugin", AssetKind::Texture, 1)
                .with_source_extensions([" .KtX2 ".to_string()]);
        assert_eq!(descriptor.source_extensions, ["ktx2"]);
    }

    #[test]
    fn importer_descriptor_normalization_contract_in_place_full_suffixes() {
        assert_eq!(
            normalize_full_suffix_owned("  Skeleton.ZRANIM  ".to_string()),
            ".skeleton.zranim"
        );
        let descriptor =
            AssetImporterDescriptor::new("test.normalized", "test.plugin", AssetKind::Texture, 1)
                .with_full_suffixes([" Scene.TOML ".to_string()]);
        assert_eq!(descriptor.full_suffixes, [".scene.toml"]);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn importer_descriptor_normalization_performance_release_extensions() {
        let (legacy_samples, optimized_samples) =
            alternating_samples(measure_legacy_extensions, measure_in_place_extensions);
        report_normalization_performance(
            "plugins07_importer_extension_in_place_normalization",
            CHECKS_PER_SAMPLE * 2,
            CHECKS_PER_SAMPLE,
            &legacy_samples,
            &optimized_samples,
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn importer_descriptor_normalization_performance_release_full_suffixes() {
        let (legacy_samples, optimized_samples) =
            alternating_samples(measure_legacy_suffixes, measure_in_place_suffixes);
        report_normalization_performance(
            "plugins07_importer_suffix_in_place_normalization",
            CHECKS_PER_SAMPLE * 2,
            CHECKS_PER_SAMPLE,
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn measure_legacy_extensions() -> u128 {
        measure_owned_normalization(&EXTENSIONS, |value| {
            let owned = value.to_string();
            normalize_extension(black_box(&owned))
        })
    }

    fn measure_in_place_extensions() -> u128 {
        measure_owned_normalization(&EXTENSIONS, |value| {
            normalize_extension_owned(value.to_string())
        })
    }

    fn measure_legacy_suffixes() -> u128 {
        measure_owned_normalization(&SUFFIXES, |value| {
            let owned = value.to_string();
            normalize_full_suffix(black_box(&owned))
        })
    }

    fn measure_in_place_suffixes() -> u128 {
        measure_owned_normalization(&SUFFIXES, |value| {
            normalize_full_suffix_owned(value.to_string())
        })
    }

    fn measure_owned_normalization(
        values: &[&str],
        mut normalize: impl FnMut(&str) -> String,
    ) -> u128 {
        let started = Instant::now();
        let mut total_len = 0_usize;
        for check in 0..CHECKS_PER_SAMPLE {
            let normalized = normalize(black_box(values[check % values.len()]));
            total_len += black_box(normalized.len());
            black_box(normalized);
        }
        black_box(total_len);
        started.elapsed().as_nanos().max(1)
    }

    fn alternating_samples(
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn report_normalization_performance(
        name: &str,
        legacy_allocations_per_sample: usize,
        optimized_allocations_per_sample: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT {name} sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={legacy_allocations_per_sample} optimized_allocations_per_sample={optimized_allocations_per_sample} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            improvement_percent >= 25,
            "in-place importer descriptor normalization must improve P95 by at least 25%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
