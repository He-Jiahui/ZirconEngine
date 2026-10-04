use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const REGISTRATION_PLANS_PER_SAMPLE: usize = 16_384;

#[test]
fn canonical_descriptor_builder_preserves_gltf_contract() {
    let descriptors = asset_importer_descriptors();

    assert_eq!(descriptors.len(), 1);
    let descriptor = &descriptors[0];
    assert_eq!(descriptor.id, "gltf_importer.gltf");
    assert_eq!(descriptor.importer_version, GLTF_IMPORTER_VERSION);
    assert_eq!(descriptor.priority, 120);
    assert_eq!(descriptor.source_extensions, ["gltf", "glb"]);
    assert_eq!(
        descriptor.additional_output_kinds,
        [
            AssetKind::Mesh,
            AssetKind::Scene,
            AssetKind::Material,
            AssetKind::Texture,
            AssetKind::Data,
            AssetKind::AnimationSkeleton,
            AssetKind::AnimationClip,
        ]
    );
    assert_eq!(descriptor.required_capabilities, [IMPORTER_CAPABILITY]);
}

#[test]
fn plugin_toml_preserves_the_versioned_gltf_output_schema() {
    let manifest = include_str!("../../../plugin.toml")
        .parse::<toml::Table>()
        .expect("glTF plugin manifest parses as TOML");
    let importers = manifest
        .get("asset_importers")
        .and_then(toml::Value::as_array)
        .expect("glTF plugin manifest declares asset importers");
    assert_eq!(importers.len(), 1);
    let importer = importers[0]
        .as_table()
        .expect("glTF asset importer is a TOML table");
    let output_kinds = importer
        .get("additional_output_kinds")
        .and_then(toml::Value::as_array)
        .expect("glTF asset importer declares every subasset kind")
        .iter()
        .map(|kind| kind.as_str().expect("output kind is a string"))
        .collect::<Vec<_>>();

    assert_eq!(GLTF_IMPORTER_VERSION, 4);
    assert_eq!(
        importer
            .get("importer_version")
            .and_then(toml::Value::as_integer),
        Some(i64::from(GLTF_IMPORTER_VERSION))
    );
    assert_eq!(
        output_kinds,
        [
            "Mesh",
            "Scene",
            "Material",
            "Texture",
            "Data",
            "AnimationSkeleton",
            "AnimationClip",
        ]
    );
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_direct_gltf_importer_registration_plan() {
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);

    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_registration_plans(legacy_registration_plan));
            optimized_raw.push(measure_registration_plans(direct_registration_plan));
        } else {
            optimized_raw.push(measure_registration_plans(direct_registration_plan));
            legacy_raw.push(measure_registration_plans(legacy_registration_plan));
        }
    }

    emit_performance_result(
        "plugins07_direct_gltf_importer_registration",
        legacy_raw,
        optimized_raw,
    );
}

fn legacy_registration_plan() -> usize {
    let mut descriptors = black_box(asset_importer_descriptors()).into_iter();
    consume_descriptor(descriptors.next().expect("one glTF importer descriptor"))
}

fn direct_registration_plan() -> usize {
    consume_descriptor(gltf_importer_descriptor())
}

fn consume_descriptor(descriptor: AssetImporterDescriptor) -> usize {
    let descriptor = black_box(descriptor);
    black_box(
        descriptor.source_extensions.len()
            + descriptor.additional_output_kinds.len()
            + descriptor.required_capabilities.len(),
    )
}

fn measure_registration_plans(plan: fn() -> usize) -> u64 {
    let started = Instant::now();
    let mut checksum = 0;
    for _ in 0..REGISTRATION_PLANS_PER_SAMPLE {
        checksum ^= plan();
    }
    black_box(checksum);
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn emit_performance_result(task: &str, legacy_raw: Vec<u64>, optimized_raw: Vec<u64>) {
    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
        "direct importer registration plan must improve P95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "PERF_RESULT task={task} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=1 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u64]) -> String {
    let values = samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}
