use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const RECORDS: usize = 32_768;
const UNIQUE_SCENES: usize = 8_192;

fn scene_ids() -> Vec<ResourceId> {
    (0..RECORDS)
        .map(|index| {
            ResourceId::from_stable_label(&format!(
                "scene.management.{:05}",
                (index * 4_099) % UNIQUE_SCENES
            ))
        })
        .collect()
}

fn ordered_scene_count(ids: &[ResourceId]) -> usize {
    let mut scenes = BTreeSet::new();
    ids.iter().filter(|id| scenes.insert(**id)).count()
}

fn hash_scene_count(ids: &[ResourceId]) -> usize {
    let mut scenes = HashSet::with_capacity(ids.len());
    ids.iter().filter(|id| scenes.insert(**id)).count()
}

#[test]
fn optimization_batch_id_runtime614_scene_count_preserves_duplicate_scene_collapsing() {
    let ids = scene_ids();
    assert_eq!(ordered_scene_count(&ids), UNIQUE_SCENES);
    assert_eq!(hash_scene_count(&ids), UNIQUE_SCENES);
    let records = ids
        .iter()
        .enumerate()
        .map(|(index, scene_id)| SceneEntityManagementRecord {
            scene_id: *scene_id,
            entity: SceneEntityOverview {
                entity: index as u64,
                name: String::new(),
                parent: None,
                active: false,
                render_layer_mask: 0,
                mobility: super::super::SceneMobilityAsset::default(),
                direct_reference_count: 0,
                has_camera: false,
                has_mesh: false,
                has_direct_mesh_reference: false,
                direct_mesh_reference_count: 0,
                mesh_primitive_binding_count: 0,
                morph_weight_count: 0,
                has_ambient_light: false,
                has_directional_light: false,
                has_point_light: false,
                has_rect_light: false,
                has_spot_light: false,
                has_post_process_settings: false,
                has_post_process_volume: false,
                has_rigid_body: false,
                has_collider: false,
                has_collider_material: false,
                has_joint: false,
                has_animation_skeleton: false,
                has_animation_player: false,
                has_animation_sequence_player: false,
                has_animation_graph_player: false,
                has_animation_state_machine_player: false,
                has_terrain: false,
                has_tilemap: false,
                has_prefab_instance: false,
            },
        })
        .collect::<Vec<_>>();
    assert_eq!(
        SceneEntityManagementRecordSetSummary::from_records(&records).scene_count,
        UNIQUE_SCENES
    );
}

#[test]
fn optimization_batch_id_runtime614_scene_count_uses_hash_membership_without_order_dependency() {
    let source = include_str!("../management.rs");
    let production = source
        .split("impl SceneEntityManagementRecordSetSummary")
        .nth(1)
        .unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production.contains("HashSet::with_capacity(records.len())"));
    assert!(production.contains("scene_ids.insert(record.scene_id)"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_id_runtime614_scene_count_hash_membership_performance_evidence() {
    let ids = scene_ids();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_scene_count(black_box(&ids)));
            ordered_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(hash_scene_count(black_box(&ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_scene_count(black_box(&ids)));
            hash_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(ordered_scene_count(black_box(&ids)));
            ordered_samples.push(started.elapsed());
        }
    }
    ordered_samples.sort_unstable();
    hash_samples.sort_unstable();
    let ordered_p95 = ordered_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    let hash_p95 = hash_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    println!(
        "RUNTIME614_HASH_SCENE_COUNT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} records={RECORDS} unique_scenes={UNIQUE_SCENES} ordered_p95_ns={} hash_p95_ns={} target_ratio_bp=5000",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95 <= ordered_p95.mul_f64(0.5),
        "hash scene-count P95 {:?} exceeded 50% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
