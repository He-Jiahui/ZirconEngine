mod project_world;

use serde_json::Value;
use zircon_runtime_interface::project::migrate_retired_asset_references;
use zircon_runtime_interface::serialization::MigrateError;

use crate::scene::dynamic_scene::DYNAMIC_SCENE_FORMAT_VERSION;

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
        Value::from(DYNAMIC_SCENE_FORMAT_VERSION),
    );
    Ok(value)
}
