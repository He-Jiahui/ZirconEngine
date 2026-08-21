use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::navigation::NavMeshAgentDescriptor;
use zircon_runtime::core::framework::navigation::{NavMeshAreaCostAsset, NavMeshAsset};

use crate::{RecastCrowd, RecastCrowdConfig};

#[test]
fn crowd_update_round_trips_agent_states() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: 8,
            max_agent_radius: 1.0,
        },
    )
    .expect("create crowd");
    let handle = crowd
        .add_agent([-5.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add agent");
    crowd
        .set_target(handle, [5.0, 0.0, 0.0])
        .expect("set target");

    for _ in 0..8 {
        crowd.update(0.1).expect("update crowd");
    }

    let states = crowd.read_states().expect("read crowd states");
    let state = states
        .iter()
        .find(|state| state.handle == handle)
        .expect("agent state was returned in the batch");
    assert!(
        state.position[0] > -5.0,
        "agent should advance toward target"
    );
    assert!(state.velocity[0] > 0.0);
    assert!(state.desired_velocity[0] > 0.0);
}

#[test]
fn crowd_rejects_agent_area_mask_that_excludes_the_surface() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd =
        RecastCrowd::from_asset(&asset, RecastCrowdConfig::default()).expect("create crowd");
    let error = crowd
        .add_agent(
            [0.0, 0.0, 0.0],
            &NavMeshAgentDescriptor {
                area_mask: 0,
                ..NavMeshAgentDescriptor::default()
            },
        )
        .expect_err("empty area mask must not silently use the default filter");

    assert!(error.to_string().contains("area mask"));
}

#[test]
fn crowd_syncs_controller_owned_position_into_the_corridor() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd =
        RecastCrowd::from_asset(&asset, RecastCrowdConfig::default()).expect("create crowd");
    let handle = crowd
        .add_agent([-4.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add agent");
    crowd
        .set_target(handle, [4.0, 0.0, 0.0])
        .expect("set target");
    crowd
        .sync_agent_position(handle, [-2.0, 0.0, 1.0])
        .expect("sync controller position");

    let state = crowd
        .read_states()
        .expect("read states")
        .into_iter()
        .find(|state| state.handle == handle)
        .expect("agent state");
    assert!((state.position[0] + 2.0).abs() < 0.01);
    assert!((state.position[2] - 1.0).abs() < 0.01);
}

#[test]
fn crowd_recycles_inactive_query_filter_slots() {
    let mut asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    asset.area_costs = (0_u8..18)
        .map(|area| NavMeshAreaCostAsset {
            area,
            cost: 1.0 + f32::from(area) * 0.1,
            walkable: true,
        })
        .collect();
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: 32,
            max_agent_radius: 1.0,
        },
    )
    .expect("create crowd");
    let mut handles = Vec::new();
    for area in 1_u8..=16 {
        handles.push(
            crowd
                .add_agent(
                    [0.0, 0.0, 0.0],
                    &NavMeshAgentDescriptor {
                        area_mask: (1_u64 << 1) | (1_u64 << area),
                        ..NavMeshAgentDescriptor::default()
                    },
                )
                .expect("allocate distinct active filter"),
        );
    }
    crowd
        .remove_agent(handles[0])
        .expect("release first filter");

    crowd
        .add_agent(
            [0.0, 0.0, 0.0],
            &NavMeshAgentDescriptor {
                area_mask: (1_u64 << 1) | (1_u64 << 17),
                ..NavMeshAgentDescriptor::default()
            },
        )
        .expect("inactive filter slot should be reusable");
}

#[test]
fn crowd_read_states_reuses_native_capacity_scratch() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: 128,
            max_agent_radius: 1.0,
        },
    )
    .expect("create crowd");
    crowd
        .add_agent([0.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add agent");
    let initial_scratch = crowd.native_state_scratch_identity();

    assert_eq!(crowd.read_states().expect("first state read").len(), 1);
    let after_first_read = crowd.native_state_scratch_identity();
    assert_eq!(crowd.read_states().expect("second state read").len(), 1);
    let after_second_read = crowd.native_state_scratch_identity();

    assert_eq!(initial_scratch, after_first_read);
    assert_eq!(after_first_read, after_second_read);
    assert_eq!(after_second_read.1, crowd.capacity());
}

const CROWD_BENCHMARK_SAMPLE_PAIRS: usize = 21;
const CROWD_BENCHMARK_ITERATIONS: usize = 64;
const CROWD_BENCHMARK_CAPACITY: usize = 4_096;

#[test]
#[ignore = "release performance evidence"]
fn crowd_state_scratch_release_gate() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut crowd = RecastCrowd::from_asset(
        &asset,
        RecastCrowdConfig {
            max_agents: u32::try_from(CROWD_BENCHMARK_CAPACITY).unwrap(),
            max_agent_radius: 1.0,
        },
    )
    .expect("create benchmark crowd");
    crowd
        .add_agent([0.0, 0.0, 0.0], &NavMeshAgentDescriptor::default())
        .expect("add benchmark agent");

    let legacy_states = crowd
        .read_states_legacy_for_benchmark()
        .expect("legacy state read");
    let optimized_states = crowd.read_states().expect("optimized state read");
    assert_eq!(legacy_states, optimized_states);
    for _ in 0..16 {
        black_box(crowd.read_states_legacy_for_benchmark().unwrap());
        black_box(crowd.read_states().unwrap());
    }

    let mut legacy_samples_ns = Vec::with_capacity(CROWD_BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples_ns = Vec::with_capacity(CROWD_BENCHMARK_SAMPLE_PAIRS);
    for pair_index in 0..CROWD_BENCHMARK_SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_ns.push(measure_crowd_reads(&crowd, false));
            optimized_samples_ns.push(measure_crowd_reads(&crowd, true));
        } else {
            optimized_samples_ns.push(measure_crowd_reads(&crowd, true));
            legacy_samples_ns.push(measure_crowd_reads(&crowd, false));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples_ns, 95);
    assert!(
        u128::from(optimized_p95_ns) * 100 <= u128::from(legacy_p95_ns) * 110,
        "scratch P95 {optimized_p95_ns}ns exceeded the 10% regression ceiling over legacy P95 {legacy_p95_ns}ns"
    );

    println!(
        "PERF-MVP-PLUGINS14-CROWD-STATE-SCRATCH capacity={CROWD_BENCHMARK_CAPACITY} active_agents=1 iterations_per_sample={CROWD_BENCHMARK_ITERATIONS} sample_pairs={CROWD_BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even percentile_method=nearest_rank legacy_native_allocations_per_sample={CROWD_BENCHMARK_ITERATIONS} optimized_native_allocations_per_sample=0 legacy_native_default_writes_per_sample={} optimized_native_default_writes_per_sample=0 allocation_reduction_pct=100 default_write_reduction_pct=100 legacy_samples_ns={} optimized_samples_ns={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} target_ratio_pct=110",
        CROWD_BENCHMARK_CAPACITY * CROWD_BENCHMARK_ITERATIONS,
        join_samples(&legacy_samples_ns),
        join_samples(&optimized_samples_ns),
    );
}

fn measure_crowd_reads(crowd: &RecastCrowd, optimized: bool) -> u64 {
    let started = Instant::now();
    for _ in 0..CROWD_BENCHMARK_ITERATIONS {
        let states = if optimized {
            crowd.read_states().unwrap()
        } else {
            crowd.read_states_legacy_for_benchmark().unwrap()
        };
        black_box(states);
    }
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
