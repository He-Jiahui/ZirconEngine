use std::cell::Cell;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use crate::scene::{ReflectResource, Resource, World};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SlotResource {
    pub(super) value: u32,
}

impl Resource for SlotResource {}

const SLOT_RESOURCE_TYPE_PATH: &str =
    "zircon_runtime::scene::dynamic_scene::scene::spawn::tests::SlotResource";

thread_local! {
    static SLOT_RESOURCE_WRITE_BUDGET: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) struct SlotResourceWriteBudgetReset;

impl SlotResourceWriteBudgetReset {
    pub(super) fn allow_exactly_one_write() -> Self {
        SLOT_RESOURCE_WRITE_BUDGET.with(|budget| budget.set(Some(1)));
        Self
    }
}

impl Drop for SlotResourceWriteBudgetReset {
    fn drop(&mut self) {
        SLOT_RESOURCE_WRITE_BUDGET.with(|budget| budget.set(None));
    }
}

pub(super) fn register_slot_resource(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(
            ReflectTypeRegistration::new(
                ReflectTypePath::new(SLOT_RESOURCE_TYPE_PATH, "SlotResource")
                    .expect("slot resource type path should be valid"),
                "Slot Resource",
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                    "value",
                    "Unsigned",
                    ReflectEditorHint::Unsigned,
                )]),
                ReflectSerializationStrategy::ResourceHandle,
            )
            .as_resource()
            .with_remote_visible(true),
            ReflectResource {
                estimate_stage_clone_bytes: Some(slot_resource_stage_clone_bytes),
                stage_clone: Some(slot_resource_stage_clone),
                transfer_preflight: slot_resource_transfer_preflight,
                ensure: None,
                contains: slot_resource_contains,
                read_field: slot_resource_read_field,
                read_fields: slot_resource_read_fields,
                write_fields_by_slot: slot_resource_write_fields_by_slot,
            },
        )
        .expect("slot resource registration should be accepted");
}

fn slot_resource_stage_clone_bytes(source: &World) -> Result<usize, ReflectError> {
    slot_resource(source)?;
    Ok(std::mem::size_of::<SlotResource>())
}

fn slot_resource_stage_clone(source: &World, target: &mut World) -> Result<(), ReflectError> {
    target.insert_resource(SlotResource {
        value: slot_resource(source)?.value,
    });
    Ok(())
}

fn slot_resource_transfer_preflight(
    source: &mut World,
    artifact: &mut World,
) -> Result<(), ReflectError> {
    source.transfer_preflight_resource::<SlotResource>(artifact)
}

fn slot_resource_contains(world: &World) -> bool {
    world.get_resource::<SlotResource>().is_some()
}

fn slot_resource_read_field(
    world: &World,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    match field_name {
        "value" => Ok(ReflectedValue::Unsigned(slot_resource(world)?.value as u64)),
        _ => Err(slot_resource_unknown_field(field_name)),
    }
}

fn slot_resource_read_fields(world: &World) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        slot_resource_read_field(world, "value")?,
    )])
}

fn slot_resource_write_fields_by_slot(
    world: &mut World,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    let target_only_failure = SLOT_RESOURCE_WRITE_BUDGET.with(|budget| {
        let Some(remaining_successes) = budget.get() else {
            return false;
        };
        if remaining_successes == 0 {
            return true;
        }
        budget.set(Some(remaining_successes - 1));
        false
    });
    if target_only_failure {
        return Err(ReflectError::UnsupportedConversion {
            source: "resource write rejected after the preflight execution".to_string(),
            target: SLOT_RESOURCE_TYPE_PATH.to_string(),
        });
    }
    let mut next = slot_resource(world)?.value;
    for (field_slot, value) in fields {
        if field_slot != 0 {
            return Err(slot_resource_unknown_field(&format!("#{field_slot}")));
        }
        next = u32::try_from(match value {
            ReflectedValue::Unsigned(value) => value,
            value => {
                return Err(ReflectError::TypeMismatch {
                    type_path: SLOT_RESOURCE_TYPE_PATH.to_string(),
                    field_name: "value".to_string(),
                    expected: "Unsigned".to_string(),
                    actual: value.type_name().to_string(),
                });
            }
        })
        .map_err(|_| ReflectError::TypeMismatch {
            type_path: SLOT_RESOURCE_TYPE_PATH.to_string(),
            field_name: "value".to_string(),
            expected: "u32 Unsigned".to_string(),
            actual: "Unsigned".to_string(),
        })?;
    }
    let resource =
        world
            .get_resource_mut::<SlotResource>()
            .ok_or_else(|| ReflectError::MissingResource {
                type_path: SLOT_RESOURCE_TYPE_PATH.to_string(),
            })?;
    if resource.value == next {
        return Ok(false);
    }
    resource.value = next;
    Ok(true)
}

fn slot_resource(world: &World) -> Result<&SlotResource, ReflectError> {
    world
        .get_resource::<SlotResource>()
        .ok_or_else(|| ReflectError::MissingResource {
            type_path: SLOT_RESOURCE_TYPE_PATH.to_string(),
        })
}

fn slot_resource_unknown_field(field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: SLOT_RESOURCE_TYPE_PATH.to_string(),
        field_name: field_name.to_string(),
    }
}

const REJECTING_RESOURCE_TYPE_PATH: &str =
    "zircon_runtime::scene::dynamic_scene::scene::spawn::tests::RejectingResource";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RejectingResource(pub(super) u32);

impl Resource for RejectingResource {}

pub(super) fn register_rejecting_resource(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(
            ReflectTypeRegistration::new(
                ReflectTypePath::new(REJECTING_RESOURCE_TYPE_PATH, "RejectingResource")
                    .expect("test resource type path should be valid"),
                "Rejecting Resource",
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                    "value",
                    "Unsigned",
                    ReflectEditorHint::Unsigned,
                )]),
                ReflectSerializationStrategy::ResourceHandle,
            )
            .as_resource()
            .with_remote_visible(true),
            ReflectResource {
                estimate_stage_clone_bytes: Some(rejecting_resource_stage_clone_bytes),
                stage_clone: Some(rejecting_resource_stage_clone),
                transfer_preflight: rejecting_resource_transfer_preflight,
                ensure: None,
                contains: rejecting_resource_contains,
                read_field: rejecting_resource_read_field,
                read_fields: rejecting_resource_read_fields,
                write_fields_by_slot: rejecting_resource_write_fields_by_slot,
            },
        )
        .expect("test resource registration should be accepted");
}

fn rejecting_resource_stage_clone_bytes(source: &World) -> Result<usize, ReflectError> {
    rejecting_resource(source)?;
    Ok(std::mem::size_of::<RejectingResource>())
}

fn rejecting_resource_stage_clone(source: &World, target: &mut World) -> Result<(), ReflectError> {
    target.insert_resource(RejectingResource(rejecting_resource(source)?.0));
    Ok(())
}

fn rejecting_resource_transfer_preflight(
    source: &mut World,
    artifact: &mut World,
) -> Result<(), ReflectError> {
    source.transfer_preflight_resource::<RejectingResource>(artifact)
}

fn rejecting_resource_contains(world: &World) -> bool {
    world.get_resource::<RejectingResource>().is_some()
}

fn rejecting_resource_read_field(
    world: &World,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    match field_name {
        "value" => Ok(ReflectedValue::Unsigned(
            rejecting_resource(world)?.0 as u64,
        )),
        _ => Err(ReflectError::UnknownField {
            type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
        }),
    }
}

fn rejecting_resource_read_fields(world: &World) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        rejecting_resource_read_field(world, "value")?,
    )])
}

fn rejecting_resource_write_fields_by_slot(
    _world: &mut World,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    let field_name = fields
        .first()
        .map(|(field_slot, _)| format!("#{field_slot}"))
        .unwrap_or_else(|| "<empty>".to_string());
    Err(ReflectError::UnsupportedConversion {
        source: "resource write intentionally rejected by test adapter".to_string(),
        target: format!("{REJECTING_RESOURCE_TYPE_PATH}.{field_name}"),
    })
}

fn rejecting_resource(world: &World) -> Result<&RejectingResource, ReflectError> {
    world
        .get_resource::<RejectingResource>()
        .ok_or_else(|| ReflectError::MissingResource {
            type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
        })
}
