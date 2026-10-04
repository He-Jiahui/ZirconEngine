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
