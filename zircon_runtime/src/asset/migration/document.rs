use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::{
    migrate_retired_persisted_asset_reference_with, PersistedAssetReference,
    RetiredAssetRefMigrationError,
};

use crate::asset::assets::project_document::{
    deserialize_material_artifact, deserialize_model_artifact, deserialize_scene_artifact,
    ProjectDocumentArtifact,
};

use super::resolver::{MigrationResolver, ResolutionFailure};
use super::{AssetMigrationIssue, AssetMigrationIssueKind};

pub(super) struct PendingDocument {
    pub(super) path: PathBuf,
    pub(super) bytes: Vec<u8>,
    pub(super) reference_count: usize,
    pub(super) retired_path: Option<PathBuf>,
}

/// Observations from one authoring document consumed by the migration generation.
pub(super) struct DocumentMigrationResult {
    pub(super) pending: Option<PendingDocument>,
    pub(super) reference_visits: usize,
}

struct MigrationDocumentArtifact {
    document: ProjectDocumentArtifact,
    changed: bool,
}

impl MigrationDocumentArtifact {
    fn parse(path: &Path, source: &str) -> Result<Self, AssetMigrationIssue> {
        let document = ProjectDocumentArtifact::parse(source)
            .map_err(|error| invalid(path, error.to_string()))?;
        Ok(Self {
            document,
            changed: false,
        })
    }

    fn value(&self) -> &toml::Value {
        self.document.value()
    }

    fn value_mut(&mut self) -> &mut toml::Value {
        self.document.value_mut()
    }

    fn record_change(&mut self, changed: bool) {
        self.changed |= changed;
    }

    fn changed(&self) -> bool {
        self.changed
    }

    fn to_pretty_bytes(&self, path: &Path) -> Result<Vec<u8>, AssetMigrationIssue> {
        self.document
            .to_pretty_bytes()
            .map_err(|error| invalid(path, error.to_string()))
    }

    fn into_project_document(self) -> ProjectDocumentArtifact {
        self.document
    }
}

pub(super) fn migrate_document(
    path: &Path,
    resolver: &MigrationResolver<'_>,
) -> Result<DocumentMigrationResult, AssetMigrationIssue> {
    let source = fs::read_to_string(path).map_err(|error| invalid(path, error.to_string()))?;
    let mut artifact = MigrationDocumentArtifact::parse(path, &source)?;

    let mut reference_count = 0;
    let mut reference_visits = 0;
    let retired_changed = if is_material_document(path) {
        migrate_material_references(
            artifact.value_mut(),
            resolver,
            &mut reference_count,
            &mut reference_visits,
        )
        .map_err(|error| migration_issue(path, error))?
    } else {
        migrate_retired_references(
            artifact.value_mut(),
            resolver,
            &mut reference_count,
            &mut reference_visits,
        )
        .map_err(|error| migration_issue(path, error))?
    };
    artifact.record_change(retired_changed);
    let current_changed = repair_current_references(
        artifact.value_mut(),
        resolver,
        &mut reference_count,
        &mut reference_visits,
    )
    .map_err(|error| {
        AssetMigrationIssue::new(error.kind, Some(path.to_path_buf()), error.message)
    })?;
    artifact.record_change(current_changed);

    let bytes = artifact
        .changed()
        .then(|| artifact.to_pretty_bytes(path))
        .transpose()?;
    validate_formal_reader(
        path,
        artifact.into_project_document(),
        resolver,
        &mut reference_visits,
    )?;

    Ok(DocumentMigrationResult {
        pending: bytes.map(|bytes| PendingDocument {
            path: path.to_path_buf(),
            bytes,
            reference_count,
            retired_path: None,
        }),
        reference_visits,
    })
}

fn migrate_retired_references(
    value: &mut toml::Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<bool, RetiredAssetRefMigrationError<ResolutionFailure>> {
    match value {
        toml::Value::Array(values) => {
            let mut changed = false;
            for value in values {
                changed |=
                    migrate_retired_references(value, resolver, reference_count, reference_visits)?;
            }
            Ok(changed)
        }
        toml::Value::Table(values) if is_retired_reference_table(values) => {
            let reference = std::mem::replace(value, toml::Value::Table(toml::Table::new()));
            *value = migrate_one_reference(reference, resolver, reference_count, reference_visits)?;
            Ok(true)
        }
        toml::Value::Table(values) => {
            let mut changed = false;
            for (_, value) in values.iter_mut() {
                changed |=
                    migrate_retired_references(value, resolver, reference_count, reference_visits)?;
            }
            Ok(changed)
        }
        _ => Ok(false),
    }
}

fn repair_current_references(
    value: &mut toml::Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<bool, ResolutionFailure> {
    match value {
        toml::Value::Array(values) => {
            let mut changed = false;
            for value in values {
                changed |=
                    repair_current_references(value, resolver, reference_count, reference_visits)?;
            }
            Ok(changed)
        }
        toml::Value::Table(values) if is_current_project_reference(values) => {
            repair_current_reference(values, resolver, reference_count, reference_visits)
        }
        toml::Value::Table(values) => {
            let mut changed = false;
            for (_, value) in values.iter_mut() {
                changed |=
                    repair_current_references(value, resolver, reference_count, reference_visits)?;
            }
            Ok(changed)
        }
        _ => Ok(false),
    }
}

fn repair_current_reference(
    values: &mut toml::Table,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<bool, ResolutionFailure> {
    let mut reference_fields = toml::Table::new();
    for key in ["kind", "guid", "path_hint", "sub"] {
        if let Some(value) = values.remove(key) {
            reference_fields.insert(key.to_owned(), value);
        }
    }
    let persisted = parse_current_reference(&reference_fields)?;
    let reference = persisted.project_ref().expect("project reference shape");
    *reference_visits += 1;
    let repaired = resolver.repair_current(reference)?;
    let changed = repaired.is_some();
    let fields = match repaired {
        Some(repaired) => {
            *reference_count += 1;
            serialize_current_reference(PersistedAssetReference::project(repaired))?
        }
        None => reference_fields,
    };
    values.extend(fields);
    Ok(changed)
}

fn parse_current_reference(
    reference_fields: &toml::Table,
) -> Result<PersistedAssetReference, ResolutionFailure> {
    let value = serde_json::to_value(toml::Value::Table(reference_fields.clone()))
        .map_err(invalid_resolution)?;
    serde_json::from_value::<PersistedAssetReference>(value).map_err(invalid_resolution)
}

fn serialize_current_reference(
    reference: PersistedAssetReference,
) -> Result<toml::Table, ResolutionFailure> {
    let value = serde_json::to_value(reference).map_err(invalid_resolution)?;
    persisted_reference_table(value).map_err(invalid_resolution)
}

fn persisted_reference_table(
    mut value: serde_json::Value,
) -> Result<toml::Table, serde_json::Error> {
    if let Some(fields) = value.as_object_mut() {
        let is_project = fields.get("kind").and_then(serde_json::Value::as_str) == Some("project");
        if is_project && fields.get("sub").is_some_and(serde_json::Value::is_null) {
            fields.remove("sub");
        }
    }
    let toml::Value::Table(fields) = serde_json::from_value::<toml::Value>(value)? else {
        unreachable!("persisted reference serializes as a TOML table")
    };
    Ok(fields)
}

fn is_current_project_reference(values: &toml::Table) -> bool {
    values.get("kind").and_then(toml::Value::as_str) == Some("project")
        && values.contains_key("guid")
        && values.contains_key("path_hint")
}

fn migrate_material_references(
    document: &mut toml::Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<bool, RetiredAssetRefMigrationError<ResolutionFailure>> {
    let root =
        document
            .as_table_mut()
            .ok_or_else(|| RetiredAssetRefMigrationError::InvalidShape {
                message: "material document root must be an object".to_string(),
            })?;
    let mut changed = false;
    for field in ["shader", "parent"] {
        let Some(reference) = root.get_mut(field) else {
            continue;
        };
        if is_retired_reference(reference) {
            let retired = std::mem::replace(reference, toml::Value::Table(toml::Table::new()));
            *reference =
                migrate_one_reference(retired, resolver, reference_count, reference_visits)?;
            changed = true;
        }
    }
    if let Some(toml::Value::Table(textures)) = root.get_mut("textures") {
        for (_, slot) in textures.iter_mut() {
            changed |=
                migrate_flattened_material_slot(slot, resolver, reference_count, reference_visits)?;
        }
    }
    Ok(changed)
}

fn migrate_flattened_material_slot(
    slot: &mut toml::Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<bool, RetiredAssetRefMigrationError<ResolutionFailure>> {
    let Some(values) = slot.as_table_mut() else {
        return Ok(false);
    };
    let Some(exact) = take_retired_material_reference_fields(values) else {
        return Ok(false);
    };
    let migrated = migrate_one_reference(
        toml::Value::Table(exact),
        resolver,
        reference_count,
        reference_visits,
    )?;
    let toml::Value::Table(fields) = migrated else {
        unreachable!("single-reference migration always serializes a TOML table")
    };
    values.extend(fields);
    Ok(true)
}

fn take_retired_material_reference_fields(values: &mut toml::Table) -> Option<toml::Table> {
    let mut exact = toml::Table::new();
    if let Some(uuid) = values.remove("uuid") {
        exact.insert("uuid".to_string(), uuid);
    }
    if let Some(url) = values.remove("url") {
        exact.insert("url".to_string(), url);
    }
    (!exact.is_empty()).then_some(exact)
}

fn migrate_one_reference(
    value: toml::Value,
    resolver: &MigrationResolver<'_>,
    reference_count: &mut usize,
    reference_visits: &mut usize,
) -> Result<toml::Value, RetiredAssetRefMigrationError<ResolutionFailure>> {
    let value = serde_json::to_value(value).map_err(invalid_migration_shape)?;
    let migrated = migrate_retired_persisted_asset_reference_with(value, |reference| {
        let resolved = resolver.resolve(reference)?;
        *reference_count += 1;
        *reference_visits += 1;
        Ok(resolved)
    })?;
    persisted_reference_table(migrated)
        .map(toml::Value::Table)
        .map_err(invalid_migration_shape)
}

fn is_retired_reference(value: &toml::Value) -> bool {
    value.as_table().is_some_and(is_retired_reference_table)
}

fn is_retired_reference_table(values: &toml::Table) -> bool {
    values.len() == 2 && values.contains_key("uuid") && values.contains_key("url")
}

fn validate_formal_reader(
    path: &Path,
    document: ProjectDocumentArtifact,
    resolver: &MigrationResolver<'_>,
    reference_visits: &mut usize,
) -> Result<(), AssetMigrationIssue> {
    let resolve = |reference: &PersistedAssetReference| {
        *reference_visits += 1;
        resolver.resolve_persisted(reference)
    };
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let result = if name.ends_with(".scene.toml") {
        deserialize_scene_artifact(document, resolve).map(|_| ())
    } else if name.ends_with(".model.toml") {
        deserialize_model_artifact(document, resolve).map(|_| ())
    } else {
        deserialize_material_artifact(document, resolve).map(|_| ())
    };
    result.map_err(|error| {
        invalid(
            path,
            format!("formal authoring reader rejected document: {error}"),
        )
    })
}

fn is_material_document(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("zmaterial")
}

fn invalid_resolution(error: impl std::fmt::Display) -> ResolutionFailure {
    ResolutionFailure {
        kind: AssetMigrationIssueKind::InvalidDocument,
        message: error.to_string(),
    }
}

fn invalid_migration_shape<E>(error: impl std::fmt::Display) -> RetiredAssetRefMigrationError<E> {
    RetiredAssetRefMigrationError::InvalidShape {
        message: error.to_string(),
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
        RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource,
            max,
            found,
        } => AssetMigrationIssue::new(
            AssetMigrationIssueKind::InvalidDocument,
            Some(path.to_path_buf()),
            format!(
                "retired asset reference migration {resource} limit {max} exceeded (found {found})"
            ),
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

#[cfg(test)]
#[path = "document/single_pass_material_reference_tests.rs"]
mod single_pass_material_reference_tests;
