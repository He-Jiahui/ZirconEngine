use serde_json::{Map, Value};
use zircon_runtime_interface::project::migrate_retired_asset_references;
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::MigrateError;

pub(super) fn migrate_reflected_json_v0_to_v1(value: Value) -> Result<Value, MigrateError> {
    let value = migrate_retired_asset_references(value)?;
    let reflected = serde_json::to_value(ReflectedValue::Json(value))
        .map_err(|error| MigrateError::invalid_payload(error.to_string()))?;
    Ok(Value::Object(Map::from_iter([(
        "value".to_string(),
        reflected,
    )])))
}
