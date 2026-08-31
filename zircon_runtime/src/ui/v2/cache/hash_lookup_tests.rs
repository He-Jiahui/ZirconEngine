use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetHeader, UiV2AssetKind, UiV2Root, UI_V2_ASSET_SCHEMA_VERSION,
};

use super::UiV2PrototypeStore;

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 1_024;

fn document(asset_id: &str) -> UiV2AssetDocument {
    UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: UiV2AssetKind::View,
            id: asset_id.to_string(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: String::new(),
        },
        root: Some(UiV2Root {
            node: "root".to_string(),
        }),
        imports: Default::default(),
        tokens: BTreeMap::new(),
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

#[test]
fn optimization_batch_20260826bq_ui_v2_prototype_hash_index_preserves_order_and_aliases() {
    let mut store = UiV2PrototypeStore::new();
    let beta = store.insert(document("asset/beta"));
    let alpha = store.insert(document("asset/alpha"));
    store.insert_alias("alias/alpha", Arc::clone(&alpha));

    assert!(Arc::ptr_eq(
        store.get("asset/beta").as_ref().unwrap(),
        &beta
    ));
    assert!(Arc::ptr_eq(
        store.get("alias/alpha").as_ref().unwrap(),
        &alpha
    ));
    assert_eq!(
        store.assets.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["alias/alpha", "asset/alpha", "asset/beta"]
    );
    assert_eq!(
        store
            .documents()
            .map(|document| document.asset.id.clone())
            .collect::<Vec<_>>(),
        vec!["asset/alpha", "asset/alpha", "asset/beta"]
    );
}

#[test]
fn optimization_batch_20260826bq_ui_v2_prototype_hash_index_keeps_indexes_in_sync() {
    let mut store = UiV2PrototypeStore::new();
    let first = store.insert(document("asset/first"));
    let replacement = store.insert(document("asset/replacement"));
    store.insert_alias("alias/shared", Arc::clone(&first));
    store.insert_alias("alias/shared", Arc::clone(&replacement));

    assert_eq!(store.assets.len(), store.asset_lookup.len());
    for (asset_id, ordered) in &store.assets {
        let indexed = store.asset_lookup.get(asset_id).unwrap();
        assert!(Arc::ptr_eq(ordered, indexed));
    }
    assert!(Arc::ptr_eq(
        store.get("alias/shared").as_ref().unwrap(),
        &replacement
    ));
}

fn run_ordered_workload(store: &UiV2PrototypeStore, asset_id: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(store.assets.get(asset_id).map(Arc::clone));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(store: &UiV2PrototypeStore, asset_id: &str) -> u128 {
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
fn optimization_batch_20260826bq_ui_v2_prototype_hash_index_p95() {
    let prefix = "ui-v2-prototype-shared-prefix/".repeat(20);
    let mut store = UiV2PrototypeStore::new();
    for index in 0..ENTRY_COUNT {
        store.insert(document(&format!("{prefix}{index:04}")));
    }
    let target = format!("{prefix}{:04}", ENTRY_COUNT - 1);
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(&store, &target));
            hash_samples.push(run_hash_workload(&store, &target));
        } else {
            hash_samples.push(run_hash_workload(&store, &target));
            ordered_samples.push(run_ordered_workload(&store, &target));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "RUNTIME74_UI_V2_PROTOTYPE_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95}"
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "hash lookup P95 must be at least 30% below ordered lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
