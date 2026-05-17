use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectTypeRegistration,
};

use crate::scene::components::{default_render_layer_mask, Mobility, NodeRecord, SceneNode};
use crate::scene::{EntityId, World};

use super::entity::{DynamicComponent, DynamicEntity, DynamicResource};
use super::value::{reflected_fields_to_json_object, remap_reflected_value};
use super::{DynamicSceneError, EntityRemap};

pub const DYNAMIC_SCENE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicScene {
    pub format_version: u32,
    #[serde(default)]
    pub entities: Vec<DynamicEntity>,
    #[serde(default)]
    pub resources: Vec<DynamicResource>,
}

impl DynamicScene {
    pub fn empty() -> Self {
        Self {
            format_version: DYNAMIC_SCENE_FORMAT_VERSION,
            entities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_world(world: &World) -> Result<Self, DynamicSceneError> {
        let mut entities = world
            .node_records()
            .into_iter()
            .map(|node| dynamic_entity_from_node(world, node))
            .collect::<Result<Vec<_>, _>>()?;
        entities.sort_by_key(|entity| entity.source_entity);

        let mut resources = reflected_resources_from_world(world)?;
        resources.sort_by(|left, right| left.type_path.cmp(&right.type_path));

        Ok(Self {
            format_version: DYNAMIC_SCENE_FORMAT_VERSION,
            entities,
            resources,
        })
    }

    pub fn spawn_into(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.ensure_supported_version()?;
        self.ensure_unique_sources()?;

        let remap = self.build_entity_remap(world)?;
        self.insert_entity_records(world, &remap)?;
        self.apply_components(world, &remap)?;
        self.apply_resources(world, &remap)?;
        Ok(remap)
    }

    pub(super) fn ensure_supported_version(&self) -> Result<(), DynamicSceneError> {
        if self.format_version == DYNAMIC_SCENE_FORMAT_VERSION {
            return Ok(());
        }
        Err(DynamicSceneError::UnsupportedFormatVersion {
            expected: DYNAMIC_SCENE_FORMAT_VERSION,
            actual: self.format_version,
        })
    }

    fn ensure_unique_sources(&self) -> Result<(), DynamicSceneError> {
        let mut seen = BTreeSet::new();
        for entity in &self.entities {
            if !seen.insert(entity.source_entity) {
                return Err(DynamicSceneError::DuplicateSourceEntity {
                    entity: entity.source_entity,
                });
            }
        }
        Ok(())
    }

    fn build_entity_remap(&self, world: &World) -> Result<EntityRemap, DynamicSceneError> {
        let mut remap = EntityRemap::new();
        let mut reserved = BTreeSet::new();
        for entity in &self.entities {
            let target = first_available_entity_id(world, &reserved, entity.source_entity)?;
            reserved.insert(target);
            remap.insert(entity.source_entity, target);
        }
        Ok(remap)
    }

    fn insert_entity_records(
        &self,
        world: &mut World,
        remap: &EntityRemap,
    ) -> Result<(), DynamicSceneError> {
        for entity in &self.entities {
            let mut record = entity.record.clone();
            let target = remap
                .get(entity.source_entity)
                .expect("validated entity remap must cover every scene entity");
            record.id = target;
            record.parent = remapped_parent(world, remap, entity.source_entity, record.parent)?;
            remap_record_entity_references(&mut record, remap);
            world
                .insert_node_record(record)
                .map_err(DynamicSceneError::WorldMutation)?;
        }
        Ok(())
    }

    fn apply_components(
        &self,
        world: &mut World,
        remap: &EntityRemap,
    ) -> Result<(), DynamicSceneError> {
        for entity in &self.entities {
            let target = remap
                .get(entity.source_entity)
                .expect("validated entity remap must cover every scene entity");
            for component in &entity.components {
                apply_component(world, target, component, remap)?;
            }
        }
        Ok(())
    }

    fn apply_resources(
        &self,
        world: &mut World,
        remap: &EntityRemap,
    ) -> Result<(), DynamicSceneError> {
        for resource in &self.resources {
            apply_resource(world, resource, remap)?;
        }
        Ok(())
    }
}

fn dynamic_entity_from_node(
    world: &World,
    node: SceneNode,
) -> Result<DynamicEntity, DynamicSceneError> {
    let record = node_record_from_scene_node(world, node);
    let components = reflected_components_for_entity(world, record.id)?;
    Ok(DynamicEntity::new(record.id, record, components))
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
        directional_light: node.directional_light,
        point_light: node.point_light,
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

fn first_available_entity_id(
    world: &World,
    reserved: &BTreeSet<EntityId>,
    source: EntityId,
) -> Result<EntityId, DynamicSceneError> {
    let mut candidate = source;
    loop {
        if !world.contains_entity(candidate) && !reserved.contains(&candidate) {
            return Ok(candidate);
        }
        candidate = candidate
            .checked_add(1)
            .ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                source_entity: source,
            })?;
    }
}

fn remapped_parent(
    world: &World,
    remap: &EntityRemap,
    entity: EntityId,
    parent: Option<EntityId>,
) -> Result<Option<EntityId>, DynamicSceneError> {
    let Some(parent) = parent else {
        return Ok(None);
    };
    if let Some(parent) = remap.get(parent) {
        return Ok(Some(parent));
    }
    if world.contains_entity(parent) {
        return Ok(Some(parent));
    }
    Err(DynamicSceneError::MissingSceneParent { entity, parent })
}

fn remap_record_entity_references(record: &mut NodeRecord, remap: &EntityRemap) {
    if let Some(joint) = &mut record.joint {
        if let Some(entity) = joint.connected_entity {
            joint.connected_entity = Some(remap.get(entity).unwrap_or(entity));
        }
    }
}

fn apply_component(
    world: &mut World,
    entity: EntityId,
    component: &DynamicComponent,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    if component.plugin_owned {
        let value = reflected_fields_to_json_object(&component.fields, remap)?;
        world
            .set_dynamic_component(entity, component.type_path.clone(), value)
            .map_err(DynamicSceneError::WorldMutation)?;
        return Ok(());
    }

    let (adapter, field_info) = {
        let runtime = world
            .type_registry()
            .runtime_registration(&component.type_path)?;
        if !runtime.registration.is_component {
            return Err(ReflectError::AddressKindMismatch {
                expected: format!("component `{}`", component.type_path),
                actual: format!("non-component `{}`", component.type_path),
            }
            .into());
        }
        (
            runtime
                .component
                .clone()
                .ok_or_else(|| ReflectError::NoComponentAdapter {
                    type_path: component.type_path.clone(),
                })?,
            runtime.registration.type_info.fields.clone(),
        )
    };

    for field in &component.fields {
        if should_write_field(&component.type_path, &field_info, &field.field_name)? {
            adapter.write_field(
                world,
                entity,
                &field.field_name,
                remap_reflected_value(&field.value, remap)?,
            )?;
        }
    }
    Ok(())
}

fn apply_resource(
    world: &mut World,
    resource: &DynamicResource,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    let (adapter, field_info) = {
        let runtime = world
            .type_registry()
            .runtime_registration(&resource.type_path)?;
        if !runtime.registration.is_resource {
            return Err(ReflectError::AddressKindMismatch {
                expected: format!("resource `{}`", resource.type_path),
                actual: format!("non-resource `{}`", resource.type_path),
            }
            .into());
        }
        (
            runtime
                .resource
                .ok_or_else(|| ReflectError::NoResourceAdapter {
                    type_path: resource.type_path.clone(),
                })?,
            runtime.registration.type_info.fields.clone(),
        )
    };

    for field in &resource.fields {
        if should_write_field(&resource.type_path, &field_info, &field.field_name)? {
            adapter.write_field(
                world,
                &field.field_name,
                remap_reflected_value(&field.value, remap)?,
            )?;
        }
    }
    Ok(())
}

fn should_write_field(
    type_path: &str,
    fields: &[ReflectFieldInfo],
    field_name: &str,
) -> Result<bool, ReflectError> {
    let Some(field) = fields.iter().find(|field| field.name == field_name) else {
        return Err(ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: field_name.to_string(),
        });
    };
    Ok(field.serializable && field.editable)
}
