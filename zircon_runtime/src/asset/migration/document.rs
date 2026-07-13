use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use zircon_runtime_interface::project::{
    migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with, PersistedAssetReference,
    RetiredAssetRefMigrationError,
};

use crate::asset::{ModelAsset, SceneAsset, ZMaterialDocument};

use super::resolver::{MigrationResolver, ResolutionFailure};
use super::{AssetMigrationIssue, AssetMigrationIssueKind};

pub(super) struct PendingDocument {
    pub(super) path: PathBuf,
    pub(super) bytes: Vec<u8>,
    pub(super) reference_count: usize,
    pub(super) retired_path: Option<PathBuf>,
}

pub(super) fn migrate_document(
    path: &Path,
    resolver: &MigrationResolver<'_>,
) -> Result<Option<PendingDocument>, AssetMigrationIssue> {
    let source = fs::read_to_string(path).map_err(|error| invalid(path, error.to_string()))?;
    let toml_value =
        toml::from_str::<toml::Value>(&source).map_err(|error| invalid(path, error.to_string()))?;
    validate_current_document(path, &toml_value)?;
    let original =
        serde_json::to_value(toml_value).map_err(|error| invalid(path, error.to_string()))?;
    let mut reference_count = 0;
    let migrated = if path.extension().and_then(|extension| extension.to_str()) == Some("zmaterial")
    {
        migrate_material_references(original.clone(), resolver, &mut reference_count)
            .map_err(|error| migration_issue(path, error))?
    } else {
        migrate_retired_persisted_asset_references_with(original.clone(), |reference| {
            let resolved = resolver.resolve(reference)?;
            reference_count += 1;
            Ok::<_, ResolutionFailure>(resolved)
        })
        .map_err(|error| migration_issue(path, error))?
    };
    let migrated =
        repair_current_references(migrated, resolver, &mut reference_count).map_err(|error| {
            AssetMigrationIssue::new(error.kind, Some(path.to_path_buf()), error.message)
        })?;
    if migrated == original {
        validate_formal_reader(path, &source, resolver)?;
        return Ok(None);
    }
    let toml_ready = omit_toml_null_subfields(migrated);
    let migrated = serde_json::from_value::<toml::Value>(toml_ready)
        .map_err(|error| invalid(path, error.to_string()))?;
    let bytes = toml::to_string_pretty(&migrated)
        .map_err(|error| invalid(path, error.to_string()))?
        .into_bytes();
    let canonical =
        std::str::from_utf8(&bytes).map_err(|error| invalid(path, error.to_string()))?;
    validate_formal_reader(path, canonical, resolver)?;
    Ok(Some(PendingDocument {
        path: path.to_path_buf(),
        bytes,
        reference_count,
        retired_path: None,
    }))
}

fn repair_current_references(
    value: Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
) -> Result<Value, ResolutionFailure> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| repair_current_references(value, resolver, reference_count))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(mut values) if is_current_project_reference(&values) => {
            let mut reference_fields = serde_json::Map::new();
            for key in ["kind", "guid", "path_hint", "sub"] {
                if let Some(value) = values.remove(key) {
                    reference_fields.insert(key.to_owned(), value);
                }
            }
            let persisted = serde_json::from_value::<PersistedAssetReference>(Value::Object(
                reference_fields.clone(),
            ))
            .map_err(|error| ResolutionFailure {
                kind: AssetMigrationIssueKind::InvalidDocument,
                message: error.to_string(),
            })?;
            let reference = persisted.project_ref().expect("project reference shape");
            let Some(repaired) = resolver.repair_current(reference)? else {
                // Preserve the exact already-current JSON fields. Re-serializing the
                // typed reference would materialize absent optional fields as null,
                // making an unchanged TOML document look dirty on every migration.
                values.extend(reference_fields);
                return Ok(Value::Object(values));
            };
            *reference_count += 1;
            let Value::Object(fields) = serde_json::to_value(PersistedAssetReference::project(
                repaired,
            ))
            .map_err(|error| ResolutionFailure {
                kind: AssetMigrationIssueKind::InvalidDocument,
                message: error.to_string(),
            })?
            else {
                unreachable!("persisted reference serializes as object")
            };
            values.extend(fields);
            Ok(Value::Object(values))
        }
        Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| {
                repair_current_references(value, resolver, reference_count)
                    .map(|value| (key, value))
            })
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(Value::Object),
        value => Ok(value),
    }
}

fn is_current_project_reference(values: &serde_json::Map<String, Value>) -> bool {
    values.get("kind").and_then(Value::as_str) == Some("project")
        && values.contains_key("guid")
        && values.contains_key("path_hint")
}

fn migrate_material_references(
    mut document: Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
) -> Result<Value, RetiredAssetRefMigrationError<ResolutionFailure>> {
    let root =
        document
            .as_object_mut()
            .ok_or_else(|| RetiredAssetRefMigrationError::InvalidShape {
                message: "material document root must be an object".to_string(),
            })?;
    for field in ["shader", "parent"] {
        let Some(reference) = root.get_mut(field) else {
            continue;
        };
        if is_retired_reference_object(reference) {
            *reference = migrate_one_reference(reference.take(), resolver, reference_count)?;
        }
    }
    if let Some(Value::Object(textures)) = root.get_mut("textures") {
        for slot in textures.values_mut() {
            migrate_flattened_material_slot(slot, resolver, reference_count)?;
        }
    }
    Ok(document)
}

fn migrate_flattened_material_slot(
    slot: &mut Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
) -> Result<(), RetiredAssetRefMigrationError<ResolutionFailure>> {
    let Some(values) = slot.as_object_mut() else {
        return Ok(());
    };
    if !values.contains_key("uuid") && !values.contains_key("url") {
        return Ok(());
    }
    let uuid = values.remove("uuid");
    let url = values.remove("url");
    let mut exact = serde_json::Map::new();
    if let Some(uuid) = uuid {
        exact.insert("uuid".to_string(), uuid);
    }
    if let Some(url) = url {
        exact.insert("url".to_string(), url);
    }
    let migrated = migrate_one_reference(Value::Object(exact), resolver, reference_count)?;
    let Value::Object(fields) = migrated else {
        unreachable!("single-reference migration always serializes an object");
    };
    values.extend(fields);
    Ok(())
}

fn migrate_one_reference(
    value: Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
) -> Result<Value, RetiredAssetRefMigrationError<ResolutionFailure>> {
    migrate_retired_persisted_asset_reference_with(value, |reference| {
        let resolved = resolver.resolve(reference)?;
        *reference_count += 1;
        Ok(resolved)
    })
}

fn is_retired_reference_object(value: &Value) -> bool {
    value.as_object().is_some_and(|values| {
        values.len() == 2 && values.contains_key("uuid") && values.contains_key("url")
    })
}

fn validate_formal_reader(
    path: &Path,
    document: &str,
    resolver: &MigrationResolver<'_>,
) -> Result<(), AssetMigrationIssue> {
    let resolve = |reference: &zircon_runtime_interface::project::PersistedAssetReference| {
        resolver.resolve_persisted(reference)
    };
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let result = if name.ends_with(".scene.toml") {
        SceneAsset::from_project_toml_str(document, resolve).map(|_| ())
    } else if name.ends_with(".model.toml") {
        ModelAsset::from_project_toml_str(document, resolve).map(|_| ())
    } else {
        ZMaterialDocument::from_project_toml_str(document, resolve).map(|_| ())
    };
    result.map_err(|error| {
        invalid(
            path,
            format!("formal authoring reader rejected document: {error}"),
        )
    })
}

fn validate_current_document(
    path: &Path,
    document: &toml::Value,
) -> Result<(), AssetMigrationIssue> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("zmaterial") {
        return Ok(());
    }
    match document.get("version").and_then(toml::Value::as_integer) {
        Some(2) => Ok(()),
        Some(version) => Err(invalid(
            path,
            format!("unsupported material schema version {version}; expected version 2"),
        )),
        None => Err(invalid(
            path,
            "material schema version is required".to_string(),
        )),
    }
}

fn omit_toml_null_subfields(value: Value) -> Value {
    match value {
        Value::Array(values) => {
            Value::Array(values.into_iter().map(omit_toml_null_subfields).collect())
        }
        Value::Object(mut values) => {
            let is_asset_ref = values.len() == 4
                && values.get("kind").and_then(Value::as_str) == Some("project")
                && values.contains_key("guid")
                && values.contains_key("path_hint")
                && values.get("sub").is_some_and(Value::is_null);
            if is_asset_ref {
                values.remove("sub");
            }
            Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, omit_toml_null_subfields(value)))
                    .collect(),
            )
        }
        value => value,
    }
}

fn migration_issue(
    path: &Path,
    error: RetiredAssetRefMigrationError<ResolutionFailure>,
) -> AssetMigrationIssue {
    match error {
        RetiredAssetRefMigrationError::InvalidShape { message } => AssetMigrationIssue::new(
            AssetMigrationIssueKind::InvalidDocument,
            Some(path.to_path_buf()),
            message,
        ),
        RetiredAssetRefMigrationError::Resolve(error) => {
            AssetMigrationIssue::new(error.kind, Some(path.to_path_buf()), error.message)
        }
    }
}

fn invalid(path: &Path, message: String) -> AssetMigrationIssue {
    AssetMigrationIssue::new(
        AssetMigrationIssueKind::InvalidDocument,
        Some(path.to_path_buf()),
        message,
    )
}
