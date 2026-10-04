use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::world_sync::{
    ComponentSelector, ComponentWorldQuery, EntityRow, QueryFilter, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

use crate::scene::{NodeKind, World};

use super::{
    build_inspection_fields, query_matches_reflected_components, selected_reflected_components,
};

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const MESH_RENDERER_TYPE_PATH: &str = "zircon_runtime::scene::components::MeshRenderer";
const PROJECTION_ENTITY_COUNT: usize = 512;
const PROJECTION_SAMPLE_PAIRS: usize = 21;
const PROJECTION_ITERATIONS: usize = 8;

#[test]
fn unbounded_component_projection_walks_stable_entities_without_node_records() {
    let source = include_str!("../../snapshot.rs");
    let unbounded_query = source
        .split("pub(crate) fn query_world_at_replacement_epoch")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn query_world_bounded").next())
        .expect("unbounded world-query implementation should remain discoverable");

    assert!(
        unbounded_query.contains("entity_ids_for_query()"),
        "unbounded component projection must use the bounded path's stable entity source"
    );
    assert!(
        !unbounded_query.contains(".node_records()"),
        "unbounded component projection must not allocate full scene-node records"
    );
}

#[test]
fn unbounded_component_projection_matches_bounded_filter_selection_and_generation() {
    let mut world = World::empty();
    let first_empty = world
        .spawn_node(NodeKind::Empty)
        .expect("first empty test node should spawn");
    let excluded_mesh = world
        .spawn_node(NodeKind::Cube)
        .expect("excluded mesh test node should spawn");
    let second_empty = world
        .spawn_node(NodeKind::Empty)
        .expect("second empty test node should spawn");
    world
        .rename_node(first_empty, "First Empty")
        .expect("first test node should be renamed");
    world
        .rename_node(excluded_mesh, "Excluded Mesh")
        .expect("mesh test node should be renamed");
    world
        .rename_node(second_empty, "Second Empty")
        .expect("second test node should be renamed");

    let query = WorldQuery::Components(ComponentWorldQuery {
        filter: QueryFilter {
            with: vec![NAME_TYPE_PATH.to_string()],
            without: vec![MESH_RENDERER_TYPE_PATH.to_string()],
        },
        select: vec![ComponentSelector::new(NAME_TYPE_PATH)],
        generation_hint: None,
    });
    let unbounded = world.query_world(&query);
    let bounded = world
        .query_world_bounded(&query, generous_query_limit())
        .expect("generous component-query budget should accept the fixture");

    assert_eq!(unbounded, bounded);
    assert_eq!(
        serde_json::to_vec(&unbounded).expect("unbounded result should serialize"),
        serde_json::to_vec(&bounded).expect("bounded result should serialize")
    );
    let WorldQueryResult::ComponentRows { generation, rows } = unbounded else {
        panic!("component query should materialize component rows");
    };
    assert_eq!(generation, world.world_generation());
    assert_eq!(
        rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![first_empty, second_empty]
    );
    assert!(rows.iter().all(|row| row
        .components
        .keys()
        .map(String::as_str)
        .eq([NAME_TYPE_PATH])));

    let not_modified = query.with_generation_hint(Some(generation));
    assert_eq!(
        world.query_world(&not_modified),
        world
            .query_world_bounded(&not_modified, generous_query_limit())
            .expect("not-modified envelope should fit the generous budget")
    );
    assert_eq!(
        world.query_world(&not_modified),
        WorldQueryResult::NotModified { generation }
    );
}

#[test]
fn direct_component_projection_preserves_entity_id_order_after_scrambled_batch_insert() {
    let mut source = World::empty();
    let first = source
        .spawn_node(NodeKind::Empty)
        .expect("first source node should spawn");
    let second = source
        .spawn_node(NodeKind::Empty)
        .expect("second source node should spawn");
    let mut low_id = source
        .node_record(first)
        .expect("first source node should project");
    let mut high_id = source
        .node_record(second)
        .expect("second source node should project");
    low_id.id = 7;
    high_id.id = 42;

    let mut world = World::empty();
    world
        .insert_owned_node_records(vec![high_id, low_id])
        .expect("scrambled node records should insert as one batch");

    let WorldQueryResult::ComponentRows { rows, .. } = world.query_world(&WorldQuery::default())
    else {
        panic!("default component query should materialize rows");
    };
    assert_eq!(
        rows.iter().map(|row| row.entity).collect::<Vec<_>>(),
        vec![7, 42],
        "component result boundary must retain canonical entity-id ordering"
    );
}

#[test]
fn unbounded_component_projection_retains_stable_entities_without_reflected_fields() {
    let mut world = World::empty();
    assert!(world
        .spawn_empty_at(77)
        .expect("explicit empty entity should spawn"));

    let query = WorldQuery::default();
    let unbounded = world.query_world(&query);
    let bounded = world
        .query_world_bounded(&query, generous_query_limit())
        .expect("componentless entity should fit the generous query budget");
    assert_eq!(unbounded, bounded);

    let WorldQueryResult::ComponentRows { rows, .. } = unbounded else {
        panic!("default component query should materialize rows");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity, 77);
    assert!(rows[0].components.is_empty());
}

#[test]
#[ignore = "release performance evidence; managed Windows Release requires candidate P95 <= 50% of legacy"]
fn direct_component_projection_release_percentiles() {
    let world = component_projection_benchmark_fixture();
    let query = WorldQuery::Components(ComponentWorldQuery {
        filter: QueryFilter {
            with: vec![NAME_TYPE_PATH.to_string()],
            without: Vec::new(),
        },
        select: vec![ComponentSelector::new(NAME_TYPE_PATH)],
        generation_hint: None,
    });
    let expected = world.query_world(&query);

    let mut legacy_samples = Vec::with_capacity(PROJECTION_SAMPLE_PAIRS);
    let mut direct_samples = Vec::with_capacity(PROJECTION_SAMPLE_PAIRS);
    let mut legacy_checksum = 0_usize;
    let mut direct_checksum = 0_usize;
    for pair in 0..PROJECTION_SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_component_projection(PROJECTION_ITERATIONS, || {
                legacy_component_projection(&world, &query)
            })
        };
        let measure_direct =
            || measure_component_projection(PROJECTION_ITERATIONS, || world.query_world(&query));
        if pair % 2 == 0 {
            let (elapsed, checksum) = measure_legacy();
            legacy_samples.push(elapsed);
            legacy_checksum = legacy_checksum.wrapping_add(checksum);
            let (elapsed, checksum) = measure_direct();
            direct_samples.push(elapsed);
            direct_checksum = direct_checksum.wrapping_add(checksum);
        } else {
            let (elapsed, checksum) = measure_direct();
            direct_samples.push(elapsed);
            direct_checksum = direct_checksum.wrapping_add(checksum);
            let (elapsed, checksum) = measure_legacy();
            legacy_samples.push(elapsed);
            legacy_checksum = legacy_checksum.wrapping_add(checksum);
        }
    }

    assert_eq!(legacy_checksum, direct_checksum);
    assert_eq!(legacy_component_projection(&world, &query), expected);

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let direct_p50_ns = nearest_rank(&direct_samples, 50);
    let direct_p95_ns = nearest_rank(&direct_samples, 95);
    println!(
        "RUNTIME_ECS_COMPONENT_PROJECTION_BENCH_V1 entities={PROJECTION_ENTITY_COUNT} sample_pairs={PROJECTION_SAMPLE_PAIRS} iterations={PROJECTION_ITERATIONS} pair_order=alternating_legacy_first_even percentile_method=nearest_rank legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} direct_p50_ns={direct_p50_ns} direct_p95_ns={direct_p95_ns} legacy_ns={} direct_ns={} p95_limit_percent=50",
        sample_csv(&legacy_samples),
        sample_csv(&direct_samples),
    );
    assert!(
        direct_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "direct component projection P95 must be at most 50% of legacy projection: legacy={legacy_p95_ns}ns direct={direct_p95_ns}ns"
    );
}

fn component_projection_benchmark_fixture() -> World {
    let mut world = World::empty();
    for index in 0..PROJECTION_ENTITY_COUNT {
        let kind = if index % 3 == 0 {
            NodeKind::Cube
        } else {
            NodeKind::Empty
        };
        world
            .spawn_node(kind)
            .expect("component projection benchmark fixture should spawn");
    }
    world
}

fn legacy_component_projection(world: &World, query: &WorldQuery) -> WorldQueryResult {
    let WorldQuery::Components(component_query) = query else {
        panic!("component projection benchmark requires a Components query");
    };
    let rows = world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let fields = build_inspection_fields(world, node.id);
            query_matches_reflected_components(&fields, &component_query.filter).then(|| {
                EntityRow {
                    entity: node.id,
                    components: selected_reflected_components(&fields, component_query),
                }
            })
        })
        .collect();
    query.component_result_for_generation(world.world_generation(), rows)
}

fn measure_component_projection(
    iterations: usize,
    mut projection: impl FnMut() -> WorldQueryResult,
) -> (u128, usize) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(component_result_checksum(&black_box(projection())));
    }
    (started.elapsed().as_nanos(), black_box(checksum))
}

fn component_result_checksum(result: &WorldQueryResult) -> usize {
    match result {
        WorldQueryResult::ComponentRows { rows, .. } => {
            rows.iter().fold(0_usize, |checksum, row| {
                checksum
                    .wrapping_add(row.entity as usize)
                    .wrapping_add(row.components.len())
            })
        }
        _ => 0,
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn generous_query_limit() -> ZrRuntimePayloadLimitV1 {
    ZrRuntimePayloadLimitV1::new(1024 * 1024, 1024, 1_000_000)
}
