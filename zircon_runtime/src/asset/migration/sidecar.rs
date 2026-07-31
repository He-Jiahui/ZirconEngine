use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::AssetUri;
use crate::asset::project::mint_meta_for_migration;
use crate::asset::project::{AssetMetaDocument, AssetSourceUnit};
use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::safe_project_path::is_safe_regular_file;

use super::document::PendingDocument;
use super::resolver_index::MigrationCompoundBinding;
use super::scan::{MigrationInventory, RecognizedSource};
use super::{AssetMigrationIssue, AssetMigrationIssueKind};

pub(super) struct SidecarPreflight {
    pub(super) index: AssetRegistryIndex,
    pub(super) pending: Vec<PendingDocument>,
    pub(super) compound_bindings: Vec<MigrationCompoundBinding>,
}

pub(super) fn preflight_sidecars(
    roots: &[PathBuf],
    inventory: &MigrationInventory,
) -> Result<SidecarPreflight, AssetMigrationIssue> {
    let mut documents = Vec::new();
    let mut pending = Vec::new();
    let mut compound_bindings = Vec::new();
    for path in inventory.sidecar_candidates().iter().cloned() {
        let source =
            fs::read_to_string(&path).map_err(|error| invalid(&path, error.to_string()))?;
        let is_retired_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".meta.toml"));
        let mut value = toml::from_str::<toml::Value>(&source)
            .map_err(|error| invalid(&path, error.to_string()))?;
        let table = value
            .as_table_mut()
            .ok_or_else(|| invalid(&path, "sidecar root must be a table"))?;
        let version_value = table
            .get("format_version")
            .ok_or_else(|| invalid(&path, "sidecar format_version is required"))?;
        let version = version_value
            .as_integer()
            .ok_or_else(|| invalid(&path, "sidecar format_version must be an integer"))?;
        let retired = match version {
            7 => {
                if is_retired_name {
                    return Err(invalid(
                        &path,
                        "retired .meta.toml is accepted only for the explicit v6 contract",
                    ));
                }
                if table.contains_key("source_hash") {
                    return Err(invalid(&path, "v7 sidecar must not contain source_hash"));
                }
                require_digest(table, "source_digest", &path)?;
                false
            }
            6 => {
                if table.contains_key("source_digest") {
                    return Err(invalid(
                        &path,
                        "v6 sidecar must use source_hash, not source_digest",
                    ));
                }
                require_digest(table, "source_hash", &path)?;
                true
            }
            found if found > 7 => {
                return Err(invalid(
                    &path,
                    format!("future sidecar format_version {found} is unsupported"),
                ));
            }
            found => {
                return Err(invalid(
                    &path,
                    format!("sidecar format_version {found} is not in the v6 migration whitelist"),
                ));
            }
        };
        let mut register_document = true;
        if retired {
            table.insert("format_version".to_string(), toml::Value::Integer(7));
            let source_hash = table
                .remove("source_hash")
                .expect("validated v6 source_hash must exist");
            table.insert("source_digest".to_string(), source_hash);
        }
        let canonical =
            toml::to_string_pretty(&value).map_err(|error| invalid(&path, error.to_string()))?;
        let document = AssetMetaDocument::from_toml_str(&canonical)
            .map_err(|error| invalid(&path, error.to_string()))?;
        if !paired_sidecar_source_is_safe(roots, &path, document.unit)? {
            continue;
        }
        if document.unit == AssetSourceUnit::Compound {
            let physical_path = inventory.physical_path_for(&path).ok_or_else(|| {
                invalid(
                    &path,
                    "sidecar is missing from the published migration inventory",
                )
            })?;
            compound_bindings.push(MigrationCompoundBinding::new(
                document.url.clone(),
                physical_path.to_path_buf(),
            ));
        }
        if retired {
            let target = if is_retired_name {
                current_sidecar_path(&path)?
            } else {
                path.clone()
            };
            if target != path && target.exists() {
                let current = fs::read_to_string(&target)
                    .map_err(|error| invalid(&target, error.to_string()))?;
                if current != canonical {
                    return Err(AssetMigrationIssue::new(
                        AssetMigrationIssueKind::RegistryConflict,
                        Some(path),
                        format!(
                            "retired sidecar target {} exists with different content",
                            target.display()
                        ),
                    ));
                }
                // The current v7 target is scanned independently. Registering the
                // converted retired document as well would duplicate its UUID.
                register_document = false;
            }
            pending.push(PendingDocument {
                path: target,
                bytes: canonical.into_bytes(),
                reference_count: 0,
                retired_path: (is_retired_name).then_some(path.clone()),
            });
        }
        if register_document {
            documents.push(document);
        }
    }
    mint_missing_sidecars(inventory.recognized_sources(), &mut documents, &mut pending)?;
    let mut entries = Vec::new();
    for document in documents {
        entries.push(
            AssetRegistryEntry::new(
                document.uuid,
                document.url.clone(),
                document.asset_kind,
                document.source_digest.clone(),
            )
            .with_tags(document.tags.clone()),
        );
        entries.extend(document.entries.into_iter().map(|entry| {
            AssetRegistryEntry::new(
                entry.uuid,
                entry.url,
                entry.asset_kind,
                document.source_digest.clone(),
            )
            .with_tags(entry.tags)
        }));
    }
    let index = AssetRegistryIndex::from_entries(entries).map_err(|error| {
        AssetMigrationIssue::new(
            AssetMigrationIssueKind::RegistryConflict,
            None,
            error.to_string(),
        )
    })?;
    Ok(SidecarPreflight {
        index,
        pending,
        compound_bindings,
    })
}

fn mint_missing_sidecars(
    recognized: &[RecognizedSource],
    documents: &mut Vec<AssetMetaDocument>,
    pending: &mut Vec<PendingDocument>,
) -> Result<(), AssetMigrationIssue> {
    for source in recognized {
        let target = source.path.with_file_name(format!(
            "{}.zmeta",
            source
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| invalid(&source.path, "source name is not valid UTF-8"))?
        ));
        let retired = source.path.with_file_name(format!(
            "{}.meta.toml",
            source.path.file_name().unwrap().to_string_lossy()
        ));
        if target.exists() || retired.exists() {
            continue;
        }
        let candidates = &source.root_relative_identities;
        let relative = match candidates.as_slice() {
            [identity] => identity.relative.as_path(),
            [] => {
                return Err(invalid(
                    &source.path,
                    "source is outside project asset roots",
                ));
            }
            _ => {
                return Err(AssetMigrationIssue::new(
                    AssetMigrationIssueKind::AmbiguousPath,
                    Some(source.path.clone()),
                    "source belongs to multiple project asset roots",
                ));
            }
        };
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let uri = AssetUri::parse(&format!("res://{relative}"))
            .map_err(|error| invalid(&source.path, error.to_string()))?;
        let source_bytes =
            fs::read(&source.path).map_err(|error| invalid(&source.path, error.to_string()))?;
        let bytes = mint_meta_for_migration(&source_bytes, &uri, &source.descriptor)
            .map_err(|error| invalid(&source.path, error.to_string()))?;
        let text =
            std::str::from_utf8(&bytes).map_err(|error| invalid(&target, error.to_string()))?;
        let document = AssetMetaDocument::from_toml_str(text)
            .map_err(|error| invalid(&target, error.to_string()))?;
        documents.push(document);
        pending.push(PendingDocument {
            path: target,
            bytes,
            reference_count: 0,
            retired_path: None,
        });
    }
    Ok(())
}

fn require_digest(
    table: &toml::Table,
    field: &str,
    path: &Path,
) -> Result<(), AssetMigrationIssue> {
    match table.get(field) {
        Some(toml::Value::String(_)) => Ok(()),
        Some(_) => Err(invalid(path, format!("sidecar {field} must be a string"))),
        None => Err(invalid(path, format!("sidecar {field} is required"))),
    }
}

fn paired_sidecar_source_is_safe(
    roots: &[PathBuf],
    sidecar: &Path,
    unit: AssetSourceUnit,
) -> Result<bool, AssetMigrationIssue> {
    let Some(source) = sidecar_source_path(sidecar) else {
        return Ok(false);
    };
    for root in roots {
        if sidecar.strip_prefix(root).is_err() {
            continue;
        }
        let metadata = match fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(invalid(root, error.to_string())),
        };
        match unit {
            AssetSourceUnit::Single
                if metadata.is_file()
                    && is_safe_regular_file(root, &source)
                        .map_err(|error| invalid(root, error.to_string()))? =>
            {
                return Ok(true);
            }
            AssetSourceUnit::Compound
                if metadata.is_dir() && safe_compound_directory(root, &source)? =>
            {
                return Ok(true);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn safe_compound_directory(root: &Path, path: &Path) -> Result<bool, AssetMigrationIssue> {
    let metadata = fs::symlink_metadata(path).map_err(|error| invalid(path, error.to_string()))?;
    if !metadata.is_dir() || crate::asset::safe_project_path::is_link_or_reparse(&metadata) {
        return Ok(false);
    }
    let canonical = fs::canonicalize(path).map_err(|error| invalid(path, error.to_string()))?;
    let canonical_root =
        fs::canonicalize(root).map_err(|error| invalid(root, error.to_string()))?;
    Ok(canonical.starts_with(canonical_root))
}

fn sidecar_source_path(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_str()?;
    let source_name = name
        .strip_suffix(".meta.toml")
        .or_else(|| name.strip_suffix(".zmeta"))?;
    Some(path.with_file_name(source_name))
}

fn current_sidecar_path(path: &Path) -> Result<PathBuf, AssetMigrationIssue> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid(path, "retired sidecar name is not valid UTF-8"))?;
    let source_name = name
        .strip_suffix(".meta.toml")
        .ok_or_else(|| invalid(path, "retired sidecar does not end with .meta.toml"))?;
    Ok(path.with_file_name(format!("{source_name}.zmeta")))
}

fn invalid(path: &Path, message: impl Into<String>) -> AssetMigrationIssue {
    AssetMigrationIssue::new(
        AssetMigrationIssueKind::InvalidDocument,
        Some(path.to_path_buf()),
        message,
    )
}
