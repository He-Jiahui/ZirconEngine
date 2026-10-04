use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const REGISTRATION_PLANS_PER_SAMPLE: usize = 4_096;

#[test]
fn canonical_descriptor_builders_preserve_public_order_and_capabilities() {
    let descriptors = asset_importer_descriptors();
    let observed = descriptors
        .iter()
        .map(|descriptor| {
            (
                descriptor.id.as_str(),
                descriptor.importer_version,
                descriptor.priority,
                descriptor.required_capabilities[0].as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        observed,
        [
            (
                "texture_importer.image",
                TEXTURE_IMPORTER_VERSION,
                120,
                IMAGE_IMPORTER_CAPABILITY,
            ),
            (
                "texture_importer.container",
                CONTAINER_IMPORTER_VERSION,
                90,
                CONTAINER_IMPORTER_CAPABILITY,
            ),
            (
                "texture_importer.psd",
                TEXTURE_IMPORTER_VERSION,
                100,
                PSD_IMPORTER_CAPABILITY,
            ),
            (
                "texture_importer.cubemap",
                CUBEMAP_IMPORTER_VERSION,
                130,
                CUBEMAP_IMPORTER_CAPABILITY,
            ),
            (
                "texture_importer.array",
                TEXTURE_IMPORTER_VERSION,
                130,
                ARRAY_IMPORTER_CAPABILITY,
            ),
            (
                "texture_importer.optional_native_container",
                TEXTURE_IMPORTER_VERSION,
                80,
                "runtime.asset.importer.native",
            ),
        ]
    );
}

#[test]
fn plugin_toml_preserves_texture_importer_versions() {
    assert_eq!(CONTAINER_IMPORTER_VERSION, 3);
    let manifest = include_str!("../../../plugin.toml")
        .parse::<toml::Table>()
        .expect("texture importer plugin manifest parses as TOML");
    let observed = manifest
        .get("asset_importers")
        .and_then(toml::Value::as_array)
        .expect("texture importer plugin manifest declares asset importers")
        .iter()
        .map(|importer| {
            let importer = importer
                .as_table()
                .expect("texture asset importer is a TOML table");
            (
                importer
                    .get("id")
                    .and_then(toml::Value::as_str)
                    .expect("texture asset importer has an id"),
                importer
                    .get("importer_version")
                    .and_then(toml::Value::as_integer)
                    .expect("texture asset importer has a version"),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        observed,
        [
            (
                "texture_importer.image",
                i64::from(TEXTURE_IMPORTER_VERSION)
            ),
            (
                "texture_importer.container",
                i64::from(CONTAINER_IMPORTER_VERSION),
            ),
            ("texture_importer.psd", i64::from(TEXTURE_IMPORTER_VERSION)),
            (
                "texture_importer.cubemap",
                i64::from(CUBEMAP_IMPORTER_VERSION),
            ),
            (
                "texture_importer.array",
                i64::from(TEXTURE_IMPORTER_VERSION)
            ),
            (
                "texture_importer.optional_native_container",
                i64::from(TEXTURE_IMPORTER_VERSION),
            ),
        ]
    );
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_direct_texture_importer_registration_plan() {
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
        "PERF_RESULT task=plugins07_direct_texture_importer_registration sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=6 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 legacy_string_dispatches_per_sample=24576 optimized_string_dispatches_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn legacy_registration_plan() -> usize {
    let mut checksum = 0;
    for importer in black_box(asset_importer_descriptors()) {
        let route = match black_box(importer.id.as_str()) {
            "texture_importer.image" => 1,
            "texture_importer.container" => 2,
            "texture_importer.psd" => 3,
            "texture_importer.cubemap" => 4,
            "texture_importer.array" => 5,
            "texture_importer.optional_native_container" => 6,
            _ => unreachable!(),
        };
        checksum += consume_descriptor(importer, route);
    }
    black_box(checksum)
}

fn direct_registration_plan() -> usize {
    let checksum = consume_descriptor(image_importer_descriptor(), 1)
        + consume_descriptor(container_importer_descriptor(), 2)
        + consume_descriptor(psd_importer_descriptor(), 3)
        + consume_descriptor(cubemap_importer_descriptor(), 4)
        + consume_descriptor(array_importer_descriptor(), 5)
        + consume_descriptor(optional_native_container_importer_descriptor(), 6);
    black_box(checksum)
}

fn consume_descriptor(descriptor: AssetImporterDescriptor, route: usize) -> usize {
    let descriptor = black_box(descriptor);
    black_box(descriptor.source_extensions.len() + route)
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
