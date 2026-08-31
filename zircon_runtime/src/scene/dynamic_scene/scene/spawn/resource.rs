use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectedValue,
};

use crate::scene::dynamic_scene::value::remap_reflected_value;
use crate::scene::dynamic_scene::{
    DynamicResource, DynamicScene, DynamicSceneError, EntityRemap, ScenePatchPreviewResource,
};
use crate::scene::reflect::validate_reflected_field_values;
use crate::scene::World;

/// The compiled resource half of a scene spawn. Field slots are resolved while
/// the scene plan is built, so commit performs no resource schema lookup.
pub(super) struct CompiledResourceWrite {
    type_path: String,
    pub(super) adapter: crate::scene::reflect::ReflectResource,
    writes: Vec<(u32, ReflectedValue)>,
}

pub(super) fn compile_resource_writes(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<(Vec<CompiledResourceWrite>, Vec<ScenePatchPreviewResource>), DynamicSceneError> {
    let mut writes = Vec::with_capacity(scene.resources.len());
    let mut previews = Vec::with_capacity(scene.resources.len());
    for resource in &scene.resources {
        let (write, preview) = compile_resource_write(world, resource, remap)?;
        writes.push(write);
        previews.push(preview);
    }
    Ok((writes, previews))
}

fn compile_resource_write(
    world: &World,
    resource: &DynamicResource,
    remap: &EntityRemap,
) -> Result<(CompiledResourceWrite, ScenePatchPreviewResource), DynamicSceneError> {
    validate_reflected_field_values(&resource.type_path, &resource.fields)?;
    let runtime = world
        .type_registry()
        .runtime_registration(&resource.type_path)?;
    if !runtime.registration.is_resource() {
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

    let writes = compile_reflected_writes(
        world,
        &resource.type_path,
        &runtime.registration.type_info.fields,
        &resource.fields,
        remap,
    )?;

    Ok((
        CompiledResourceWrite {
            type_path: resource.type_path.clone(),
            adapter,
            writes,
        },
        ScenePatchPreviewResource {
            type_path: resource.type_path.clone(),
            already_present,
            can_create_on_apply,
            field_count: resource.fields.len(),
        },
    ))
}

pub(super) fn compile_reflected_writes(
    world: &World,
    type_path: &str,
    schema_fields: &[ReflectFieldInfo],
    fields: &[ReflectFieldValue],
    remap: &EntityRemap,
) -> Result<Vec<(u32, ReflectedValue)>, DynamicSceneError> {
    let mut writes = Vec::with_capacity(fields.len());
    let mut seen_slots = vec![false; schema_fields.len()];
    let mut next_schema_slot = 0usize;
    for field in fields {
        let field_slot = if schema_fields
            .get(next_schema_slot)
            .is_some_and(|schema| schema.id == field.field_id)
        {
            u32::try_from(next_schema_slot).map_err(|_| ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason: "reflection schema has more than u32::MAX fields".to_string(),
            })?
        } else {
            world
                .type_registry()
                .resolve_field_slot_by_id(type_path, field.field_id)?
        };
        let schema = schema_fields
            .get(field_slot as usize)
            .filter(|schema| schema.id == field.field_id)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field.field_id.to_string(),
            })?;
        if std::mem::replace(&mut seen_slots[field_slot as usize], true) {
            return Err(ReflectError::InvalidValue {
                type_path: type_path.to_string(),
                field_name: field.field_id.to_string(),
                reason: "duplicate stable field identity in dynamic scene payload".to_string(),
            }
            .into());
        }
        next_schema_slot = (field_slot as usize).saturating_add(1);
        if schema.serializable && schema.editable {
            writes.push((field_slot, remap_reflected_value(&field.value, remap)?));
        }
    }
    Ok(writes)
}

pub(super) fn apply_resource_writes_to_preflight(
    world: &mut World,
    writes: Vec<CompiledResourceWrite>,
) -> Result<(), DynamicSceneError> {
    crate::profile_scope!(
        "runtime",
        "dynamic_scene.transaction",
        "resource_adapter_apply"
    );
    let resource_adapter_ensure_calls = writes.len();
    let resource_adapter_write_calls = writes
        .iter()
        .filter(|write| !write.writes.is_empty())
        .count();
    let resource_field_writes = writes.iter().fold(0_usize, |count, write| {
        count.saturating_add(write.writes.len())
    });
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.resource_adapter.ensure_calls",
        resource_adapter_ensure_calls
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.resource_adapter.write_fields_calls",
        resource_adapter_write_calls
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.resource_adapter.field_writes",
        resource_field_writes
    );
    for write in writes {
        ensure_reflected_resource_exists(world, &write.type_path, write.adapter)?;
        if !write.writes.is_empty() {
            write.adapter.write_fields_by_slot(world, write.writes)?;
        }
    }
    Ok(())
}

/// Detaches only the resources named by this scene from the preflight World.
/// Every adapter has already validated and written its fields there; target
/// commit receives these owned rows without calling an adapter a second time.
pub(super) fn transfer_preflight_resource_writes(
    source: &mut World,
    artifact: &mut World,
    adapters: &[crate::scene::reflect::ReflectResource],
) -> Result<(), DynamicSceneError> {
    crate::profile_scope!(
        "runtime",
        "dynamic_scene.transaction",
        "resource_preflight_transfer"
    );
    for adapter in adapters {
        adapter.transfer_preflight(source, artifact)?;
    }
    Ok(())
}

/// Stages exactly the resource adapters captured by a compiled spawn plan.
/// Once compilation succeeds, later preflight phases never consult a mutable
/// DynamicScene again.
pub(super) fn stage_compiled_resource_writes_bounded(
    source: &World,
    target: &mut World,
    writes: &[CompiledResourceWrite],
    base_estimated_bytes: usize,
    limit_bytes: usize,
) -> Result<usize, DynamicSceneError> {
    let mut estimated_bytes = base_estimated_bytes;
    for write in writes {
        stage_existing_resource_bounded(
            source,
            target,
            &write.type_path,
            write.adapter,
            &mut estimated_bytes,
            limit_bytes,
        )?;
    }
    Ok(estimated_bytes)
}

pub(in crate::scene::dynamic_scene::scene) fn stage_existing_resources_bounded(
    scene: &DynamicScene,
    source: &World,
    target: &mut World,
    base_estimated_bytes: usize,
    limit_bytes: usize,
) -> Result<usize, DynamicSceneError> {
    let mut estimated_bytes = base_estimated_bytes;
    for resource in &scene.resources {
        let runtime = source
            .type_registry()
            .runtime_registration(&resource.type_path)?;
        let adapter = runtime
            .resource
            .ok_or_else(|| ReflectError::NoResourceAdapter {
                type_path: resource.type_path.clone(),
            })?;
        stage_existing_resource_bounded(
            source,
            target,
            &resource.type_path,
            adapter,
            &mut estimated_bytes,
            limit_bytes,
        )?;
    }
    Ok(estimated_bytes)
}

fn stage_existing_resource_bounded(
    source: &World,
    target: &mut World,
    type_path: &str,
    adapter: crate::scene::reflect::ReflectResource,
    estimated_bytes: &mut usize,
    limit_bytes: usize,
) -> Result<(), DynamicSceneError> {
    if !adapter.contains(source) {
        return Ok(());
    }
    let resource_bytes = adapter.estimate_stage_clone_bytes(source)?.ok_or_else(|| {
        DynamicSceneError::MissingResourceStagingSizeEstimate {
            type_path: type_path.to_string(),
        }
    })?;
    *estimated_bytes = estimated_bytes.saturating_add(resource_bytes);
    if *estimated_bytes > limit_bytes {
        return Err(DynamicSceneError::TargetSnapshotTooLarge {
            estimated_bytes: *estimated_bytes,
            limit_bytes,
        });
    }
    if !adapter.stage_clone(source, target)? {
        return Err(DynamicSceneError::MissingResourceStagingClone {
            type_path: type_path.to_string(),
        });
    }
    Ok(())
}

fn ensure_reflected_resource_exists(
    world: &mut World,
    type_path: &str,
    adapter: crate::scene::reflect::ReflectResource,
) -> Result<(), ReflectError> {
    if adapter.contains(world) {
        return Ok(());
    }
    let _ = adapter.ensure(world)?;
    if adapter.contains(world) {
        return Ok(());
    }
    Err(ReflectError::MissingResource {
        type_path: type_path.to_string(),
    })
}
