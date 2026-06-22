use std::collections::BTreeSet;

use zircon_runtime_interface::reflect::{ReflectError, ReflectFieldInfo};

use crate::scene::components::NodeRecord;
use crate::scene::{EntityId, World};

use super::DynamicScene;
use crate::scene::dynamic_scene::value::{reflected_fields_to_json_object, remap_reflected_value};
use crate::scene::dynamic_scene::{
    DynamicComponent, DynamicResource, DynamicSceneError, EntityRemap,
    ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap, ScenePatchPreviewReport,
    ScenePatchPreviewResource,
};

pub(super) fn spawn_scene_into(
    scene: &DynamicScene,
    world: &mut World,
) -> Result<EntityRemap, DynamicSceneError> {
    scene.ensure_supported()?;

    install_component_type_descriptors(scene, world)?;
    let remap = build_entity_remap(scene, world)?;
    insert_entity_records(scene, world, &remap)?;
    apply_components(scene, world, &remap)?;
    apply_resources(scene, world, &remap)?;
    Ok(remap)
}

pub(super) fn preview_scene_spawn_into(
    scene: &DynamicScene,
    world: &World,
) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
    scene.ensure_supported()?;
    ensure_component_type_descriptors_are_compatible(scene, world)?;
    let remap = build_entity_remap(scene, world)?;
    validate_remapped_parents(scene, world, &remap)?;
    validate_components_are_previewable(scene, world, &remap)?;
    let resources = preview_resources(scene, world, &remap)?;
    let component_type_count = scene.component_types.len();
    let existing_component_type_count = scene
        .component_types
        .iter()
        .filter(|descriptor| {
            world
                .component_type_descriptor(&descriptor.type_id)
                .is_some()
        })
        .count();
    let new_component_type_count =
        component_type_count.saturating_sub(existing_component_type_count);
    let component_types = scene
        .component_types
        .iter()
        .map(|descriptor| ScenePatchPreviewComponentType {
            type_id: descriptor.type_id.clone(),
            plugin_id: descriptor.plugin_id.clone(),
            display_name: descriptor.display_name.clone(),
            already_registered: world
                .component_type_descriptor(&descriptor.type_id)
                .is_some(),
        })
        .collect();
    let remapped_entity_count = remap
        .iter()
        .filter(|(source, target)| source != target)
        .count();
    let entity_remaps = remap
        .iter()
        .map(
            |(source_entity, target_entity)| ScenePatchPreviewEntityRemap {
                source_entity,
                target_entity,
            },
        )
        .collect();
    let entity_count = scene.entities.len();
    let component_instance_count: usize = scene
        .entities
        .iter()
        .map(|entity| entity.components.len())
        .sum();
    Ok(ScenePatchPreviewReport {
        component_type_count,
        existing_component_type_count,
        new_component_type_count,
        component_instance_count,
        entity_count,
        resource_count: scene.resources.len(),
        target_entity_count: world.node_records().len(),
        preserved_entity_count: entity_count.saturating_sub(remapped_entity_count),
        remapped_entity_count,
        component_types,
        resources,
        entity_remaps,
    })
}

fn install_component_type_descriptors(
    scene: &DynamicScene,
    world: &mut World,
) -> Result<(), DynamicSceneError> {
    for descriptor in &scene.component_types {
        if let Some(existing) = world.component_type_descriptor(&descriptor.type_id) {
            if existing != descriptor {
                return Err(DynamicSceneError::ComponentTypeDescriptorConflict {
                    type_id: descriptor.type_id.clone(),
                });
            }
            continue;
        }
        world.register_component_type(descriptor.clone())?;
    }
    Ok(())
}

fn ensure_component_type_descriptors_are_compatible(
    scene: &DynamicScene,
    world: &World,
) -> Result<(), DynamicSceneError> {
    for descriptor in &scene.component_types {
        if let Some(existing) = world.component_type_descriptor(&descriptor.type_id) {
            if existing != descriptor {
                return Err(DynamicSceneError::ComponentTypeDescriptorConflict {
                    type_id: descriptor.type_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_remapped_parents(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    for entity in &scene.entities {
        remapped_parent(world, remap, entity.source_entity, entity.record.parent)?;
    }
    Ok(())
}

fn validate_components_are_previewable(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    for entity in &scene.entities {
        for component in &entity.components {
            validate_component_is_previewable(world, component, remap)?;
        }
    }
    Ok(())
}

fn validate_component_is_previewable(
    world: &World,
    component: &DynamicComponent,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    if component.plugin_owned {
        let _ = reflected_fields_to_json_object(&component.fields, remap)?;
        return Ok(());
    }

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
    if runtime.component.is_none() {
        return Err(ReflectError::NoComponentAdapter {
            type_path: component.type_path.clone(),
        }
        .into());
    }

    for field in &component.fields {
        let _ = should_write_field(
            &component.type_path,
            &runtime.registration.type_info.fields,
            &field.field_name,
        )?;
        let _ = remap_reflected_value(&field.value, remap)?;
    }
    Ok(())
}

fn preview_resources(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<Vec<ScenePatchPreviewResource>, DynamicSceneError> {
    let mut preview_resources = Vec::with_capacity(scene.resources.len());
    for resource in &scene.resources {
        preview_resources.push(preview_resource(world, resource, remap)?);
    }
    Ok(preview_resources)
}

fn preview_resource(
    world: &World,
    resource: &DynamicResource,
    remap: &EntityRemap,
) -> Result<ScenePatchPreviewResource, DynamicSceneError> {
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
    let adapter = runtime
        .resource
        .ok_or_else(|| ReflectError::NoResourceAdapter {
            type_path: resource.type_path.clone(),
        })?;
    let already_present = adapter.contains(world);
    let can_create_on_apply = adapter.ensure.is_some();
    if !already_present && !can_create_on_apply {
        return Err(ReflectError::MissingResource {
            type_path: resource.type_path.clone(),
        }
        .into());
    }

    for field in &resource.fields {
        let _ = should_write_field(
            &resource.type_path,
            &runtime.registration.type_info.fields,
            &field.field_name,
        )?;
        let _ = remap_reflected_value(&field.value, remap)?;
    }
    Ok(ScenePatchPreviewResource {
        type_path: resource.type_path.clone(),
        already_present,
        can_create_on_apply,
        field_count: resource.fields.len(),
    })
}

fn build_entity_remap(
    scene: &DynamicScene,
    world: &World,
) -> Result<EntityRemap, DynamicSceneError> {
    let mut remap = EntityRemap::new();
    let mut reserved = BTreeSet::new();
    for entity in &scene.entities {
        let target = first_available_entity_id(world, &reserved, entity.source_entity)?;
        reserved.insert(target);
        remap.insert(entity.source_entity, target);
    }
    Ok(remap)
}

fn insert_entity_records(
    scene: &DynamicScene,
    world: &mut World,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    for entity in &scene.entities {
        let mut record = entity.record.clone();
        let target = remap
            .get(entity.source_entity)
            .expect("validated entity remap must cover every scene entity");
        record.id = target;
        record.parent = remapped_parent(world, remap, entity.source_entity, record.parent)?;
        remap_record_entity_references(&mut record, remap);
        world.insert_node_record(record)?;
    }
    Ok(())
}

fn apply_components(
    scene: &DynamicScene,
    world: &mut World,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    for entity in &scene.entities {
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
    scene: &DynamicScene,
    world: &mut World,
    remap: &EntityRemap,
) -> Result<(), DynamicSceneError> {
    for resource in &scene.resources {
        apply_resource(world, resource, remap)?;
    }
    Ok(())
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
        if let Some(binding) = &mut joint.skeleton_binding {
            binding.skeleton_entity = remap
                .get(binding.skeleton_entity)
                .unwrap_or(binding.skeleton_entity);
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
        world.set_dynamic_component(entity, component.type_path.clone(), value)?;
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

    ensure_reflected_resource_exists(world, resource, &adapter)?;
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

fn ensure_reflected_resource_exists(
    world: &mut World,
    resource: &DynamicResource,
    adapter: &crate::scene::reflect::ReflectResource,
) -> Result<(), ReflectError> {
    if adapter.contains(world) {
        return Ok(());
    }
    let _ = adapter.ensure(world)?;
    if adapter.contains(world) {
        return Ok(());
    }
    Err(ReflectError::MissingResource {
        type_path: resource.type_path.clone(),
    })
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
