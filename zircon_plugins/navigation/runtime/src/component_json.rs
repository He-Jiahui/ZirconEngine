use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) fn parse_component<T>(value: &Value) -> T
where
    T: Default + DeserializeOwned,
{
    serde_json::from_value(normalize_scene_property_json(value)).unwrap_or_default()
}

fn normalize_scene_property_json(value: &Value) -> Value {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("resource") => object
            .get("resource")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
        Value::Object(object) if object.len() == 1 && object.contains_key("entity") => {
            object.get("entity").cloned().unwrap_or(Value::Null)
        }
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), normalize_scene_property_json(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(normalize_scene_property_json)
                .collect::<Vec<_>>(),
        ),
        value => value.clone(),
    }
}

/// One current world's parsed agent descriptors, invalidated on component edits or replacement.
pub(crate) fn cached_agent_descriptor(
    world: &zircon_runtime::scene::World,
    entity: u64,
) -> Option<zircon_runtime::core::framework::navigation::NavMeshAgentDescriptor> {
    use std::{cell::RefCell, collections::HashMap};
    use zircon_runtime::core::framework::navigation::{
        NavMeshAgentDescriptor, NAV_MESH_AGENT_COMPONENT_TYPE,
    };
    type Cache = (Option<(u64, u64)>, HashMap<u64, NavMeshAgentDescriptor>);
    thread_local! { static CACHE: RefCell<Cache> = RefCell::new((None, HashMap::new())); }
    let value = world.dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)?;
    let key = (
        world.render_world_snapshot_handle().raw(),
        world.dynamic_component_generation(NAV_MESH_AGENT_COMPONENT_TYPE),
    );
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.0 != Some(key) {
            cache.1.clear();
            cache.0 = Some(key);
        }
        Some(
            cache
                .1
                .entry(entity)
                .or_insert_with(|| parse_component(value))
                .clone(),
        )
    })
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use zircon_runtime::{
        core::framework::navigation::NAV_MESH_AGENT_COMPONENT_TYPE,
        scene::{NodeKind, World},
    };
    #[test]
    fn agent_cache_invalidates_on_component_edits_and_world_replacement() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::json!({"speed": 2.0}),
            )
            .unwrap();
        assert_eq!(cached_agent_descriptor(&world, entity).unwrap().speed, 2.0);
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::json!({"speed": 7.0}),
            )
            .unwrap();
        assert_eq!(cached_agent_descriptor(&world, entity).unwrap().speed, 7.0);
        let mut replacement = World::empty();
        let other = replacement.spawn_node(NodeKind::Empty).unwrap();
        replacement
            .set_dynamic_component(
                other,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::json!({"speed": 9.0}),
            )
            .unwrap();
        assert_eq!(
            cached_agent_descriptor(&replacement, other).unwrap().speed,
            9.0
        );
    }
}
