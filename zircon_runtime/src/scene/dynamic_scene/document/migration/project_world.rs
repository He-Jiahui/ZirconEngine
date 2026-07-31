use serde_json::{json, Map, Value};
use zircon_runtime_interface::serialization::MigrateError;

use crate::scene::components::default_render_layer_mask;

const OPTIONAL_RECORD_MAPS: [(&str, &str); 17] = [
    ("cameras", "camera"),
    ("mesh_renderers", "mesh"),
    ("sprite_2d", "sprite_2d"),
    ("mesh_2d", "mesh_2d"),
    ("ambient_lights", "ambient_light"),
    ("directional_lights", "directional_light"),
    ("point_lights", "point_light"),
    ("rect_lights", "rect_light"),
    ("spot_lights", "spot_light"),
    ("rigid_bodies", "rigid_body"),
    ("colliders", "collider"),
    ("joints", "joint"),
    ("animation_skeletons", "animation_skeleton"),
    ("animation_players", "animation_player"),
    ("animation_sequence_players", "animation_sequence_player"),
    ("animation_graph_players", "animation_graph_player"),
    (
        "animation_state_machine_players",
        "animation_state_machine_player",
    ),
];

pub(super) fn migrate_project_world(value: Value) -> Result<Value, MigrateError> {
    let mut document = into_object(value, "dynamic scene v0 project document")?;
    let world = document
        .remove("world")
        .ok_or_else(|| MigrateError::invalid_payload("project document is missing world"))?;
    dynamic_scene_from_world_value(world)
}

fn dynamic_scene_from_world_value(world: Value) -> Result<Value, MigrateError> {
    let mut world = into_object(world, "dynamic scene v0 project world")?;
    let entities = world
        .remove("entities")
        .ok_or_else(|| MigrateError::invalid_payload("project world is missing entities"))?;
    let mut ids = into_array(entities, "project world entities")?;
    ids.sort_by_key(|value| value.as_u64().unwrap_or(u64::MAX));
    let entities = ids
        .into_iter()
        .map(|id| dynamic_entity_from_world(&world, id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "entities": entities,
        "resources": []
    }))
}

fn dynamic_entity_from_world(world: &Map<String, Value>, id: Value) -> Result<Value, MigrateError> {
    let id_number = id
        .as_u64()
        .ok_or_else(|| MigrateError::invalid_payload("project world entity id must be unsigned"))?;
    let key = id_number.to_string();
    let mut record = Map::new();
    record.insert("id".to_string(), Value::from(id_number));
    record.insert(
        "name".to_string(),
        required_map_value(world, "names", &key)?.clone(),
    );
    record.insert(
        "kind".to_string(),
        map_value(world, "kinds", &key).unwrap_or_else(|| Value::String("Empty".to_string())),
    );
    record.insert(
        "parent".to_string(),
        map_value(world, "hierarchy", &key)
            .and_then(|value| value.get("parent").cloned())
            .unwrap_or(Value::Null),
    );
    record.insert(
        "transform".to_string(),
        map_value(world, "local_transforms", &key)
            .and_then(|value| value.get("transform").cloned())
            .unwrap_or_else(identity_transform),
    );
    for (source, target) in OPTIONAL_RECORD_MAPS {
        record.insert(
            target.to_string(),
            map_value(world, source, &key).unwrap_or(Value::Null),
        );
    }
    record.insert(
        "active".to_string(),
        map_value(world, "active", &key).unwrap_or(Value::Bool(true)),
    );
    record.insert(
        "render_layer_mask".to_string(),
        map_value(world, "render_layer_masks", &key)
            .unwrap_or_else(|| Value::from(default_render_layer_mask())),
    );
    record.insert(
        "mobility".to_string(),
        map_value(world, "mobility", &key).unwrap_or_else(|| Value::String("Dynamic".to_string())),
    );

    Ok(json!({
        "source_entity": id_number,
        "record": Value::Object(record),
        "components": []
    }))
}

fn into_object(value: Value, label: &str) -> Result<Map<String, Value>, MigrateError> {
    match value {
        Value::Object(object) => Ok(object),
        _ => Err(MigrateError::invalid_payload(format!(
            "{label} must be an object"
        ))),
    }
}

fn into_array(value: Value, label: &str) -> Result<Vec<Value>, MigrateError> {
    match value {
        Value::Array(values) => Ok(values),
        _ => Err(MigrateError::invalid_payload(format!(
            "{label} must be an array"
        ))),
    }
}

fn required_map_value<'a>(
    world: &'a Map<String, Value>,
    map_name: &str,
    key: &str,
) -> Result<&'a Value, MigrateError> {
    world
        .get(map_name)
        .and_then(Value::as_object)
        .and_then(|values| values.get(key))
        .ok_or_else(|| {
            MigrateError::invalid_payload(format!(
                "project world {map_name} is missing entity {key}"
            ))
        })
}

fn map_value(world: &Map<String, Value>, map_name: &str, key: &str) -> Option<Value> {
    world
        .get(map_name)
        .and_then(Value::as_object)
        .and_then(|values| values.get(key))
        .cloned()
}

fn identity_transform() -> Value {
    json!({
        "translation": [0.0, 0.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn legacy_project_world_migration_consumes_owned_maps_and_entity_ids() {
        let source = include_str!("project_world.rs");
        let migration = source
            .split("pub(super) fn migrate_project_world")
            .nth(1)
            .and_then(|source| source.split("#[cfg(test)]").next())
            .expect("read project-world migration body");
        let world_projection = migration
            .split("fn dynamic_scene_from_world_value")
            .nth(1)
            .and_then(|source| source.split("fn dynamic_entity_from_world").next())
            .expect("read owned world projection");
        let object_conversion = migration
            .split("fn into_object")
            .nth(1)
            .and_then(|source| source.split("fn into_array").next())
            .expect("read owned object conversion");
        let array_conversion = migration
            .split("fn into_array")
            .nth(1)
            .and_then(|source| source.split("fn required_map_value").next())
            .expect("read owned array conversion");

        assert!(
            world_projection.contains("let mut world = into_object(")
                && world_projection.contains(".remove(\"entities\")")
                && world_projection.contains("into_array(")
                && !world_projection.contains(".as_array()")
                && !world_projection.contains(".cloned()")
                && object_conversion.contains("Value::Object(object) => Ok(object)")
                && !object_conversion.contains(".as_object()")
                && array_conversion.contains("Value::Array(values) => Ok(values)")
                && !array_conversion.contains(".as_array()"),
            "owned legacy project-world migration must not deep-clone the document, world, or entity-id array"
        );
    }
}
