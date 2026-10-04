use std::hint::black_box;
use std::time::Instant;

use super::super::math::distance_xz;
use super::BuiltinNavigationState;
use crate::core::math::{Real, Vec3};

const SAMPLE_PAIRS: usize = 31;
const ROUTE_COUNT: usize = 4_096;
const LOOKUP_PASSES: usize = 16;

#[test]
fn optimization_batch_20260829au_runtime321_matching_repath_route_returns_cached_waypoint() {
    let mut state = BuiltinNavigationState::default();
    state.store_repath_route(7, destination(), "ground".to_owned(), 3, vec![waypoint()]);

    assert_eq!(
        state.cached_repath_target(7, Vec3::ZERO, destination(), "ground", 3, 0.0),
        Some(waypoint())
    );
    assert_eq!(state.repath_routes.len(), 1);
}

#[test]
fn optimization_batch_20260829au_runtime321_mismatched_repath_route_is_removed() {
    let mut state = BuiltinNavigationState::default();
    state.store_repath_route(7, destination(), "ground".to_owned(), 3, vec![waypoint()]);

    assert_eq!(
        state.cached_repath_target(7, Vec3::ZERO, destination(), "flying", 3, 0.0),
        None
    );
    assert!(state.repath_routes.is_empty());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829au_runtime321_single_lookup_repath_route_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let mut legacy = populated_state();
        let mut optimized = populated_state();
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut legacy, false));
            optimized_samples.push(measure(&mut optimized, true));
        } else {
            optimized_samples.push(measure(&mut optimized, true));
            legacy_samples.push(measure(&mut legacy, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME321_SINGLE_LOOKUP_REPATH_ROUTE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
routes={ROUTE_COUNT} lookup_passes={LOOKUP_PASSES} legacy_hash_lookups_per_hit=2 \
optimized_hash_lookups_per_hit=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn populated_state() -> BuiltinNavigationState {
    let mut state = BuiltinNavigationState::default();
    for entity in 0..ROUTE_COUNT as u64 {
        state.store_repath_route(
            entity,
            destination(),
            "ground".to_owned(),
            3,
            vec![waypoint()],
        );
    }
    state
}

fn measure(state: &mut BuiltinNavigationState, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUP_PASSES {
        for entity in 0..ROUTE_COUNT as u64 {
            let target = if optimized {
                state.cached_repath_target(
                    black_box(entity),
                    Vec3::ZERO,
                    destination(),
                    "ground",
                    3,
                    0.0,
                )
            } else {
                legacy_cached_repath_target(
                    state,
                    black_box(entity),
                    Vec3::ZERO,
                    destination(),
                    "ground",
                    3,
                    0.0,
                )
            };
            checksum ^= usize::from(target.is_some());
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_cached_repath_target(
    state: &mut BuiltinNavigationState,
    entity: u64,
    current: Vec3,
    destination: Vec3,
    agent_type: &str,
    area_mask: u64,
    stopping_distance: Real,
) -> Option<Vec3> {
    let matches_request = state.repath_routes.get(&entity).is_some_and(|route| {
        route.destination == destination
            && route.agent_type == agent_type
            && route.area_mask == area_mask
    });
    if !matches_request {
        state.repath_routes.remove(&entity);
        return None;
    }

    let route = state
        .repath_routes
        .get_mut(&entity)
        .expect("matching repath route must remain present");
    while route.next_waypoint < route.waypoints.len()
        && distance_xz(current, route.waypoints[route.next_waypoint]) <= stopping_distance.max(0.0)
    {
        route.next_waypoint += 1;
    }
    if let Some(target) = route.waypoints.get(route.next_waypoint).copied() {
        return Some(target);
    }
    state.repath_routes.remove(&entity);
    None
}

fn destination() -> Vec3 {
    Vec3::new(120.0, 0.0, 80.0)
}

fn waypoint() -> Vec3 {
    Vec3::new(60.0, 0.0, 40.0)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
