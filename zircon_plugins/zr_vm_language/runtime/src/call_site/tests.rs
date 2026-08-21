use std::collections::HashMap;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::json;
use zircon_runtime::scene::{
    EntityId, NodeKind, ReflectComponent, RuntimeTypeRegistration, TypeRegistry, VmTypeBacking,
    World,
};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectScriptVisibility,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use super::{CallSiteError, ScriptCallTable};

const TOKEN_BENCH_CALL_SITES: usize = 4_096;
const TOKEN_BENCH_ROUNDS: usize = 32;
const TOKEN_BENCH_SAMPLE_PAIRS: usize = 21;

#[test]
fn call_site_resolution_happens_once() {
    let mut world = World::empty();
    world
        .register_vm_type(vm_health_registration(), VmTypeBacking::DynamicComponent)
        .expect("public VM component type should register");
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.Health",
            json!({ "current": 25.0, "maximum": 100.0 }),
        )
        .expect("dynamic component should attach");

    let table = ScriptCallTable::compile(world.type_registry())
        .expect("registered reflection schema should compile into dense call sites");
    let site = table
        .resolve("gameplay.Component.Health", "current")
        .expect("field call site should resolve during module loading");

    assert_eq!(site.type_slot, 0);
    assert_eq!(site.member_slot, 0);
    assert_eq!(site.layout.value_type_path.as_ref(), "Scalar");
    assert_eq!(table.resolution_count(), 1);
    assert_eq!(
        table
            .read(&site, &world, entity)
            .expect("compiled field read should succeed"),
        ReflectedValue::Scalar(25.0)
    );
    assert!(
        table
            .write(&site, &mut world, entity, ReflectedValue::Scalar(40.0))
            .expect("compiled field write should succeed")
    );
    assert_eq!(
        table
            .read(&site, &world, entity)
            .expect("compiled field read should observe the write"),
        ReflectedValue::Scalar(40.0)
    );
    assert_eq!(
        table.resolution_count(),
        1,
        "runtime calls must not repeat type/member name resolution"
    );

    let token = site.token();
    assert_eq!(
        table
            .read_token(token, &world, entity)
            .expect("opaque numeric call-site token should use the dense callbacks"),
        ReflectedValue::Scalar(40.0)
    );
    assert_eq!(table.resolution_count(), 1);
}

#[test]
fn private_types_are_not_compiled_into_script_call_sites() {
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration::metadata(
            reflected_component_registration(
                "test.PublicProbe",
                "PublicProbe",
                ReflectScriptVisibility::Public,
            ),
        ))
        .expect("public test registration should be valid");
    registry
        .register(RuntimeTypeRegistration::metadata(
            reflected_component_registration(
                "test.PrivateProbe",
                "PrivateProbe",
                ReflectScriptVisibility::Private,
            ),
        ))
        .expect("private test registration should be valid");

    let table =
        ScriptCallTable::compile(&registry).expect("public reflection table should compile");

    assert!(table.resolve("test.PublicProbe", "value").is_ok());
    assert!(matches!(
        table.resolve("test.PrivateProbe", "value"),
        Err(super::CallSiteError::UnknownMember { .. })
    ));
}

#[test]
fn opaque_tokens_are_never_reused_across_compiled_tables() {
    let mut world = World::empty();
    world
        .register_vm_type(vm_health_registration(), VmTypeBacking::DynamicComponent)
        .expect("public VM component type should register");
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.Health",
            json!({ "current": 25.0, "maximum": 100.0 }),
        )
        .expect("dynamic component should attach");
    let first =
        ScriptCallTable::compile(world.type_registry()).expect("first table should compile");
    let second =
        ScriptCallTable::compile(world.type_registry()).expect("second table should compile");
    let first_site = first
        .resolve("gameplay.Component.Health", "current")
        .expect("first table should resolve");
    let second_site = second
        .resolve("gameplay.Component.Health", "current")
        .expect("second table should resolve");

    assert_ne!(first_site.token(), second_site.token());
    assert!(matches!(
        second.read_token(first_site.token(), &world, entity),
        Err(CallSiteError::InvalidToken { token }) if token == first_site.token()
    ));
}

#[test]
fn tokens_embed_table_generation_and_one_based_dense_ordinal() {
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration::metadata(
            public_registration_with_fields("test.TokenProbe", 2),
        ))
        .expect("token probe registration should be valid");

    let first = ScriptCallTable::compile(&registry).expect("first table should compile");
    let second = ScriptCallTable::compile(&registry).expect("second table should compile");
    let first_a = first.resolve("test.TokenProbe", "field_0000").unwrap();
    let first_b = first.resolve("test.TokenProbe", "field_0001").unwrap();
    let second_a = second.resolve("test.TokenProbe", "field_0000").unwrap();

    assert_ne!(first_a.token() >> 32, 0);
    assert_eq!(first_a.token() >> 32, first_b.token() >> 32);
    assert_ne!(first_a.token() >> 32, second_a.token() >> 32);
    assert_eq!(first_a.token() as u32, 1);
    assert_eq!(first_b.token() as u32, 2);
    assert_eq!(second_a.token() as u32, 1);
}

#[test]
fn direct_token_resolution_rejects_wrong_generation_zero_and_out_of_range_ordinals() {
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration::metadata(
            public_registration_with_fields("test.TokenProbe", 2),
        ))
        .expect("token probe registration should be valid");
    let table = ScriptCallTable::compile(&registry).expect("table should compile");
    let token = table
        .resolve("test.TokenProbe", "field_0000")
        .unwrap()
        .token();

    for invalid in [
        token ^ (1_u64 << 32),
        token & 0xffff_ffff_0000_0000,
        (token & 0xffff_ffff_0000_0000) | u64::from(u32::MAX),
    ] {
        assert!(matches!(
            table.resolve_token_for_test(invalid),
            Err(CallSiteError::InvalidToken { token }) if token == invalid
        ));
    }
}

#[test]
#[ignore = "release performance gate"]
fn generation_qualified_direct_token_release_benchmark() {
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration::metadata(
            public_registration_with_fields("test.TokenBench", TOKEN_BENCH_CALL_SITES),
        ))
        .expect("token benchmark registration should be valid");
    let table = ScriptCallTable::compile(&registry).expect("token benchmark table should compile");
    let sites = (0..TOKEN_BENCH_CALL_SITES)
        .map(|index| {
            table
                .resolve("test.TokenBench", &format!("field_{index:04}"))
                .expect("benchmark field should resolve")
        })
        .collect::<Vec<_>>();
    let tokens = sites.iter().map(|site| site.token()).collect::<Vec<_>>();
    let legacy = sites
        .iter()
        .cloned()
        .map(|site| (site.token(), site))
        .collect::<HashMap<_, _>>();

    let mut legacy_samples = Vec::with_capacity(TOKEN_BENCH_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(TOKEN_BENCH_SAMPLE_PAIRS);
    for pair_index in 0..TOKEN_BENCH_SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy_token_resolution(&legacy, &tokens));
            optimized_samples.push(measure_direct_token_resolution(&table, &tokens));
        } else {
            optimized_samples.push(measure_direct_token_resolution(&table, &tokens));
            legacy_samples.push(measure_legacy_token_resolution(&legacy, &tokens));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let lookups_per_sample = TOKEN_BENCH_CALL_SITES * TOKEN_BENCH_ROUNDS;
    println!(
        "ZR_VM_CALLSITE_TOKEN_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank call_sites={} rounds={} lookups_per_sample={} legacy_compile_atomic_ops={} optimized_compile_atomic_ops=1 legacy_token_hash_entries={} optimized_token_hash_entries=0 legacy_hash_lookups={} optimized_hash_lookups=0 optimized_direct_index_lookups={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        TOKEN_BENCH_SAMPLE_PAIRS,
        TOKEN_BENCH_CALL_SITES,
        TOKEN_BENCH_ROUNDS,
        lookups_per_sample,
        TOKEN_BENCH_CALL_SITES,
        TOKEN_BENCH_CALL_SITES,
        lookups_per_sample,
        lookups_per_sample,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(2) <= legacy_p95,
        "direct token P95 must be at most 50% of legacy: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn measure_legacy_token_resolution(
    legacy: &HashMap<u64, super::CompiledCallSite>,
    tokens: &[u64],
) -> u128 {
    let started = Instant::now();
    for _ in 0..TOKEN_BENCH_ROUNDS {
        for token in tokens {
            black_box(legacy.get(token).cloned().expect("legacy token exists"));
        }
    }
    started.elapsed().as_nanos()
}

fn measure_direct_token_resolution(table: &ScriptCallTable, tokens: &[u64]) -> u128 {
    let started = Instant::now();
    for _ in 0..TOKEN_BENCH_ROUNDS {
        for token in tokens {
            black_box(
                table
                    .resolve_token_for_test(*token)
                    .expect("direct token exists"),
            );
        }
    }
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

static NAMED_FIELD_CALLS: AtomicUsize = AtomicUsize::new(0);
static DENSE_FIELD_CALLS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn runtime_calls_use_dense_slots_without_field_name_dispatch() {
    NAMED_FIELD_CALLS.store(0, Ordering::Relaxed);
    DENSE_FIELD_CALLS.store(0, Ordering::Relaxed);
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration {
            registration: ReflectTypeRegistration::new(
                ReflectTypePath::new("test.DenseProbe", "DenseProbe")
                    .expect("test reflection path should be valid"),
                "DenseProbe",
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                    "value",
                    "Unsigned",
                    ReflectEditorHint::Unsigned,
                )]),
                ReflectSerializationStrategy::Value,
            )
            .as_component()
            .with_script_visibility(ReflectScriptVisibility::Public),
            component: Some(
                ReflectComponent::new(
                    "test.DenseProbe",
                    dense_probe_contains,
                    dense_probe_named_read,
                    dense_probe_read_fields,
                    dense_probe_named_write,
                    dense_probe_remove,
                )
                .with_dense_field_slots(dense_probe_slot_read, dense_probe_slot_write),
            ),
            resource: None,
        })
        .expect("dense probe registration should succeed");

    let table = ScriptCallTable::compile(&registry)
        .expect("dense probe should compile into one numeric call site");
    let site = table
        .resolve("test.DenseProbe", "value")
        .expect("dense probe field should resolve once during loading");
    let mut world = World::empty();

    assert_eq!(
        table
            .read(&site, &world, 7)
            .expect("dense probe read should use the slot callback"),
        ReflectedValue::Unsigned(0)
    );
    assert!(
        table
            .write(&site, &mut world, 7, ReflectedValue::Unsigned(1))
            .expect("dense probe write should use the slot callback")
    );
    assert_eq!(NAMED_FIELD_CALLS.load(Ordering::Relaxed), 0);
    assert_eq!(DENSE_FIELD_CALLS.load(Ordering::Relaxed), 2);
}

fn dense_probe_contains(_world: &World, _entity: EntityId, _type_path: &str) -> bool {
    true
}

fn dense_probe_named_read(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
    _field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    NAMED_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(ReflectedValue::Unsigned(u64::MAX))
}

fn dense_probe_read_fields(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(Vec::new())
}

fn dense_probe_named_write(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    _field_name: &str,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    NAMED_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(false)
}

fn dense_probe_slot_read(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    DENSE_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(ReflectedValue::Unsigned(u64::from(field_slot)))
}

fn dense_probe_slot_write(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    field_slot: u32,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    DENSE_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(field_slot == 0)
}

fn dense_probe_remove(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    Ok(false)
}

fn vm_health_registration() -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new("gameplay.Component.Health", "Health")
            .expect("health reflection path should be valid")
            .with_plugin_id("gameplay"),
        "Health",
        ReflectTypeInfo::struct_with_fields(vec![
            ReflectFieldInfo::new("current", "Scalar", ReflectEditorHint::Scalar),
            ReflectFieldInfo::new("maximum", "Scalar", ReflectEditorHint::Scalar)
                .with_editable(false),
        ]),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_plugin_owned(true)
    .with_plugin_id("gameplay")
    .with_script_visibility(ReflectScriptVisibility::Public)
}

fn reflected_component_registration(
    type_path: &str,
    short_type_path: &str,
    visibility: ReflectScriptVisibility,
) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path)
            .expect("test reflection path should be valid"),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            "value",
            "Unsigned",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_script_visibility(visibility)
}

fn public_registration_with_fields(type_path: &str, field_count: usize) -> ReflectTypeRegistration {
    let short_type_path = type_path.rsplit('.').next().unwrap_or(type_path);
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path)
            .expect("benchmark reflection path should be valid"),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(
            (0..field_count)
                .map(|index| {
                    ReflectFieldInfo::new(
                        format!("field_{index:04}"),
                        "Unsigned",
                        ReflectEditorHint::Unsigned,
                    )
                })
                .collect(),
        ),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_script_visibility(ReflectScriptVisibility::Public)
}
