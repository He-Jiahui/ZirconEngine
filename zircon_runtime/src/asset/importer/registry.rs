use std::fmt;
use std::path::Path;
use std::sync::Arc;

use thiserror::Error;

use super::{
    normalize_extension, normalize_full_suffix, AssetImporterCapabilityReport,
    AssetImporterDescriptor, AssetImporterHandler,
};
use crate::asset::AssetImportError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetImporterRegistryError {
    #[error("asset importer {0} already registered")]
    DuplicateImporterId(String),
    #[error("duplicate importer matcher {matcher} at priority {priority}")]
    DuplicateMatcher { matcher: String, priority: i32 },
    #[error("asset importer {0} cannot register legacy .ui.toml; UI assets must use .zui")]
    LegacyUiTomlImporter(String),
    #[error("asset importer {0} cannot register legacy .v2.ui.toml; UI components must use .zui")]
    LegacyV2UiTomlImporter(String),
    #[error("asset importer {0} must declare at least one source extension or full suffix")]
    MissingMatcher(String),
}

#[derive(Clone, Default)]
pub struct AssetImporterRegistry {
    importers: Vec<Arc<dyn AssetImporterHandler>>,
}

impl AssetImporterRegistry {
    pub fn register(
        &mut self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), AssetImporterRegistryError> {
        self.register_arc(Arc::new(importer))
    }

    pub fn register_arc(
        &mut self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), AssetImporterRegistryError> {
        validate_descriptor(importer.descriptor())?;
        if self
            .importers
            .iter()
            .any(|existing| existing.descriptor().id == importer.descriptor().id)
        {
            return Err(AssetImporterRegistryError::DuplicateImporterId(
                importer.descriptor().id.clone(),
            ));
        }
        for matcher in matcher_keys(importer.descriptor()) {
            if self.importers.iter().any(|existing| {
                existing.descriptor().priority == importer.descriptor().priority
                    && matcher_keys(existing.descriptor()).any(|existing| existing == matcher)
            }) {
                return Err(AssetImporterRegistryError::DuplicateMatcher {
                    matcher,
                    priority: importer.descriptor().priority,
                });
            }
        }
        self.importers.push(importer);
        Ok(())
    }

    pub fn select(
        &self,
        source_path: &Path,
    ) -> Result<Arc<dyn AssetImporterHandler>, AssetImportError> {
        if let Some(importer) = self.best_full_suffix_match(source_path) {
            return Ok(importer);
        }
        if lower_file_name(source_path).ends_with(".v2.ui.toml") {
            return Err(AssetImportError::UnsupportedFormat(
                "legacy UI v2 suffix `.v2.ui.toml` has no registered importer; use `.zui`"
                    .to_string(),
            ));
        }
        if let Some(suffix) = unknown_typed_toml_suffix(source_path) {
            return Err(AssetImportError::UnsupportedFormat(format!(
                "typed toml asset suffix `{suffix}` has no registered importer"
            )));
        }
        if let Some(importer) = self.best_extension_match(source_path) {
            return Ok(importer);
        }
        Err(AssetImportError::UnsupportedFormat(format!(
            "no asset importer registered for {}",
            source_path.display()
        )))
    }

    pub fn descriptor_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterDescriptor, AssetImportError> {
        self.select(source_path)
            .map(|importer| importer.descriptor().clone())
    }

    pub fn capability_report_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError> {
        self.select(source_path)
            .map(|importer| AssetImporterCapabilityReport {
                descriptor: importer.descriptor().clone(),
                status: importer.capability_status(),
            })
    }

    pub fn capability_reports(&self) -> Vec<AssetImporterCapabilityReport> {
        self.importers
            .iter()
            .map(|importer| AssetImporterCapabilityReport {
                descriptor: importer.descriptor().clone(),
                status: importer.capability_status(),
            })
            .collect()
    }

    pub fn descriptors(&self) -> Vec<AssetImporterDescriptor> {
        self.importers
            .iter()
            .map(|importer| importer.descriptor().clone())
            .collect()
    }

    pub fn importers(&self) -> Vec<Arc<dyn AssetImporterHandler>> {
        self.importers.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.importers.is_empty()
    }

    fn best_full_suffix_match(&self, source_path: &Path) -> Option<Arc<dyn AssetImporterHandler>> {
        let name = lower_file_name(source_path);
        self.importers
            .iter()
            .filter_map(|importer| {
                let suffix_len = importer
                    .descriptor()
                    .full_suffixes
                    .iter()
                    .map(|suffix| normalize_full_suffix(suffix))
                    .filter(|suffix| name.ends_with(suffix))
                    .map(|suffix| suffix.len())
                    .max()?;
                Some((
                    importer.clone(),
                    capability_rank(importer.as_ref()),
                    importer.descriptor().priority,
                    suffix_len,
                ))
            })
            .max_by(|left, right| {
                left.1
                    .cmp(&right.1)
                    .then_with(|| left.2.cmp(&right.2))
                    .then_with(|| left.3.cmp(&right.3))
            })
            .map(|(importer, _, _, _)| importer)
    }

    fn best_extension_match(&self, source_path: &Path) -> Option<Arc<dyn AssetImporterHandler>> {
        let extension = source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(normalize_extension)?;
        self.importers
            .iter()
            .filter(|importer| {
                importer
                    .descriptor()
                    .source_extensions
                    .iter()
                    .map(|extension| normalize_extension(extension))
                    .any(|candidate| candidate == extension)
            })
            .max_by(|left, right| {
                capability_rank(left.as_ref())
                    .cmp(&capability_rank(right.as_ref()))
                    .then_with(|| left.descriptor().priority.cmp(&right.descriptor().priority))
            })
            .cloned()
    }
}

fn capability_rank(importer: &dyn AssetImporterHandler) -> u8 {
    if importer.capability_status().is_available() {
        1
    } else {
        0
    }
}

impl fmt::Debug for AssetImporterRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetImporterRegistry")
            .field("descriptors", &self.descriptors())
            .finish()
    }
}

fn validate_descriptor(
    descriptor: &AssetImporterDescriptor,
) -> Result<(), AssetImporterRegistryError> {
    if descriptor.source_extensions.is_empty() && descriptor.full_suffixes.is_empty() {
        return Err(AssetImporterRegistryError::MissingMatcher(
            descriptor.id.clone(),
        ));
    }
    if descriptor
        .full_suffixes
        .iter()
        .any(|suffix| normalize_full_suffix(suffix) == ".ui.toml")
        && !legacy_ui_toml_importer_allowed_for_tests(descriptor)
    {
        return Err(AssetImporterRegistryError::LegacyUiTomlImporter(
            descriptor.id.clone(),
        ));
    }
    if descriptor
        .full_suffixes
        .iter()
        .any(|suffix| normalize_full_suffix(suffix) == ".v2.ui.toml")
        && !legacy_v2_ui_toml_importer_allowed_for_tests(descriptor)
    {
        return Err(AssetImporterRegistryError::LegacyV2UiTomlImporter(
            descriptor.id.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
fn legacy_ui_toml_importer_allowed_for_tests(descriptor: &AssetImporterDescriptor) -> bool {
    descriptor.id == "ui_document_importer.typed_toml"
        && descriptor.plugin_id == "ui_document_importer"
        && descriptor
            .required_capabilities
            .iter()
            .any(|capability| capability == "runtime.asset.importer.ui_document")
}

#[cfg(test)]
fn legacy_v2_ui_toml_importer_allowed_for_tests(descriptor: &AssetImporterDescriptor) -> bool {
    descriptor.id == "ui_document_importer.v2_typed_toml"
        && descriptor.plugin_id == "ui_document_importer"
        && descriptor
            .required_capabilities
            .iter()
            .any(|capability| capability == "runtime.asset.importer.ui_document.v2")
}

#[cfg(not(test))]
fn legacy_ui_toml_importer_allowed_for_tests(_descriptor: &AssetImporterDescriptor) -> bool {
    false
}

#[cfg(not(test))]
fn legacy_v2_ui_toml_importer_allowed_for_tests(_descriptor: &AssetImporterDescriptor) -> bool {
    false
}

fn matcher_keys(descriptor: &AssetImporterDescriptor) -> impl Iterator<Item = String> + '_ {
    descriptor
        .full_suffixes
        .iter()
        .map(|suffix| format!("suffix:{}", normalize_full_suffix(suffix)))
        .chain(
            descriptor
                .source_extensions
                .iter()
                .map(|extension| format!("ext:{}", normalize_extension(extension))),
        )
}

fn lower_file_name(source_path: &Path) -> String {
    source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn unknown_typed_toml_suffix(source_path: &Path) -> Option<String> {
    let name = lower_file_name(source_path);
    if !name.ends_with(".toml") {
        return None;
    }
    let stem = name.strip_suffix(".toml")?;
    let typed_suffix_start = stem.rfind('.')?;
    let suffix = &stem[typed_suffix_start..];
    if suffix.is_empty() {
        return None;
    }
    Some(format!("{suffix}.toml"))
}
