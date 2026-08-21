use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::navigation::{
    AREA_WALKABLE, DEFAULT_AGENT_TYPE, NavAgentTickReport, NavMeshAsset, NavigationDebugCapture,
    NavigationManager,
};
use zircon_runtime::scene::World;

use crate::DefaultNavigationManager;
use crate::plugin::navigation_overlay_frame_if_enabled;
use crate::tests::support::two_island_navmesh;

const BENCHMARK_SAMPLES: usize = 21;

#[test]
fn manager_overlay_frame_projects_loaded_navmesh_and_owner_generation() {
    let manager = DefaultNavigationManager::new();
    let first = manager.navigation_overlay_frame(NavAgentTickReport::default());
    assert_eq!(first.owner_generation, 0);
    assert!(first.nav_mesh.triangles.is_empty());

    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(true)).unwrap();
    let frame = manager.navigation_overlay_frame(NavAgentTickReport {
        moved_agents: 3,
        ..NavAgentTickReport::default()
    });
    assert_eq!(frame.owner_generation, 1);
    assert_eq!(frame.nav_mesh.triangles.len(), 4);
    assert_eq!(frame.nav_mesh.off_mesh_links.len(), 1);
    assert_eq!(frame.tick_report.moved_agents, 3);

    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(false)).unwrap();
    assert_eq!(
        manager
            .navigation_overlay_frame(NavAgentTickReport::default())
            .owner_generation,
        2
    );
}

#[test]
fn loaded_asset_snapshots_share_the_immutable_navmesh_allocation() {
    let manager = DefaultNavigationManager::new();
    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(true)).unwrap();

    let first = manager.loaded_assets();
    let second = manager.loaded_assets();

    assert_eq!(first.len(), 1);
    assert!(Arc::ptr_eq(&first[0].1, &second[0].1));
}

#[test]
fn overlay_frame_is_projected_only_while_debug_capture_is_enabled() {
    let manager = DefaultNavigationManager::new();
    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(true)).unwrap();
    let mut world = World::empty();
    world.insert_resource(NavigationDebugCapture { enabled: false });

    assert!(
        navigation_overlay_frame_if_enabled(&manager, &world, &NavAgentTickReport::default())
            .is_none()
    );

    world.resource_mut::<NavigationDebugCapture>().enabled = true;
    let frame =
        navigation_overlay_frame_if_enabled(&manager, &world, &NavAgentTickReport::default())
            .expect("active debug reader receives a frame");
    assert_eq!(frame.nav_mesh.triangles.len(), 4);
    assert_eq!(frame.nav_mesh.off_mesh_links.len(), 1);
}

#[test]
#[ignore = "release-only performance evidence"]
fn arc_loaded_navmesh_snapshot_release_benchmark_evidence() {
    const ASSET_COUNT: usize = 16;
    const TRIANGLES_PER_ASSET: usize = 8_192;
    const ITERATIONS: usize = 4;

    let manager = DefaultNavigationManager::new();
    for _ in 0..ASSET_COUNT {
        NavigationManager::load_nav_mesh(&manager, large_navmesh(TRIANGLES_PER_ASSET)).unwrap();
    }
    let shared = manager.loaded_assets();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..ITERATIONS {
                black_box(
                    shared
                        .iter()
                        .map(|(handle, asset)| (*handle, asset.as_ref().clone()))
                        .collect::<Vec<_>>(),
                );
            }
        },
        || {
            for _ in 0..ITERATIONS {
                black_box(manager.loaded_assets());
            }
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let ratio = optimized_p95 as f64 / legacy_p95.max(1) as f64;
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    println!(
        "PERF_RESULT plugins14_arc_loaded_navmesh_snapshot assets={} triangles_per_asset={} iterations={} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_ratio={ratio:.6} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        ASSET_COUNT,
        TRIANGLES_PER_ASSET,
        ITERATIONS,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(5) <= legacy_p95,
        "Arc asset snapshot P95 must be at most 20% of deep-clone P95"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn demand_driven_overlay_frame_release_benchmark_evidence() {
    const TRIANGLE_COUNT: usize = 32_768;
    const ITERATIONS: usize = 4;

    let manager = DefaultNavigationManager::new();
    NavigationManager::load_nav_mesh(&manager, large_navmesh(TRIANGLE_COUNT)).unwrap();
    let mut world = World::empty();
    world.insert_resource(NavigationDebugCapture { enabled: false });
    let report = NavAgentTickReport::default();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..ITERATIONS {
                black_box(manager.navigation_overlay_frame(report.clone()));
            }
        },
        || {
            for _ in 0..ITERATIONS {
                black_box(navigation_overlay_frame_if_enabled(
                    &manager, &world, &report,
                ));
            }
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let ratio = optimized_p95 as f64 / legacy_p95.max(1) as f64;
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    println!(
        "PERF_RESULT plugins14_demand_overlay_frame triangles={} iterations={} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_ratio={ratio:.6} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        TRIANGLE_COUNT, ITERATIONS, legacy_p50, legacy_p95, optimized_p50, optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95,
        "disabled overlay P95 must be at most 1% of unconditional projection P95"
    );
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn large_navmesh(triangle_count: usize) -> NavMeshAsset {
    let indices = (0..triangle_count)
        .flat_map(|_| [0_u32, 1, 2])
        .collect::<Vec<_>>();
    NavMeshAsset::from_triangle_mesh(
        DEFAULT_AGENT_TYPE,
        vec![[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
        indices,
        AREA_WALKABLE,
    )
}

fn benchmark_paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample_index in 0..BENCHMARK_SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(&result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    assert!(!ordered.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
