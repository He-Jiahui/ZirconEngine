mod project_world;

use serde_json::Value;
use zircon_runtime_interface::project::migrate_retired_asset_references;
use zircon_runtime_interface::reflect::ReflectFieldId;
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

pub(super) fn migrate_dynamic_scene_v2_to_v3(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("dynamic scene v2 must be an object"))?;

    let entities = object
        .get_mut("entities")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            MigrateError::invalid_payload("dynamic scene v2 entities must be an array")
        })?;
    for entity in entities {
        let components = entity
            .get_mut("components")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                MigrateError::invalid_payload("dynamic scene v2 entity components must be an array")
            })?;
        for component in components {
            migrate_reflected_fields(component, "component")?;
        }
    }

    let resources = object
        .get_mut("resources")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            MigrateError::invalid_payload("dynamic scene v2 resources must be an array")
        })?;
    for resource in resources {
        migrate_reflected_fields(resource, "resource")?;
    }
    Ok(value)
}

fn migrate_reflected_fields(value: &mut Value, kind: &str) -> Result<(), MigrateError> {
    let object = value.as_object_mut().ok_or_else(|| {
        MigrateError::invalid_payload(format!("dynamic scene v2 {kind} must be an object"))
    })?;
    let type_path = object
        .get("type_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            MigrateError::invalid_payload(format!(
                "dynamic scene v2 {kind} type_path must be a string"
            ))
        })?
        .to_string();
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            MigrateError::invalid_payload(format!(
                "dynamic scene v2 {kind} fields must be an array"
            ))
        })?;
    for field in fields {
        let field = field.as_object_mut().ok_or_else(|| {
            MigrateError::invalid_payload(format!(
                "dynamic scene v2 {kind} field must be an object"
            ))
        })?;
        let field_name = field
            .get("field_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                MigrateError::invalid_payload(format!(
                    "dynamic scene v2 {kind} field_name must be a string"
                ))
            })?
            .to_string();
        field.insert(
            "field_id".to_string(),
            Value::String(ReflectFieldId::from_stable_keys(&type_path, &field_name).to_string()),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{migrate_dynamic_scene_v2_to_v3, ReflectFieldId};

    #[test]
    fn v2_migration_assigns_stable_ids_to_component_and_resource_fields() {
        let component_type = "tests.Component.Legacy";
        let resource_type = "tests.Resource.Legacy";
        let migrated = migrate_dynamic_scene_v2_to_v3(json!({
            "entities": [{
                "components": [{
                    "type_path": component_type,
                    "fields": [{ "field_name": "enabled" }]
                }]
            }],
            "resources": [{
                "type_path": resource_type,
                "fields": [{ "field_name": "value" }]
            }]
        }))
        .expect("valid v2 reflected fields should migrate");
        let component_id = ReflectFieldId::from_stable_keys(component_type, "enabled").to_string();
        let resource_id = ReflectFieldId::from_stable_keys(resource_type, "value").to_string();

        assert_eq!(
            migrated["entities"][0]["components"][0]["fields"][0]["field_id"].as_str(),
            Some(component_id.as_str())
        );
        assert_eq!(
            migrated["resources"][0]["fields"][0]["field_id"].as_str(),
            Some(resource_id.as_str())
        );
    }
}
