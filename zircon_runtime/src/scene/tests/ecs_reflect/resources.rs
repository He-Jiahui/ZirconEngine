use std::{cell::Cell, hint::black_box, time::Instant};

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
    ReflectObjectAddress, ReflectReadRequest, ReflectSchemaRequest, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
};

use crate::scene::ecs::Resource;
use crate::scene::{NodeKind, ReflectResource, RuntimeTypeRegistration, World};

const FRAME_COUNTER_TYPE_PATH: &str = "zircon_runtime::scene::tests::ecs_reflect::FrameCounter";
const RESOURCE_SINGLE_WRITE_BENCH_PAIRS: usize = 21;
const RESOURCE_SINGLE_WRITE_BENCH_WRITES: usize = 100_000;

thread_local! {
    static FRAME_COUNTER_NAMED_READS: Cell<usize> = const { Cell::new(0) };
    static FRAME_COUNTER_SLOT_READS: Cell<usize> = const { Cell::new(0) };
    static FRAME_COUNTER_SINGLE_WRITES: Cell<usize> = const { Cell::new(0) };
    static FRAME_COUNTER_BATCH_WRITES: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter {
    value: u32,
}

impl Resource for FrameCounter {}

#[test]
fn manual_resource_registration_adds_reflected_resource_schema() {
    let mut world = World::empty();

    register_frame_counter_resource(&mut world);

    let registration = world
        .reflect_schema(FRAME_COUNTER_TYPE_PATH)
        .expect("resource schema should be registered");
    assert_eq!(registration.type_path.type_path(), FRAME_COUNTER_TYPE_PATH);
    assert_eq!(registration.type_path.short_type_path(), "FrameCounter");
    assert_eq!(registration.display_name, "Frame Counter");
    assert!(!registration.is_component());
    assert!(registration.is_resource());
    assert!(matches!(
        registration.serialization,
        ReflectSerializationStrategy::ResourceHandle
    ));
    assert_eq!(registration.type_info.fields.len(), 1);
    assert_eq!(registration.type_info.fields[0].name, "value");
    assert_eq!(registration.type_info.fields[0].value_type_path, "Unsigned");
    assert!(registration.type_info.fields[0].editable);

    let listed = world
        .list_reflect_types(ReflectSchemaRequest::for_type("FrameCounter"))
        .expect("short resource type path should resolve")
        .registrations;

    assert_eq!(listed, vec![registration]);
}

#[test]
fn resource_reflection_reads_and_writes_field_through_facade() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 7 });
    let address = frame_counter_address();

    reset_frame_counter_read_routes();
    let read = world
        .reflect_read(ReflectReadRequest::new(
            address.clone(),
            frame_counter_field_id(),
        ))
        .expect("resource field should read through reflection");
    assert_eq!(
        read.field,
        ReflectFieldValue::new(
            frame_counter_field_id(),
            "value",
            ReflectedValue::Unsigned(7),
        )
    );
    assert_eq!(frame_counter_read_routes(), (0, 1));
    let fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(address.clone()),
        )
        .expect("resource fields should enumerate through reflection")
        .fields;
    assert_eq!(
        fields,
        vec![ReflectFieldValue::new(
            frame_counter_field_id(),
            "value",
            ReflectedValue::Unsigned(7),
        )]
    );

    reset_frame_counter_write_routes();
    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            frame_counter_field_id(),
            ReflectedValue::Unsigned(11),
        ))
        .expect("resource field should write through reflection");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new(
            frame_counter_field_id(),
            "value",
            ReflectedValue::Unsigned(11),
        )
    );
    assert_eq!(world.get_resource::<FrameCounter>().unwrap().value, 11);
    assert_eq!(frame_counter_write_routes(), (1, 0));

    let unchanged = world
        .reflect_write(ReflectWriteRequest::new(
            address,
            frame_counter_field_id(),
            ReflectedValue::Unsigned(11),
        ))
        .expect("same resource value should be accepted as unchanged");
    assert!(!unchanged.changed);
}

#[test]
fn resource_reflection_write_updates_change_tick() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 1 });
    let before = world
        .resource_change_ticks::<FrameCounter>()
        .expect("inserted resource should have ticks")
        .changed();

    world
        .reflect_write(ReflectWriteRequest::new(
            frame_counter_address(),
            frame_counter_field_id(),
            ReflectedValue::Unsigned(2),
        ))
        .expect("resource write should route through mutable resource access");

    let after = world
        .resource_change_ticks::<FrameCounter>()
        .expect("written resource should still have ticks")
        .changed();
    assert!(after > before);
}

#[test]
fn missing_reflected_resource_returns_structured_error() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                frame_counter_address(),
                frame_counter_field_id(),
            ))
            .expect_err("missing reflected resources should be structured"),
        ReflectError::MissingResource {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                frame_counter_address(),
                frame_counter_field_id(),
                ReflectedValue::Unsigned(1),
            ))
            .expect_err("missing reflected resource writes should be structured"),
        ReflectError::MissingResource {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
}

#[test]
fn resource_registration_without_adapter_returns_structured_error() {
    let mut world = World::empty();
    world.type_registry_mut_for_tests().clear();
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(
            frame_counter_registration(),
        ))
        .expect("metadata-only resource registration should be accepted");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                frame_counter_address(),
                frame_counter_field_id(),
            ))
            .expect_err("metadata-only resource should report missing resource adapter"),
        ReflectError::NoResourceAdapter {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
}

#[test]
fn component_and_resource_reflection_share_address_and_facade_shape() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 3 });
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let component_address =
        ReflectObjectAddress::component(entity, "Name").expect("component address should be valid");
    let resource_address = frame_counter_address();

    let component_read = world
        .reflect_read(ReflectReadRequest::new(
            component_address.clone(),
            ReflectFieldId::from_stable_keys("zircon_runtime::scene::components::Name", "value"),
        ))
        .expect("component read should use shared facade");
    let resource_read = world
        .reflect_read(ReflectReadRequest::new(
            resource_address.clone(),
            frame_counter_field_id(),
        ))
        .expect("resource read should use shared facade");

    assert_eq!(component_read.address, component_address.clone());
    assert_eq!(resource_read.address, resource_address.clone());
    assert_eq!(component_read.field.field_name, "value");
    assert_eq!(resource_read.field.field_name, "value");

    let schema_type_paths = world
        .list_reflect_types(ReflectSchemaRequest::editor_visible())
        .expect("component and resource schemas should share schema facade")
        .registrations
        .into_iter()
        .map(|registration| registration.type_path.type_path().to_string())
        .collect::<Vec<_>>();
    assert!(schema_type_paths.contains(&"zircon_runtime::scene::components::Name".to_string()));
    assert!(schema_type_paths.contains(&FRAME_COUNTER_TYPE_PATH.to_string()));

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::component(entity, FRAME_COUNTER_TYPE_PATH)
                    .expect("component-shaped resource address should be valid DTO"),
                frame_counter_field_id(),
            ))
            .expect_err("resource registration cannot be addressed as a component"),
        ReflectError::AddressKindMismatch {
            expected: format!("component `{FRAME_COUNTER_TYPE_PATH}`"),
            actual: format!("non-component `{FRAME_COUNTER_TYPE_PATH}`"),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::resource("Name")
                    .expect("resource-shaped component address should be valid DTO"),
                ReflectFieldId::from_stable_keys(
                    "zircon_runtime::scene::components::Name",
                    "value",
                ),
            ))
            .expect_err("component registration cannot be addressed as a resource"),
        ReflectError::AddressKindMismatch {
            expected: "resource `zircon_runtime::scene::components::Name`".to_string(),
            actual: "non-resource `zircon_runtime::scene::components::Name`".to_string(),
        }
    );
}

fn register_frame_counter_resource(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(frame_counter_registration(), frame_counter_adapter())
        .expect("frame counter resource registration should be accepted");
}

fn frame_counter_registration() -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(FRAME_COUNTER_TYPE_PATH, "FrameCounter")
            .expect("frame counter type path should be valid"),
        "Frame Counter",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            FRAME_COUNTER_TYPE_PATH,
            "value",
            "value",
            "Unsigned",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::ResourceHandle,
    )
    .as_resource()
    .with_remote_visible(true)
}

fn frame_counter_adapter() -> ReflectResource {
    ReflectResource {
        estimate_stage_clone_bytes: None,
        stage_clone: None,
        transfer_preflight: frame_counter_transfer_preflight,
        ensure: None,
        contains: frame_counter_contains,
        read_field: frame_counter_read_field,
        read_field_by_slot: frame_counter_read_field_by_slot,
        write_field_by_slot: frame_counter_write_field_by_slot,
        write_fields_by_slot: frame_counter_write_fields_by_slot,
    }
}

fn frame_counter_address() -> ReflectObjectAddress {
    ReflectObjectAddress::resource(FRAME_COUNTER_TYPE_PATH)
        .expect("resource address should be valid")
}

fn frame_counter_contains(world: &World) -> bool {
    world.get_resource::<FrameCounter>().is_some()
}

fn frame_counter_transfer_preflight(
    source: &mut World,
    artifact: &mut World,
) -> Result<(), ReflectError> {
    source.transfer_preflight_resource::<FrameCounter>(artifact)
}

fn frame_counter_read_field(
    world: &World,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    FRAME_COUNTER_NAMED_READS.with(|count| count.set(count.get().saturating_add(1)));
    let resource = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    match field_name {
        "value" => Ok(ReflectedValue::Unsigned(resource.value as u64)),
        _ => Err(unknown_frame_counter_field(field_name)),
    }
}

fn frame_counter_field_id() -> ReflectFieldId {
    ReflectFieldId::from_stable_keys(FRAME_COUNTER_TYPE_PATH, "value")
}

fn frame_counter_read_field_by_slot(
    world: &World,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    FRAME_COUNTER_SLOT_READS.with(|count| count.set(count.get().saturating_add(1)));
    if field_slot != 0 {
        return Err(unknown_frame_counter_field(&format!("#{field_slot}")));
    }
    let resource = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    Ok(ReflectedValue::Unsigned(resource.value as u64))
}

fn frame_counter_write_fields_by_slot(
    world: &mut World,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    FRAME_COUNTER_BATCH_WRITES.with(|count| count.set(count.get().saturating_add(1)));
    let current = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?
        .value;
    let mut next = current;
    for (field_slot, value) in fields {
        if field_slot != 0 {
            return Err(unknown_frame_counter_field(&format!("#{field_slot}")));
        }
        next = expect_frame_counter_value("value", value)?;
    }
    if current == next {
        return Ok(false);
    }

    world
        .get_resource_mut::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?
        .value = next;
    Ok(true)
}

fn frame_counter_write_field_by_slot(
    world: &mut World,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    FRAME_COUNTER_SINGLE_WRITES.with(|count| count.set(count.get().saturating_add(1)));
    if field_slot != 0 {
        return Err(unknown_frame_counter_field(&format!("#{field_slot}")));
    }
    let next = expect_frame_counter_value("value", value)?;
    let resource = world
        .get_resource_mut::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    if resource.value == next {
        return Ok(false);
    }
    resource.value = next;
    Ok(true)
}

fn reset_frame_counter_write_routes() {
    FRAME_COUNTER_SINGLE_WRITES.with(|count| count.set(0));
    FRAME_COUNTER_BATCH_WRITES.with(|count| count.set(0));
}

fn reset_frame_counter_read_routes() {
    FRAME_COUNTER_NAMED_READS.with(|count| count.set(0));
    FRAME_COUNTER_SLOT_READS.with(|count| count.set(0));
}

fn frame_counter_read_routes() -> (usize, usize) {
    (
        FRAME_COUNTER_NAMED_READS.with(Cell::get),
        FRAME_COUNTER_SLOT_READS.with(Cell::get),
    )
}

fn frame_counter_write_routes() -> (usize, usize) {
    (
        FRAME_COUNTER_SINGLE_WRITES.with(Cell::get),
        FRAME_COUNTER_BATCH_WRITES.with(Cell::get),
    )
}

#[test]
#[ignore = "release performance gate"]
fn resource_reflection_single_write_release_benchmark() {
    let adapter = frame_counter_adapter();
    let mut legacy_samples = Vec::with_capacity(RESOURCE_SINGLE_WRITE_BENCH_PAIRS);
    let mut optimized_samples = Vec::with_capacity(RESOURCE_SINGLE_WRITE_BENCH_PAIRS);
    for pair_index in 0..RESOURCE_SINGLE_WRITE_BENCH_PAIRS {
        let legacy_first = pair_index % 2 == 0;
        if legacy_first {
            legacy_samples.push(measure_resource_writes(&adapter, true));
            optimized_samples.push(measure_resource_writes(&adapter, false));
        } else {
            optimized_samples.push(measure_resource_writes(&adapter, false));
            legacy_samples.push(measure_resource_writes(&adapter, true));
        }
    }
    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);

    println!(
        "RESOURCE_REFLECTION_SINGLE_WRITE_BENCH_V1 sample_pairs={} writes_per_sample={} legacy_allocations_per_sample={} optimized_allocations_per_sample=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={} optimized_samples_ns={}",
        RESOURCE_SINGLE_WRITE_BENCH_PAIRS,
        RESOURCE_SINGLE_WRITE_BENCH_WRITES,
        RESOURCE_SINGLE_WRITE_BENCH_WRITES,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(4) <= legacy_p95.saturating_mul(3),
        "single-slot writes must reduce nearest-rank P95 by at least 25%: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn measure_resource_writes(adapter: &ReflectResource, legacy: bool) -> u128 {
    let mut world = World::empty();
    world.insert_resource(FrameCounter { value: 0 });
    let started = Instant::now();
    for index in 0..RESOURCE_SINGLE_WRITE_BENCH_WRITES {
        let value = ReflectedValue::Unsigned(((index + 1) & 1) as u64);
        let changed = if legacy {
            adapter
                .write_fields_by_slot(&mut world, vec![(0, value)])
                .expect("legacy single-element batch write")
        } else {
            adapter
                .write_field_by_slot(&mut world, 0, value)
                .expect("optimized single-slot write")
        };
        black_box(changed);
    }
    black_box(world.get_resource::<FrameCounter>().unwrap().value);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn expect_frame_counter_value(
    field_name: &str,
    value: ReflectedValue,
) -> Result<u32, ReflectError> {
    match value {
        ReflectedValue::Unsigned(value) if u32::try_from(value).is_ok() => Ok(value as u32),
        ReflectedValue::Unsigned(_) => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "u32 Unsigned".to_string(),
            actual: "Unsigned".to_string(),
        }),
        value => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "Unsigned".to_string(),
            actual: value.type_name().to_string(),
        }),
    }
}

fn missing_frame_counter_resource() -> ReflectError {
    ReflectError::MissingResource {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
    }
}

fn unknown_frame_counter_field(field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        field_name: field_name.to_string(),
    }
}
