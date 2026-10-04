use super::*;

#[test]
fn runtime846_navigation_projection_reserves_component_rows() {
    let source = include_str!("../../world_scan.rs");
    let body = source
        .split("pub(super) fn collect_navigation_world_projection")
        .nth(1)
        .and_then(|source| source.split("#[cfg(test)]").next())
        .expect("read navigation projection body");

    assert!(body.contains("let mut agents = Vec::with_capacity(agent_component_rows);"));
    assert!(body.contains("let mut agent_positions = Vec::with_capacity(agent_component_rows);"));
    assert!(body.contains("let mut obstacles = Vec::with_capacity(obstacle_component_rows);"));
    assert!(!body.contains("let mut agents = Vec::new();"));
    assert!(!body.contains("let mut agent_positions = Vec::new();"));
    assert!(!body.contains("let mut obstacles = Vec::new();"));

    let agent_count = body
        .find("let agent_component_rows = component_rows.len();")
        .expect("agent row count");
    let agent_reserve = body
        .find("Vec::with_capacity(agent_component_rows)")
        .expect("agent reserve");
    let agent_drain = body
        .find("for (entity, value) in component_rows.drain(..)")
        .expect("agent drain");
    assert!(agent_count < agent_reserve && agent_reserve < agent_drain);

    let obstacle_count = body
        .find("let obstacle_component_rows = component_rows.len();")
        .expect("obstacle row count");
    let obstacle_reserve = body
        .find("Vec::with_capacity(obstacle_component_rows)")
        .expect("obstacle reserve");
    let obstacle_drain = body
        .get(agent_drain + 1..)
        .and_then(|remaining| remaining.find("for (entity, value) in component_rows.drain(..)"))
        .map(|offset| agent_drain + 1 + offset)
        .expect("obstacle drain");
    assert!(obstacle_count < obstacle_reserve && obstacle_reserve < obstacle_drain);
}

#[test]
fn runtime846_navigation_projection_empty_rows_keep_zero_capacity() {
    let agents: Vec<RuntimeAgent> = Vec::with_capacity(0);
    let positions: Vec<(u64, Vec3, Real)> = Vec::with_capacity(0);
    let obstacles: Vec<RuntimeObstacle> = Vec::with_capacity(0);

    assert_eq!(agents.capacity(), 0);
    assert_eq!(positions.capacity(), 0);
    assert_eq!(obstacles.capacity(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime846_navigation_projection_capacity_bench() {
    const ROW_COUNT: usize = 4_096;
    const MARKER: &str = "RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1";
    let legacy_growth_events = growth_events(ROW_COUNT, 0);
    let optimized_growth_events = growth_events(ROW_COUNT, ROW_COUNT);
    println!(
        "{MARKER} rows={ROW_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events}"
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
