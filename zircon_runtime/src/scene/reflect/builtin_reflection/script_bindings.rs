//! Preserve authored script bindings in reflected Play snapshots and transactions.
//! The runtime's existing JSON component remains the authoritative storage.

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use crate::asset::assets::SceneScriptBindingAsset;
use crate::scene::{EntityId, ReflectComponent, RuntimeTypeRegistration, World};

use super::component_support;

const TYPE_PATH: &str = SceneScriptBindingAsset::COMPONENT_TYPE_PATH;
const FIELD: &str = "bindings";

pub(super) fn registration() -> Result<RuntimeTypeRegistration, ReflectError> {
    let field = ReflectFieldInfo::new(
        ReflectFieldId::from_stable_keys(TYPE_PATH, FIELD),
        FIELD,
        "Json",
        ReflectEditorHint::None,
    );
    Ok(RuntimeTypeRegistration {
        registration: ReflectTypeRegistration::new(
            ReflectTypePath::new(TYPE_PATH, "bindings")?,
            "Script bindings",
            ReflectTypeInfo::json_with_fields(vec![field]),
            ReflectSerializationStrategy::Json,
        )
        .as_component()
        // Structured Scripts controls own authoring; keep this serialization
        // payload out of the generic reflected JSON inspector.
        .with_editor_visible(false),
        component: Some(
            ReflectComponent::new(TYPE_PATH, contains, read_field, write_field, remove)
                .with_dense_field_slots(read_slot, write_slot)
                .with_dense_field_batch_write(write_slots)
                .with_stage_clone(stage_clone),
        ),
        resource: None,
    })
}

fn contains(world: &World, entity: EntityId, _: &str) -> bool {
    world.contains_entity(entity) && world.dynamic_component(entity, TYPE_PATH).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _: &str,
    name: &str,
) -> Result<ReflectedValue, ReflectError> {
    if name != FIELD {
        return Err(component_support::unknown(TYPE_PATH, name));
    }
    world
        .dynamic_component(entity, TYPE_PATH)
        .cloned()
        .map(ReflectedValue::Json)
        .ok_or_else(|| component_support::missing(entity, TYPE_PATH))
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _: &str,
    name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if name != FIELD {
        return Err(component_support::unknown(TYPE_PATH, name));
    }
    let ReflectedValue::Json(value) = value else {
        return Err(ReflectError::TypeMismatch {
            type_path: TYPE_PATH.into(),
            field_name: FIELD.into(),
            expected: "Json".into(),
            actual: value.type_name().into(),
        });
    };
    serde_json::from_value::<Vec<SceneScriptBindingAsset>>(value.clone()).map_err(|error| {
        ReflectError::InvalidRegistration {
            type_path: TYPE_PATH.into(),
            reason: format!("invalid script bindings: {error}"),
        }
    })?;
    world
        .set_dynamic_component(entity, TYPE_PATH, value)
        .map_err(|error| error.into_reflect_error(TYPE_PATH.into()))
}

fn check_slot(slot: u32) -> Result<(), ReflectError> {
    if slot == 0 {
        Ok(())
    } else {
        Err(component_support::unknown(TYPE_PATH, &format!("#{slot}")))
    }
}

fn read_slot(
    world: &World,
    entity: EntityId,
    path: &str,
    slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    check_slot(slot)?;
    read_field(world, entity, path, FIELD)
}

fn write_slot(
    world: &mut World,
    entity: EntityId,
    path: &str,
    slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    check_slot(slot)?;
    write_field(world, entity, path, FIELD, value)
}

fn write_slots(
    world: &mut World,
    entity: EntityId,
    path: &str,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    for (slot, _) in &fields {
        check_slot(*slot)?;
    }
    match fields.into_iter().last() {
        Some((_, value)) => write_field(world, entity, path, FIELD, value),
        None => Ok(false),
    }
}

fn remove(world: &mut World, entity: EntityId, _: &str) -> Result<bool, ReflectError> {
    world
        .remove_dynamic_component(entity, TYPE_PATH)
        .map_err(|error| error.into_reflect_error(TYPE_PATH.into()))
}

fn stage_clone(
    source: &World,
    entity: EntityId,
    path: &str,
    target: &mut World,
) -> Result<(), ReflectError> {
    let value = read_field(source, entity, path, FIELD)?;
    write_field(target, entity, path, FIELD, value)?;
    Ok(())
}
