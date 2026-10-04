#[cfg(target_os = "windows")]
use std::hint::black_box;
#[cfg(target_os = "windows")]
use std::time::Instant;

use crate::core::math::{Transform, Vec3};
#[cfg(target_os = "windows")]
use crate::scene::components::NodeRecord;
use crate::scene::components::{ActiveInHierarchy, NodeKind, WorldMatrix};
use crate::scene::World;

#[test]
fn clean_world_matrix_reads_match_the_published_cache() {
    let world = World::new();
    let entity = world.active_camera();
    assert!(!world.has_pending_scene_systems());

    let published = world
        .get::<WorldMatrix>(entity)
        .expect("World::new must publish the camera's derived matrix")
        .0;
    assert_eq!(world.world_matrix(entity), Some(published));
}

#[test]
fn missing_or_dirty_world_matrix_cache_keeps_projection_behavior() {
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("parent fixture should spawn");
    let child = world
        .spawn_node(NodeKind::Empty)
        .expect("child fixture should spawn");
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .expect("parent transform should update");
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .expect("child transform should update");
    world
        .set_parent_checked(child, Some(parent))
        .expect("child should attach to parent");

    assert!(world.get::<WorldMatrix>(child).is_none());
    assert_eq!(world.world_matrix(child).unwrap().w_axis.x, 3.0);
    world.flush_pending_scene_systems();

    let committed = world
        .get::<WorldMatrix>(child)
        .expect("flush should publish the child matrix")
        .0;
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
        )
        .expect("parent transform should update again");
    assert!(world.has_pending_scene_systems());

    let projected = world
        .world_matrix(child)
        .expect("dirty value should project");
    assert_eq!(projected.w_axis.x, 6.0);
    assert_ne!(projected, committed);
    world.flush_pending_scene_systems();
    assert_eq!(world.world_matrix(child), Some(projected));
}

#[cfg(target_os = "windows")]
const CLEAN_CHAIN_DEPTHS: [usize; 3] = [1, 32, 1_024];
#[cfg(target_os = "windows")]
const PROFILE_SAMPLE_COUNT: usize = 31;
#[cfg(target_os = "windows")]
const QUERIES_PER_SAMPLE: usize = 256;

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct ProfileSample {
    elapsed_ns: u64,
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "managed Windows Release raw latency evidence for Runtime62 G19"]
fn runtime62_clean_world_matrix_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    for depth in CLEAN_CHAIN_DEPTHS {
        let (world, leaf) = committed_chain_world(depth);
        let cached = world
            .get::<WorldMatrix>(leaf)
            .expect("clean profile fixture must publish its leaf matrix")
            .0;
        assert!(!world.has_pending_scene_systems());
        assert_eq!(world.world_matrix(leaf), Some(cached));

        let samples = collect_clean_samples(|| {
            black_box(world.world_matrix(leaf));
        });
        report_clean_samples(
            "RUNTIME62_CLEAN_WORLD_MATRIX_LATENCY_PROFILE_V1",
            "world_matrix",
            depth,
            &samples,
        );
    }
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "managed Windows Release raw latency evidence for Runtime62 G19 clean active reads"]
fn runtime62_clean_active_in_hierarchy_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this ignored profile with a Release build"
    );

    for depth in CLEAN_CHAIN_DEPTHS {
        let (world, leaf) = committed_chain_world(depth);
        let cached = world
            .get::<ActiveInHierarchy>(leaf)
            .expect("clean profile fixture must publish its leaf active state")
            .0;
        assert!(!world.has_pending_scene_systems());
        assert_eq!(world.active_in_hierarchy(leaf), Some(cached));

        let samples = collect_clean_samples(|| {
            black_box(world.active_in_hierarchy(leaf));
        });
        report_clean_samples(
            "RUNTIME62_CLEAN_ACTIVE_LATENCY_PROFILE_V1",
            "active_in_hierarchy",
            depth,
            &samples,
        );
    }
}

#[cfg(target_os = "windows")]
fn committed_chain_world(depth: usize) -> (World, u64) {
    assert!(depth > 0);
    let mut template_world = World::empty();
    let template_entity = template_world
        .spawn_node(NodeKind::Empty)
        .expect("profile template node should spawn");
    let template = template_world
        .node_record(template_entity)
        .expect("profile template node should have a record");

    let first_id = 100_000_u64;
    let mut records = Vec::with_capacity(depth);
    for index in 0..depth {
        let mut record: NodeRecord = template.clone();
        record.id = first_id + index as u64;
        record.name = format!("Clean matrix profile {index}");
        record.parent = index.checked_sub(1).map(|parent| first_id + parent as u64);
        record.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        records.push(record);
    }
    let leaf = records.last().expect("depth is nonzero").id;

    let mut world = World::empty();
    world
        .insert_node_records(&records)
        .expect("profile hierarchy records should publish");
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    assert!(world.get::<WorldMatrix>(leaf).is_some());
    (world, leaf)
}

#[cfg(target_os = "windows")]
fn collect_clean_samples(mut query: impl FnMut()) -> Vec<ProfileSample> {
    let mut samples = Vec::with_capacity(PROFILE_SAMPLE_COUNT);
    for _ in 0..PROFILE_SAMPLE_COUNT {
        let started = Instant::now();
        for _ in 0..QUERIES_PER_SAMPLE {
            query();
        }
        let elapsed_ns = started.elapsed().as_nanos() as u64;
        samples.push(ProfileSample { elapsed_ns });
    }
    samples
}

#[cfg(target_os = "windows")]
fn report_clean_samples(marker: &str, query: &str, depth: usize, samples: &[ProfileSample]) {
    let elapsed_ns = samples
        .iter()
        .map(|sample| sample.elapsed_ns)
        .collect::<Vec<_>>();
    let latency_ns_per_query = elapsed_ns
        .iter()
        .map(|elapsed| elapsed / QUERIES_PER_SAMPLE as u64)
        .collect::<Vec<_>>();
    let mut ordered_latency = latency_ns_per_query.clone();
    ordered_latency.sort_unstable();

    println!(
        "{marker} query={query} depth={depth} samples={} queries_per_sample={QUERIES_PER_SAMPLE} latency_ns_per_query_p50={} latency_ns_per_query_p95={} latency_ns_per_query_p99={} elapsed_ns_raw={elapsed_ns:?} latency_ns_per_query_raw={latency_ns_per_query:?}",
        samples.len(),
        nearest_rank_percentile(&ordered_latency, 50),
        nearest_rank_percentile(&ordered_latency, 95),
        nearest_rank_percentile(&ordered_latency, 99),
    );
}

#[cfg(target_os = "windows")]
fn nearest_rank_percentile(sorted_samples: &[u64], percentile: usize) -> u64 {
    let rank = (sorted_samples.len() * percentile).div_ceil(100);
    sorted_samples[rank.saturating_sub(1)]
}
