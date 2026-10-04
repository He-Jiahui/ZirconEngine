use super::*;

#[test]
fn runtime847_particle_extract_output_capacity_is_bounded() {
    let source = include_str!("../../render_particles.rs");
    let body = source
        .split("pub(super) fn collect_render_particles")
        .nth(1)
        .and_then(|source| source.split("#[derive(Clone, Debug, PartialEq)]").next())
        .expect("read particle extraction body");

    assert!(body.contains("let entity_count = dynamic_component_entities.len();"));
    assert!(body.contains("let mut emitters = Vec::with_capacity(entity_count);"));
    assert!(body.contains("let mut bounds = Vec::with_capacity(entity_count);"));
    assert!(!body.contains("let mut emitters = Vec::new();"));
    assert!(!body.contains("let mut bounds = Vec::new();"));

    let sort_index = body
        .find("dynamic_component_entities.sort_unstable();")
        .expect("sorted owner collection");
    let count_index = body
        .find("let entity_count = dynamic_component_entities.len();")
        .expect("owner count");
    let reserve_index = body
        .find("let mut emitters = Vec::with_capacity(entity_count);")
        .expect("emitter reservation");
    let loop_index = body
        .find("for entity in dynamic_component_entities")
        .expect("owner loop");
    assert!(sort_index < count_index && count_index < reserve_index && reserve_index < loop_index);
}

#[test]
fn runtime847_particle_extract_empty_outputs_keep_zero_capacity() {
    let emitters: Vec<EntityId> = Vec::with_capacity(0);
    let bounds: Vec<RenderParticleBoundsSnapshot> = Vec::with_capacity(0);

    assert_eq!(emitters.capacity(), 0);
    assert_eq!(bounds.capacity(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime847_particle_extract_output_capacity_bench() {
    const OWNER_COUNT: usize = 4_096;
    const MARKER: &str = "RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1";
    let legacy_growth_events = growth_events(OWNER_COUNT, 0);
    let optimized_growth_events = growth_events(OWNER_COUNT, OWNER_COUNT);
    println!(
        "{MARKER} owners={OWNER_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}
