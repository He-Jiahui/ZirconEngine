use std::str::FromStr;

use serde_json::{Map, Value};

use crate::project::{AssetRef, PersistedAssetReference, RelPath};
use crate::resource::{AssetUuid, ResourceLocator, ResourceScheme};
use crate::serialization::MigrateError;

use super::{RetiredAssetRefMigrationError, RetiredAssetReference};

/// Rewrites only exact retired `{ uuid, url }` objects through the supplied resolver.
pub fn migrate_retired_asset_references_with<E>(
    value: Value,
    mut resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_value(value, &mut resolver)
}

/// Plan 11's context-free v0-to-v1 rule for `res://` project references.
pub fn migrate_retired_asset_references(value: Value) -> Result<Value, MigrateError> {
    migrate_retired_persisted_asset_references_with(value, |reference| {
        let locator = reference.locator();
        if locator.scheme() != ResourceScheme::Res {
            return Err(MigrateError::invalid_payload(format!(
                "retired project asset reference must use res://, found {locator}"
            )));
        }
        let path_hint = RelPath::parse(format!("assets/{}", locator.path()))
            .map_err(|error| MigrateError::invalid_payload(error.to_string()))?;
        AssetRef::try_new(
            reference.guid(),
            path_hint,
            locator.label().map(str::to_string),
        )
        .map_err(|error| MigrateError::invalid_payload(error.to_string()))
    })
    .map_err(flatten_default_error)
}

/// Rewrites project references to `Project(AssetRef)` and retired builtin references to
/// the distinct `Builtin { locator }` authoring contract.
pub fn migrate_retired_persisted_asset_references_with<E>(
    value: Value,
    mut project_resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    migrate_value(value, &mut |reference| {
        if reference.locator().scheme() == ResourceScheme::Builtin {
            return Ok(PersistedAssetReference::builtin(
                reference.locator().clone(),
            ));
        }
        project_resolver(reference).map(PersistedAssetReference::project)
    })
}

/// Rewrites one exact retired `{ uuid, url }` object without traversing its container.
///
/// Format owners use this primitive when a reference is flattened beside owner-specific
/// fields that must not be interpreted by the generic recursive walker.
pub fn migrate_retired_persisted_asset_reference_with<E>(
    value: Value,
    mut project_resolver: impl FnMut(RetiredAssetReference) -> Result<AssetRef, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>> {
    let Value::Object(values) = value else {
        return Err(invalid_shape(
            "retired asset reference must be an object".to_string(),
        ));
    };
    if !is_exact_retired_shape(&values) {
        return Err(invalid_shape(
            "retired asset reference must contain exactly uuid and url".to_string(),
        ));
    }
    migrate_reference(values, &mut |reference| {
        if reference.locator().scheme() == ResourceScheme::Builtin {
            return Ok(PersistedAssetReference::builtin(
                reference.locator().clone(),
            ));
        }
        project_resolver(reference).map(PersistedAssetReference::project)
    })
}

fn migrate_value<E, R>(
    value: Value,
    resolver: &mut impl FnMut(RetiredAssetReference) -> Result<R, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>>
where
    R: serde::Serialize,
{
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| migrate_value(value, resolver))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(values) if is_exact_retired_shape(&values) => {
            migrate_reference(values, resolver)
        }
        Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, migrate_value(value, resolver)?)))
            .collect::<Result<Map<_, _>, _>>()
            .map(Value::Object),
        value => Ok(value),
    }
}

fn is_exact_retired_shape(values: &Map<String, Value>) -> bool {
    values.len() == 2 && values.contains_key("uuid") && values.contains_key("url")
}

fn migrate_reference<E, R>(
    mut values: Map<String, Value>,
    resolver: &mut impl FnMut(RetiredAssetReference) -> Result<R, E>,
) -> Result<Value, RetiredAssetRefMigrationError<E>>
where
    R: serde::Serialize,
{
    let guid = take_string(&mut values, "uuid")?;
    let locator = take_string(&mut values, "url")?;
    let guid = AssetUuid::from_str(&guid).map_err(|error| invalid_shape(error.to_string()))?;
    let locator =
        ResourceLocator::parse(&locator).map_err(|error| invalid_shape(error.to_string()))?;
    let reference = resolver(RetiredAssetReference::new(guid, locator))
        .map_err(RetiredAssetRefMigrationError::Resolve)?;
    serde_json::to_value(reference).map_err(|error| invalid_shape(error.to_string()))
}

fn take_string<E>(
    values: &mut Map<String, Value>,
    field: &'static str,
) -> Result<String, RetiredAssetRefMigrationError<E>> {
    values
        .remove(field)
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| invalid_shape(format!("retired asset {field} must be a string")))
}

fn invalid_shape<E>(message: String) -> RetiredAssetRefMigrationError<E> {
    RetiredAssetRefMigrationError::InvalidShape { message }
}

fn flatten_default_error(error: RetiredAssetRefMigrationError<MigrateError>) -> MigrateError {
    match error {
        RetiredAssetRefMigrationError::InvalidShape { message } => {
            MigrateError::invalid_payload(message)
        }
        RetiredAssetRefMigrationError::Resolve(error) => error,
    }
}
