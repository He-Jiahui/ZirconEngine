use serde_json::{Map, Value};

use crate::serialization::{Loaded, MigrateError, MigrationChain, MigrationStep, SchemaId};

use super::ProjectManifestSummaryError;

pub const PROJECT_MANIFEST_FORMAT_VERSION: u32 = 2;
const PROJECT_MANIFEST_SCHEMA: SchemaId = SchemaId::new("zircon.project-manifest");

struct ProjectManifestValue;

static PROJECT_MANIFEST_MIGRATIONS: MigrationChain<ProjectManifestValue> = MigrationChain::new(&[
    MigrationStep::new(0, migrate_v0_to_v1),
    MigrationStep::new(1, migrate_v1_to_v2),
]);

/// Parses TOML into the shared JSON value domain and applies the validated manifest chain.
pub fn load_project_manifest_value_from_toml_str(
    document: &str,
) -> Result<Loaded<Value>, ProjectManifestSummaryError> {
    let toml_value = toml::from_str::<toml::Value>(document)
        .map_err(|source| ProjectManifestSummaryError::InvalidToml { source })?;
    let value = serde_json::to_value(toml_value)
        .map_err(|source| ProjectManifestSummaryError::InvalidShape { source })?;
    let source_version = source_version(&value)?;
    if source_version > PROJECT_MANIFEST_FORMAT_VERSION {
        return Err(ProjectManifestSummaryError::FutureVersion {
            found: source_version,
            supported: PROJECT_MANIFEST_FORMAT_VERSION,
        });
    }
    let migrated_from =
        (source_version < PROJECT_MANIFEST_FORMAT_VERSION).then_some(source_version);
    let value = PROJECT_MANIFEST_MIGRATIONS.migrate_value(
        &PROJECT_MANIFEST_SCHEMA,
        value,
        source_version,
        PROJECT_MANIFEST_FORMAT_VERSION,
    )?;
    Ok(Loaded {
        value,
        migrated_from,
    })
}

fn source_version(value: &Value) -> Result<u32, ProjectManifestSummaryError> {
    let Some(raw) = value.get("format_version") else {
        return Ok(1);
    };
    let version = raw
        .as_u64()
        .and_then(|version| u32::try_from(version).ok())
        .ok_or(ProjectManifestSummaryError::InvalidFormatVersion)?;
    Ok(version)
}

fn migrate_v0_to_v1(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "project manifest format_version 0 predates the supported manifest schema",
    ))
}

fn migrate_v1_to_v2(mut value: Value) -> Result<Value, MigrateError> {
    let object = manifest_object(&mut value, 1)?;
    object.insert(
        "format_version".to_string(),
        Value::from(PROJECT_MANIFEST_FORMAT_VERSION),
    );
    object
        .entry("engine_version_req".to_string())
        .or_insert(Value::Null);
    object
        .entry("asset_roots".to_string())
        .or_insert_with(|| Value::Array(vec![Value::String("assets".to_string())]));
    object.entry("settings".to_string()).or_insert(Value::Null);
    Ok(value)
}

fn manifest_object(
    value: &mut Value,
    version: u32,
) -> Result<&mut Map<String, Value>, MigrateError> {
    value.as_object_mut().ok_or_else(|| {
        MigrateError::invalid_payload(format!("project manifest v{version} must be a TOML table"))
    })
}
