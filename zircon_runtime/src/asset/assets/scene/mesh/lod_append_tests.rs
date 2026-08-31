use std::hint::black_box;
use std::time::Instant;

use crate::asset::AssetUri;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 64;
const LODS_PER_BUILD: usize = 256;
const REFERENCES_PER_LOD: usize = 16;

#[test]
fn optimization_batch_20260826gp_runtime236_lod_append_preserves_reference_order() {
    let model = reference("res://models/lod.zmodel");
    let mesh = reference("res://meshes/lod.zmesh");
    let material = reference("res://materials/lod.zmaterial");
    let primitive_mesh = reference("res://meshes/primitive.zmesh");
    let primitive_material = reference("res://materials/primitive.zmaterial");
    let lod = SceneMeshLodLevelAsset {
        min_distance: 10.0,
        model: model.clone(),
        mesh: Some(mesh.clone()),
        material: material.clone(),
        primitives: vec![SceneMeshPrimitiveBindingAsset {
            mesh: primitive_mesh.clone(),
            material: primitive_material.clone(),
        }],
    };
    let expected = vec![model, mesh, material, primitive_mesh, primitive_material];

    let mut appended = vec![reference("res://models/root.zmodel")];
    lod.append_direct_references(&mut appended);

    assert_eq!(&appended[1..], expected.as_slice());
    assert_eq!(lod.direct_references(), expected);
    assert_eq!(lod.direct_reference_count(), 5);

    let root_model = reference("res://models/root.zmodel");
    let root_material = reference("res://materials/root.zmaterial");
    let instance = SceneMeshInstanceAsset {
        model: root_model.clone(),
        mesh: None,
        material: root_material.clone(),
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        morph_weights: Vec::new(),
        primitives: Vec::new(),
        lods: vec![lod],
    };
    let mut expected_instance = vec![root_model, root_material];
    expected_instance.extend(expected);

    assert_eq!(instance.direct_references(), expected_instance);
    assert_eq!(instance.direct_reference_count(), 7);
}

#[test]
fn optimization_batch_20260826gp_runtime236_instance_appends_lods_without_temporary_vecs() {
    let source = include_str!("../mesh.rs");

    assert!(source.contains("lod.append_direct_references(&mut references);"));
    assert!(source.contains("self.append_direct_references(&mut references);"));
    assert!(source.contains("Vec::with_capacity(self.direct_reference_count())"));
    assert!(!source.contains("references.extend(lod.direct_references())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gp_runtime236_scene_mesh_lod_direct_append_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME236_SCENE_MESH_LOD_DIRECT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} lods_per_build={LODS_PER_BUILD} \
references_per_lod={REFERENCES_PER_LOD} reference_payload_usize_fields=8 \
legacy_temporary_vec_allocations_per_build={LODS_PER_BUILD} \
optimized_temporary_vec_allocations_per_build=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("valid asset locator"))
}

fn measure(direct_append: bool) -> u128 {
    let total_references = LODS_PER_BUILD * REFERENCES_PER_LOD;
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let mut references = Vec::with_capacity(total_references);
        for lod in 0..LODS_PER_BUILD {
            if direct_append {
                append_payloads(&mut references, build, lod);
            } else {
                let mut temporary = Vec::with_capacity(REFERENCES_PER_LOD);
                append_payloads(&mut temporary, build, lod);
                references.extend(temporary);
            }
        }
        checksum ^= black_box(references.len() ^ references.capacity() ^ build);
        black_box(references);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn append_payloads(out: &mut Vec<[usize; 8]>, build: usize, lod: usize) {
    for reference in 0..REFERENCES_PER_LOD {
        let value = black_box(
            build * LODS_PER_BUILD * REFERENCES_PER_LOD + lod * REFERENCES_PER_LOD + reference,
        );
        out.push([value; 8]);
    }
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
