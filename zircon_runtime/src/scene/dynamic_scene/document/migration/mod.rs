mod project_world;

use serde_json::Value;
use zircon_runtime_interface::project::migrate_retired_asset_references;
use zircon_runtime_interface::serialization::MigrateError;

const DYNAMIC_SCENE_V1_INNER_FORMAT_VERSION: u32 = 1;

pub(super) fn migrate_dynamic_scene_v0_to_v1(value: Value) -> Result<Value, MigrateError> {
    let mut value = if value.get("world").is_some() {
        project_world::migrate_project_world(value)?
    } else {
        value
    };
    value = migrate_retired_asset_references(value)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("dynamic scene v0 must be an object"))?;
    object.insert(
        "format_version".to_string(),
        Value::from(DYNAMIC_SCENE_V1_INNER_FORMAT_VERSION),
    );
    Ok(value)
}

pub(super) fn migrate_dynamic_scene_v1_to_v2(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("dynamic scene v1 must be an object"))?;
    let format_version = object.remove("format_version").ok_or_else(|| {
        MigrateError::invalid_payload("dynamic scene v1 must contain format_version 1")
    })?;
    if format_version.as_u64() != Some(DYNAMIC_SCENE_V1_INNER_FORMAT_VERSION.into()) {
        return Err(MigrateError::invalid_payload(
            "dynamic scene v1 format_version must equal 1",
        ));
    }
    Ok(value)
}
