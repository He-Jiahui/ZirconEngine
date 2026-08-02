use std::collections::BTreeSet;

use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectTypeRegistration};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::components::{Mobility, NodeRecord, SceneNode, default_render_layer_mask};
use crate::scene::{EntityId, World};

use super::DynamicScene;
use crate::scene::dynamic_scene::{
    DynamicComponent, DynamicEntity, DynamicResource, DynamicSceneError,
};

pub(super) fn dynamic_scene_from_world(world: &World) -> Result<DynamicScene, DynamicSceneError> {
    let entities = world
        .node_records()
        .into_iter()
        .map(|node| dynamic_entity_from_node(world, node))
        .collect::<Result<Vec<_>, _>>()?;

    let component_types = component_type_descriptors_from_world(world, &entities);
    let mut resources = reflected_resources_from_world(world)?;
    resources.sort_by(|left, right| left.type_path.cmp(&right.type_path));

    Ok(DynamicScene {
        payload_header: crate::scene::dynamic_scene::document::current_dynamic_scene_header(),
        component_types,
        entities,
        resources,
    })
}

fn dynamic_entity_from_node(
    world: &World,
    node: SceneNode,
) -> Result<DynamicEntity, DynamicSceneError> {
    let record = node_record_from_scene_node(world, node);
    let components = reflected_components_for_entity(world, record.id)?;
    Ok(DynamicEntity::new(record.id, record, components))
}

fn component_type_descriptors_from_world(
    world: &World,
    entities: &[DynamicEntity],
) -> Vec<ComponentTypeDescriptor> {
    let mut required_type_ids = BTreeSet::new();
    for entity in entities {
        for component in &entity.components {
            if component.plugin_owned {
                required_type_ids.insert(component.type_path.as_str());
            }
        }
    }

    let mut descriptors = Vec::with_capacity(required_type_ids.len());
    for descriptor in world.component_type_descriptors() {
        if required_type_ids.contains(descriptor.type_id.as_str()) {
            descriptors.push(descriptor.clone());
        }
    }
    descriptors.sort_by(|left, right| left.type_id.cmp(&right.type_id));
    descriptors
}

fn node_record_from_scene_node(world: &World, node: SceneNode) -> NodeRecord {
    NodeRecord {
        id: node.id,
        name: node.name,
        kind: node.kind,
        parent: node.parent,
        transform: node.transform,
        camera: node.camera,
        mesh: node.mesh,
        sprite_2d: node.sprite_2d,
        mesh_2d: node.mesh_2d,
        ambient_light: node.ambient_light,
        directional_light: node.directional_light,
        point_light: node.point_light,
        rect_light: node.rect_light,
        spot_light: node.spot_light,
        active: world.active_self(node.id).unwrap_or(true),
        render_layer_mask: world
            .render_layer_mask(node.id)
            .unwrap_or_else(default_render_layer_mask),
        mobility: world.mobility(node.id).unwrap_or(Mobility::Dynamic),
        rigid_body: node.rigid_body,
        collider: node.collider,
        joint: node.joint,
        animation_skeleton: node.animation_skeleton,
        animation_player: node.animation_player,
        animation_sequence_player: node.animation_sequence_player,
        animation_graph_player: node.animation_graph_player,
        animation_state_machine_player: node.animation_state_machine_player,
    }
}

fn reflected_components_for_entity(
    world: &World,
    entity: EntityId,
) -> Result<Vec<DynamicComponent>, DynamicSceneError> {
    let mut components = Vec::new();
    for runtime in world.type_registry().iter() {
        let metadata = &runtime.registration;
        if !metadata.is_component || !metadata.serializable {
            continue;
        }
        let Some(adapter) = &runtime.component else {
            continue;
        };
        if !adapter.contains(world, entity) {
            continue;
        }
        let fields = serializable_fields(metadata, adapter.read_fields(world, entity)?);
        components.push(DynamicComponent::new(
            metadata.type_path.type_path.clone(),
            metadata.plugin_owned,
            fields,
        ));
    }
    components.sort_by(|left, right| left.type_path.cmp(&right.type_path));
    Ok(components)
}

fn reflected_resources_from_world(
    world: &World,
) -> Result<Vec<DynamicResource>, DynamicSceneError> {
    let mut resources = Vec::new();
    for runtime in world.type_registry().iter() {
        let metadata = &runtime.registration;
        if !metadata.is_resource || !metadata.serializable {
            continue;
        }
        let Some(adapter) = runtime.resource else {
            continue;
        };
        if !adapter.contains(world) {
            continue;
        }
        let fields = serializable_fields(metadata, adapter.read_fields(world)?);
        resources.push(DynamicResource::new(
            metadata.type_path.type_path.clone(),
            fields,
        ));
    }
    Ok(resources)
}

fn serializable_fields(
    metadata: &ReflectTypeRegistration,
    fields: Vec<ReflectFieldValue>,
) -> Vec<ReflectFieldValue> {
    fields
        .into_iter()
        .filter(|field| {
            metadata
                .type_info
                .fields
                .iter()
                .any(|info| info.name == field.field_name && info.serializable)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_reuses_node_record_order_without_resorting_entities() {
        let source = include_str!("capture.rs");
        let capture = source
            .split("pub(super) fn dynamic_scene_from_world")
            .nth(1)
            .and_then(|source| source.split("fn dynamic_entity_from_node").next())
            .expect("read dynamic scene capture body");

        assert!(capture.contains(".node_records()"));
        assert!(
            !capture.contains("entities.sort_by_key"),
            "World::node_records already publishes entity-id order; capture must not sort it again"
        );
    }
}
