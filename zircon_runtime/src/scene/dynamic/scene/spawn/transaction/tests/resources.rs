use std::cell::Cell;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use crate::scene::dynamic_scene::{DynamicSceneError, EntityRemap};
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
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
                    SLOT_RESOURCE_TYPE_PATH,
                    "value",
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
                read_field_by_slot: slot_resource_read_field_by_slot,
                write_field_by_slot: slot_resource_write_field_by_slot,
                write_fields_by_slot: slot_resource_write_fields_by_slot,
            },
        )
        .expect("slot resource registration should be accepted");
}

#[test]
fn compiled_reflected_writes_reject_duplicate_stable_field_ids() {
    let mut world = World::empty();
    register_slot_resource(&mut world);
    let field_id = ReflectFieldId::from_stable_keys(SLOT_RESOURCE_TYPE_PATH, "value");
    let fields = vec![
        ReflectFieldValue::new(field_id, "value", ReflectedValue::Unsigned(1)),
        ReflectFieldValue::new(field_id, "stale_value", ReflectedValue::Unsigned(2)),
    ];
    let schema_fields = &world
        .type_registry()
        .runtime_registration(SLOT_RESOURCE_TYPE_PATH)
        .expect("slot resource registration should resolve")
        .registration
        .type_info
        .fields;

    let error = super::super::super::resource::compile_reflected_writes(
        &world,
        SLOT_RESOURCE_TYPE_PATH,
        schema_fields,
        &fields,
        &EntityRemap::new(),
    )
    .expect_err("duplicate stable field identities must fail scene admission");

    assert!(matches!(
        error,
        DynamicSceneError::Reflect(ReflectError::InvalidValue { reason, .. })
            if reason == "duplicate stable field identity in dynamic scene payload"
    ));
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

fn slot_resource_read_field_by_slot(
    world: &World,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match field_slot {
        0 => Ok(ReflectedValue::Unsigned(slot_resource(world)?.value as u64)),
        _ => Err(slot_resource_unknown_field(&format!("#{field_slot}"))),
    }
}

fn slot_resource_write_fields_by_slot(
    world: &mut World,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    admit_slot_resource_write()?;
    let mut next = slot_resource(world)?.value;
    for (field_slot, value) in fields {
        next = slot_resource_value(field_slot, value)?;
    }
    update_slot_resource(world, next)
}

fn slot_resource_write_field_by_slot(
    world: &mut World,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    admit_slot_resource_write()?;
    update_slot_resource(world, slot_resource_value(field_slot, value)?)
}

fn admit_slot_resource_write() -> Result<(), ReflectError> {
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
    Ok(())
}

fn slot_resource_value(field_slot: u32, value: ReflectedValue) -> Result<u32, ReflectError> {
    if field_slot != 0 {
        return Err(slot_resource_unknown_field(&format!("#{field_slot}")));
    }
    u32::try_from(match value {
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
    })
}

fn update_slot_resource(world: &mut World, next: u32) -> Result<bool, ReflectError> {
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
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
                    REJECTING_RESOURCE_TYPE_PATH,
                    "value",
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
                read_field_by_slot: rejecting_resource_read_field_by_slot,
                write_field_by_slot: rejecting_resource_write_field_by_slot,
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

fn rejecting_resource_read_field_by_slot(
    world: &World,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match field_slot {
        0 => Ok(ReflectedValue::Unsigned(
            rejecting_resource(world)?.0 as u64,
        )),
        _ => Err(ReflectError::UnknownField {
            type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
            field_name: format!("#{field_slot}"),
        }),
    }
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

fn rejecting_resource_write_field_by_slot(
    _world: &mut World,
    field_slot: u32,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    Err(ReflectError::UnsupportedConversion {
        source: "resource write intentionally rejected by test adapter".to_string(),
        target: format!("{REJECTING_RESOURCE_TYPE_PATH}.#{field_slot}"),
    })
}

fn rejecting_resource(world: &World) -> Result<&RejectingResource, ReflectError> {
    world
        .get_resource::<RejectingResource>()
        .ok_or_else(|| ReflectError::MissingResource {
            type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
        })
}
