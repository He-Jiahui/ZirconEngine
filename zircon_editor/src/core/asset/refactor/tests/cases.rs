use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::asset::mutation::AssetMutationDeletePreflight;
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

use super::{AssetDeleteDisposition, AssetDeletePreflight};
use crate::core::asset::AssetSourceWritePolicy;

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn entry(label: &str, path: &str, dependencies: Vec<AssetUuid>) -> AssetRegistryEntry {
    AssetRegistryEntry::new(uuid(label), uri(path), AssetKind::Data, label)
        .with_dependencies(dependencies)
}

fn registry(entries: impl IntoIterator<Item = AssetRegistryEntry>) -> AssetRegistryIndex {
    AssetRegistryIndex::from_entries(entries).unwrap()
}

#[test]
fn delete_preflight_allows_an_unreferenced_writable_project_asset() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(preflight.disposition(), AssetDeleteDisposition::Allowed);
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
    assert!(preflight.referencers().is_empty());
}

#[test]
fn delete_preflight_rejects_a_missing_asset_without_path_fallback() {
    let registry = registry([entry("occupied", "res://data/reused.json", Vec::new())]);
    let missing_uuid = uuid("deleted-asset");

    let preflight = AssetDeletePreflight::evaluate(
        &registry,
        missing_uuid,
        AssetSourceWritePolicy::ProjectOnly,
    );

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::MissingAsset
    );
    assert!(preflight.target().is_none());
    assert!(preflight.referencers().is_empty());
}

#[test]
fn delete_preflight_rejects_non_project_sources_even_for_mutable_asset_types() {
    let target = entry(
        "package-target",
        "package://sample/data/target.json",
        Vec::new(),
    );
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::ReadOnlySource
    );
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
}

#[test]
fn delete_preflight_rejects_labeled_subassets_without_a_source_mutation_plan() {
    let target = entry("mesh-subasset", "res://models/ship.glb#mesh", Vec::new());
    let target_uuid = target.uuid();
    let registry = registry([target]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::UnsupportedSubasset
    );
    assert_eq!(preflight.target().unwrap().uuid(), target_uuid);
}

#[test]
fn delete_preflight_blocks_and_projects_referencers_in_stable_registry_order() {
    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let referencer_z = entry("referencer-z", "res://data/z.json", vec![target_uuid]);
    let referencer_a = entry("referencer-a", "res://data/a.json", vec![target_uuid]);
    let registry = registry([target, referencer_z, referencer_a]);

    let preflight =
        AssetDeletePreflight::evaluate(&registry, target_uuid, AssetSourceWritePolicy::ProjectOnly);

    assert_eq!(
        preflight.disposition(),
        AssetDeleteDisposition::BlockedByReferencers
    );
    assert_eq!(
        preflight
            .referencers()
            .iter()
            .map(|referencer| referencer.locator().to_string())
            .collect::<Vec<_>>(),
        vec!["res://data/a.json", "res://data/z.json"]
    );
}

#[test]
fn optimization_batch_hq_editor598_delete_projection_consumes_runtime_topology() {
    let source = include_str!("../delete.rs");
    let projection = source
        .split("pub fn evaluate(")
        .nth(1)
        .expect("delete projection")
        .split("pub fn disposition")
        .next()
        .expect("delete projection body");

    assert!(projection.contains("topology.into_parts()"));
    assert!(!projection.contains("topology.target().cloned()"));
    assert!(!projection.contains("topology.referencers().to_vec()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hq_editor598_delete_projection_ownership_p95() {
    const MARKER: &str = "EDITOR598_ASSET_DELETE_PREFLIGHT_OWNERSHIP_BENCH_V1";
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS_PER_SAMPLE: usize = 256;
    const REFERENCER_COUNT: usize = 128;

    let target = entry("target", "res://data/target.json", Vec::new());
    let target_uuid = target.uuid();
    let mut entries = Vec::with_capacity(REFERENCER_COUNT + 1);
    entries.push(target);
    for index in 0..REFERENCER_COUNT {
        let label = format!("referencer-{index:04}");
        let path = format!(
            "res://data/{index:04}-{}-referencer.json",
            "ownership-payload".repeat(8)
        );
        entries.push(entry(&label, &path, vec![target_uuid]));
    }
    let registry = registry(entries);
    let topology = AssetMutationDeletePreflight::evaluate(&registry, target_uuid);
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure_delete_projection(
                &topology,
                ITERATIONS_PER_SAMPLE,
                false,
            ));
            optimized.push(measure_delete_projection(
                &topology,
                ITERATIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized.push(measure_delete_projection(
                &topology,
                ITERATIONS_PER_SAMPLE,
                true,
            ));
            legacy.push(measure_delete_projection(
                &topology,
                ITERATIONS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} referencers={REFERENCER_COUNT} iterations={ITERATIONS_PER_SAMPLE}"
    );
    assert!(
        ratio <= 0.25,
        "{MARKER} expected ownership projection ratio <= 0.25, got {ratio:.4}"
    );
}

fn measure_delete_projection(
    topology: &AssetMutationDeletePreflight,
    iterations: usize,
    optimized: bool,
) -> Duration {
    let mut elapsed = Duration::ZERO;
    for _ in 0..iterations {
        let topology = topology.clone();
        let start = Instant::now();
        let projected = if optimized {
            topology.into_parts()
        } else {
            (
                topology.disposition(),
                topology.target().cloned(),
                topology.referencers().to_vec(),
            )
        };
        black_box(projected);
        elapsed += start.elapsed();
    }
    elapsed
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
