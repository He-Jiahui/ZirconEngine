use std::collections::{BTreeSet, HashMap, HashSet};

use crate::asset::importer::{
    AssetImportBuildContext, AssetImportBuildIdentity, AssetImportRecipe,
};
use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{
    AssetId, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind, AssetUri,
    AssetUuid, ImportedAssetEntry,
};

pub(super) fn clear_schema_migration_metadata(meta: &mut AssetMetaDocument) {
    meta.source_schema_version = None;
    meta.target_schema_version = None;
    meta.migration_summary.clear();
}

pub(super) fn apply_importer_metadata(
    meta: &mut AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) {
    if let Some(descriptor) = descriptor {
        meta.importer_id = descriptor.id.clone();
        meta.importer_version = descriptor.importer_version;
    } else {
        meta.importer_id.clear();
        meta.importer_version = 0;
    }
}

pub(super) fn asset_id_for_meta_entry(entry: &AssetMetaEntry) -> AssetId {
    AssetId::from_asset_uuid(entry.uuid)
}

pub(super) fn validate_import_entries(
    source_uri: &AssetUri,
    outcome: &AssetImportOutcome,
) -> Result<(), AssetImportError> {
    if outcome.entries.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "importer did not return any asset entries for {source_uri}"
        )));
    }

    let mut labels = HashSet::with_capacity(outcome.entries.len());
    let mut root_count = 0;
    for entry in &outcome.entries {
        if entry.locator.scheme() != source_uri.scheme()
            || entry.locator.path() != source_uri.path()
        {
            return Err(AssetImportError::Parse(format!(
                "imported asset entry locator {} does not belong to source {source_uri}",
                entry.locator
            )));
        }
        match entry.locator.label() {
            Some(label) => {
                if !labels.insert(label) {
                    return Err(AssetImportError::DuplicateAssetLabel {
                        source_uri: source_uri.clone(),
                        label: label.to_string(),
                    });
                }
            }
            None => root_count += 1,
        }
    }
    if root_count != 1 {
        return Err(AssetImportError::Parse(format!(
            "importer returned {root_count} root entries for {source_uri}; expected exactly one"
        )));
    }
    Ok(())
}

pub(super) fn existing_entry_uuids_for_source(
    meta: &AssetMetaDocument,
    source_uri: &AssetUri,
) -> HashMap<AssetUri, AssetUuid> {
    meta.entries
        .iter()
        .map(|entry| (entry_url_for_source(&entry.url, source_uri), entry.uuid))
        .collect()
}

pub(super) fn existing_entry_tags_for_source(
    meta: &AssetMetaDocument,
    source_uri: &AssetUri,
) -> HashMap<AssetUri, BTreeSet<String>> {
    meta.entries
        .iter()
        .map(|entry| {
            (
                entry_url_for_source(&entry.url, source_uri),
                entry.tags.clone(),
            )
        })
        .collect()
}

pub(super) fn failed_entries_for_source(
    previous_meta: &AssetMetaDocument,
    root_uuid: AssetUuid,
    source_uri: &AssetUri,
    root_kind: AssetKind,
) -> Vec<AssetMetaEntry> {
    if previous_meta.entries.is_empty() {
        return Vec::new();
    }

    previous_meta
        .entries
        .iter()
        .map(|entry| AssetMetaEntry {
            uuid: if entry.url.label().is_none() {
                root_uuid
            } else {
                entry.uuid
            },
            url: entry_url_for_source(&entry.url, source_uri),
            asset_kind: if entry.url.label().is_none() {
                root_kind
            } else {
                entry.asset_kind
            },
            artifact_locator: None,
            dependencies: entry.dependencies.clone(),
            tags: if entry.url.label().is_none() {
                previous_meta.tags.clone()
            } else {
                entry.tags.clone()
            },
        })
        .collect()
}

pub(super) fn remap_meta_entry_urls_to_source(meta: &mut AssetMetaDocument, source_uri: &AssetUri) {
    for entry in &mut meta.entries {
        entry.url = entry_url_for_source(&entry.url, source_uri);
    }
}

fn entry_url_for_source(entry_url: &AssetUri, source_uri: &AssetUri) -> AssetUri {
    if entry_url.label().is_none() {
        source_uri.clone()
    } else {
        AssetUri::new(
            source_uri.scheme(),
            source_uri.path().to_string(),
            entry_url.label().map(ToOwned::to_owned),
        )
        .expect("source URI with existing entry label should be a valid asset URI")
    }
}

pub(super) fn entry_uuid_for_import_entry(
    root_uuid: AssetUuid,
    existing_entry_uuids: &HashMap<AssetUri, AssetUuid>,
    entry: &ImportedAssetEntry,
) -> AssetUuid {
    if entry.locator.label().is_none() {
        root_uuid
    } else {
        existing_entry_uuids
            .get(&entry.locator)
            .copied()
            .unwrap_or_else(AssetUuid::new)
    }
}

pub(super) fn importer_contract_matches(
    meta: &AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) -> bool {
    descriptor
        .map(|descriptor| {
            meta.importer_id == descriptor.id
                && meta.importer_version == descriptor.importer_version
        })
        .unwrap_or_else(|| !meta.importer_id.is_empty())
}

#[cfg(test)]
#[path = "metadata/tests/allocation_tests.rs"]
mod allocation_tests;

#[cfg(test)]
#[path = "tests/metadata_build_identity_tests.rs"]
mod build_identity_tests;

pub(super) fn build_identity_for_import(
    settings: &toml::Table,
    input_digest: &str,
    descriptor: Option<&AssetImporterDescriptor>,
    persisted_importer: Option<(&str, u32)>,
) -> AssetImportBuildIdentity {
    let recipe = AssetImportRecipe::from_legacy_settings(settings.clone());
    let (function_identity, importer_version) = descriptor
        .map(|descriptor| (descriptor.id.clone(), descriptor.importer_version))
        .or_else(|| {
            persisted_importer
                .filter(|(importer_id, _)| !importer_id.is_empty())
                .map(|(importer_id, version)| (importer_id.to_string(), version))
        })
        .unwrap_or_else(|| ("zircon.runtime.asset.unresolved_importer".to_string(), 0));
    let toolchain_identity = format!(
        "zircon-runtime@{};{}@{}",
        env!("CARGO_PKG_VERSION"),
        function_identity,
        importer_version
    );
    let build_context = AssetImportBuildContext::project_import(toolchain_identity);
    AssetImportBuildIdentity::new(function_identity, recipe, input_digest, build_context)
}
