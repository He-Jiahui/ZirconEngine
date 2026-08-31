use std::time::{Duration, Instant};

use crate::scene::ecs::{
    ArchetypeIndex, ArchetypeSignature, Changed, ComponentId, ComponentTicks, QueryState,
    SystemState,
};
use crate::scene::{EntityId, World};

use super::Health;

const COLUMNAR_ACCEPTANCE_SCALES: [usize; 3] = [1, 1_000, 100_000];
const P95_SAMPLE_COUNT: usize = 20;

#[derive(Debug)]
struct ColumnarQueryAcceptanceSample {
    entities: usize,
    components: usize,
    archetypes: usize,
    changed_zero: usize,
    changed_one: usize,
    changed_all: usize,
    plan_compilations: u64,
    component_membership_checks: u64,
    table_bindings: u64,
    sparse_bindings: u64,
    cache_misses: u64,
    retained_bytes: usize,
    p95: Duration,
}

#[derive(Debug)]
struct ArchetypeCardinalityAcceptanceSample {
    entities: usize,
    components: usize,
    archetypes: usize,
    component_index_probes: u64,
    signature_membership_checks: u64,
    row_appends: u64,
    retained_bytes: usize,
    p95: Duration,
}

fn p95(mut samples: Vec<Duration>) -> Duration {
    assert!(!samples.is_empty());
    samples.sort_unstable();
    let index = samples.len().saturating_mul(95).div_ceil(100) - 1;
    samples[index]
}

fn spawn_compact_health_entities(world: &mut World, count: usize) -> Vec<EntityId> {
    let mut entities = Vec::with_capacity(count);
    for index in 0..count {
        let entity = index as EntityId + 1;
        assert!(world.spawn_empty_at(entity).unwrap());
        world.insert(entity, Health(index as u32)).unwrap();
        entities.push(entity);
    }
    entities
}

#[test]
fn columnar_query_records_one_to_one_hundred_thousand_scale_counters() {
    for entity_count in COLUMNAR_ACCEPTANCE_SCALES {
        let mut world = World::empty();
        let entities = spawn_compact_health_entities(&mut world, entity_count);

        type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
        let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
        let mut timings = Vec::with_capacity(4);

        let start = Instant::now();
        let initial = system.run(&mut world, |mut query| query.iter().count());
        timings.push(start.elapsed());
        assert_eq!(initial, entity_count);

        let start = Instant::now();
        let changed_zero = system.run(&mut world, |mut query| query.iter().count());
        timings.push(start.elapsed());
        assert_eq!(changed_zero, 0);

        world.get_mut::<Health>(entities[0]).unwrap().0 += 1;
        let start = Instant::now();
        let changed_one = system.run(&mut world, |mut query| query.iter().count());
        timings.push(start.elapsed());
        assert_eq!(changed_one, 1);

        for entity in entities.iter().copied() {
            world.get_mut::<Health>(entity).unwrap().0 += 1;
        }
        let start = Instant::now();
        let changed_all = system.run(&mut world, |mut query| query.iter().count());
        timings.push(start.elapsed());
        assert_eq!(changed_all, entity_count);

        let stats = system.state().cache_stats();
        let sample = ColumnarQueryAcceptanceSample {
            entities: entity_count,
            components: 1,
            archetypes: system.state().cached_archetype_count(),
            changed_zero,
            changed_one,
            changed_all,
            plan_compilations: stats.archetype_plan_compilations,
            component_membership_checks: stats.archetype_component_membership_checks,
            table_bindings: stats.table_column_slot_bindings,
            sparse_bindings: stats.sparse_component_bindings,
            cache_misses: stats.cache_misses,
            retained_bytes: system.state().estimated_cache_bytes(),
            p95: p95(timings),
        };

        assert_eq!(sample.entities, entity_count);
        assert_eq!(sample.components, 1);
        assert_eq!(sample.archetypes, 1);
        assert_eq!((sample.changed_zero, sample.changed_one), (0, 1));
        assert_eq!(sample.changed_all, entity_count);
        assert_eq!(sample.plan_compilations, 1);
        assert_eq!(sample.component_membership_checks, 1);
        assert_eq!(sample.table_bindings, 1);
        assert_eq!(sample.sparse_bindings, 0);
        assert_eq!(sample.cache_misses, 1);
        assert!(sample.retained_bytes > 0);
        assert_ne!(sample.p95, Duration::MAX);
        eprintln!("ecs columnar query acceptance: {sample:?}");
    }
}

#[test]
fn archetype_index_records_component_and_archetype_cardinality_counters() {
    for cardinality in COLUMNAR_ACCEPTANCE_SCALES {
        let mut index = ArchetypeIndex::new();
        for value in 0..cardinality {
            let component_id = ComponentId::new(value);
            let archetype =
                index.id_or_insert(ArchetypeSignature::new(Vec::new(), vec![component_id]), []);
            let row = index
                .preflight_row(
                    archetype,
                    std::iter::empty::<(
                        ComponentId,
                        Box<dyn std::any::Any + Send + Sync>,
                        ComponentTicks,
                    )>(),
                )
                .unwrap();
            index.append_preflighted_row(archetype, value as EntityId + 1, row);
        }

        let target = ComponentId::new(cardinality - 1);
        let mut lookup_timings = Vec::with_capacity(P95_SAMPLE_COUNT);
        for _ in 0..P95_SAMPLE_COUNT {
            let start = Instant::now();
            let matches = index.matching_archetypes(&[target], &[]);
            lookup_timings.push(start.elapsed());
            assert_eq!(matches.len(), 1);
            assert_eq!(index.entities(matches[0]).unwrap().len(), 1);
        }

        let stats = index.performance_stats();
        let sample = ArchetypeCardinalityAcceptanceSample {
            entities: cardinality,
            components: cardinality,
            archetypes: index.len() - 1,
            component_index_probes: stats.component_index_probes,
            signature_membership_checks: stats.signature_membership_checks,
            row_appends: stats.row_appends,
            retained_bytes: index.estimated_heap_bytes(),
            p95: p95(lookup_timings),
        };
        assert_eq!(sample.entities, cardinality);
        assert_eq!(sample.components, cardinality);
        assert_eq!(sample.archetypes, cardinality);
        assert_eq!(sample.component_index_probes, P95_SAMPLE_COUNT as u64);
        assert_eq!(sample.signature_membership_checks, P95_SAMPLE_COUNT as u64);
        assert_eq!(sample.row_appends, cardinality as u64);
        assert!(sample.retained_bytes > 0);
        assert_ne!(sample.p95, Duration::MAX);
        eprintln!("ecs archetype cardinality acceptance: {sample:?}");
    }
}
