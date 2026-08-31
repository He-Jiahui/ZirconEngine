use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetReference, AssetUri};

use super::SceneAsset;
use crate::asset::assets::{
    SceneCameraAsset, SceneCameraTargetAsset, SceneEntityAsset, SceneMobilityAsset, TransformAsset,
};

const BENCH_ENTITY_COUNT: usize = 20_000;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime04_scene_reference_flatten_preserves_entity_order_and_duplicates() {
    let alpha = reference(0);
    let beta = reference(1);
    let scene = SceneAsset {
        entities: vec![
            entity(0, alpha.clone()),
            entity(1, beta.clone()),
            entity(2, alpha.clone()),
        ],
    };

    assert_eq!(scene.direct_references(), [alpha.clone(), beta, alpha]);
}

#[test]
#[ignore = "release-only scene dependency flatten benchmark"]
fn runtime04_scene_reference_flatten_release_benchmark_evidence() {
    let shared = reference(0);
    let scene = SceneAsset {
        entities: (0..BENCH_ENTITY_COUNT)
            .map(|index| entity(index as u64, shared.clone()))
            .collect(),
    };
    assert_eq!(legacy_flatten(&scene), scene.direct_references());

    let (legacy_samples, flattened_samples) =
        paired_samples(|| measure_legacy(&scene), || measure_flattened(&scene));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let flattened_p50_ns = percentile(&flattened_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let flattened_p95_ns = percentile(&flattened_samples, 95);

    println!(
        "PERF_RESULT plan=Runtime04 task=scene_direct_reference_flatten \
sample_pairs={SAMPLE_PAIRS} entity_count={BENCH_ENTITY_COUNT} references_per_entity=1 \
legacy_projection=per_entity_temporary_vec optimized_projection=single_preallocated_output \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={flattened_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={flattened_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&flattened_samples),
    );

    assert!(
        flattened_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "single-output scene reference flatten must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={flattened_p95_ns}ns"
    );
}

fn entity(entity: u64, texture: AssetReference) -> SceneEntityAsset {
    SceneEntityAsset {
        entity,
        name: format!("Camera {entity}"),
        parent: None,
        transform: TransformAsset::default(),
        active: true,
        render_layer_mask: u32::MAX,
        mobility: SceneMobilityAsset::Dynamic,
        camera: Some(SceneCameraAsset {
            target: SceneCameraTargetAsset::Texture { texture },
            ..SceneCameraAsset::default()
        }),
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}

fn reference(index: usize) -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse(&format!("res://textures/scene_{index:04}.ztexture"))
            .expect("benchmark asset URI should be valid"),
    )
}

fn legacy_flatten(scene: &SceneAsset) -> Vec<AssetReference> {
    scene
        .entities
        .iter()
        .flat_map(SceneEntityAsset::direct_references)
        .collect()
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_flattened: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_flattened());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut flattened_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            flattened_samples.push(measure_flattened());
        } else {
            flattened_samples.push(measure_flattened());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, flattened_samples)
}

fn measure_legacy(scene: &SceneAsset) -> u128 {
    let started = Instant::now();
    black_box(legacy_flatten(black_box(scene)));
    started.elapsed().as_nanos().max(1)
}

fn measure_flattened(scene: &SceneAsset) -> u128 {
    let started = Instant::now();
    black_box(black_box(scene).direct_references());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
