#[test]
fn legacy_project_world_migration_consumes_owned_maps_and_entity_ids() {
    let source = include_str!("../project_world.rs");
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
