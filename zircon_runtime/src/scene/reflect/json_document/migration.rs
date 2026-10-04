use serde_json::{Map, Value};
use zircon_runtime_interface::project::migrate_retired_asset_references;
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::MigrateError;

// 兼容早期未封装的反射 JSON，先迁移已退役的资源引用，再交给当前版本信封解析。
pub(super) fn migrate_reflected_json_v0_to_v1(value: Value) -> Result<Value, MigrateError> {
    let value = migrate_retired_asset_references(value)?;
    let reflected = serde_json::to_value(ReflectedValue::Json(value))
        .map_err(|error| MigrateError::invalid_payload(error.to_string()))?;
    Ok(Value::Object(Map::from_iter([(
        "value".to_string(),
        reflected,
    )])))
}
