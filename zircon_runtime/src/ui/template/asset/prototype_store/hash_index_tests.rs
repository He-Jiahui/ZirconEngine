use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::UiPrototypeStore;
use crate::ui::template::UiAssetLoader;
use zircon_runtime_interface::ui::template::UiRawAssetPrototype;

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 1_024;

const MINIMAL_PROTOTYPE: &str = r#"
[asset]
kind = "layout"
id = "asset://ui/tests/minimal.zui"
version = 3

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Panel"
"#;

fn prototype(asset_id: &str) -> UiRawAssetPrototype {
    let mut prototype = UiAssetLoader::load_flat_prototype_toml_str(MINIMAL_PROTOTYPE).unwrap();
    prototype.asset.id = asset_id.to_string();
    prototype
}

#[test]
fn optimization_batch_20260826bs_ui_prototype_hash_index_preserves_canonical_and_alias_lookup() {
    let mut store = UiPrototypeStore::new();
    let canonical = store.insert(prototype("asset://ui/canonical.zui"));
    store.insert_alias("res://ui/canonical.zui", Arc::clone(&canonical));

    let _: &HashMap<String, Arc<UiRawAssetPrototype>> = &store.assets;
    assert!(Arc::ptr_eq(
        store.get("asset://ui/canonical.zui").as_ref().unwrap(),
        &canonical
    ));
    assert!(Arc::ptr_eq(
        store.get("res://ui/canonical.zui").as_ref().unwrap(),
        &canonical
    ));
    assert_eq!(store.len(), 2);
}

#[test]
fn optimization_batch_20260826bs_ui_prototype_hash_index_preserves_alias_replacement() {
    let mut store = UiPrototypeStore::new();
    let first = store.insert(prototype("asset://ui/first.zui"));
    let replacement = store.insert(prototype("asset://ui/replacement.zui"));
    store.insert_alias("res://ui/shared.zui", first);
    store.insert_alias("res://ui/shared.zui", Arc::clone(&replacement));

    assert!(Arc::ptr_eq(
        store.get("res://ui/shared.zui").as_ref().unwrap(),
        &replacement
    ));
    assert_eq!(store.len(), 3);
}

fn run_ordered_workload(
    assets: &BTreeMap<String, Arc<UiRawAssetPrototype>>,
    asset_id: &str,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(assets.get(asset_id).map(Arc::clone));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(store: &UiPrototypeStore, asset_id: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(store.get(asset_id));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn optimization_batch_20260826bs_ui_prototype_hash_index_p95() {
    let prefix = "ui-prototype-shared-prefix/".repeat(20);
    let mut store = UiPrototypeStore::new();
    let mut ordered = BTreeMap::new();
    for index in 0..ENTRY_COUNT {
        let asset_id = format!("asset://{prefix}{index:04}.zui");
        let prototype = store.insert(prototype(&asset_id));
        ordered.insert(asset_id, prototype);
    }
    let target = format!("asset://{prefix}{:04}.zui", ENTRY_COUNT - 1);
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(&ordered, &target));
            hash_samples.push(run_hash_workload(&store, &target));
        } else {
            hash_samples.push(run_hash_workload(&store, &target));
            ordered_samples.push(run_ordered_workload(&ordered, &target));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "RUNTIME74_UI_PROTOTYPE_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95}"
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
