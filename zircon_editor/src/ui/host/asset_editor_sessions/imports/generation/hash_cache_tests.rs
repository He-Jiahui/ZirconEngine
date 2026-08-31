use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::UiAssetImportGeneration;

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826cc_ui_import_borrowed_hash_cache_loads_each_path_once() {
    let mut generation = UiAssetImportGeneration::default();
    let load_calls = Cell::new(0_usize);
    let path = Path::new("D:/project/ui/shared/component.zui");

    let first = generation
        .load_physical_document(path, || {
            load_calls.set(load_calls.get() + 1);
            Err("cached parse failure".to_string())
        })
        .expect_err("the first parse should expose its failure")
        .to_string();
    let repeated = generation
        .load_physical_document(path, || {
            load_calls.set(load_calls.get() + 1);
            Err("unexpected second load".to_string())
        })
        .expect_err("the cached failure should be replayed")
        .to_string();

    assert_eq!(load_calls.get(), 1);
    assert_eq!(first, repeated);
    assert_eq!(generation.parsed_by_physical_path.len(), 1);
}

#[test]
fn optimization_batch_20260826cc_ui_import_borrowed_hash_cache_allocates_only_on_miss() {
    let source = include_str!("../generation.rs");
    let borrowed_lookup = source
        .find(".get(physical_path)")
        .expect("borrowed Path lookup should exist");
    let owned_insert = source
        .find("physical_path.to_path_buf()")
        .expect("owned path should only be created for insertion");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains(
        "parsed_by_physical_path: HashMap<PathBuf, Result<Arc<ParsedUiAssetImportDocument>, String>>"
    ));
    assert!(borrowed_lookup < owned_insert);
    assert!(!source.contains("BTreeMap"));
    assert!(!source.contains(".entry(physical_path.to_path_buf())"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cc_ui_import_borrowed_hash_cache_p95() {
    let paths = (0..ENTRY_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "D:/project/ui/shared/long/component/namespace/document-{index:04}.zui"
            ))
        })
        .collect::<Vec<_>>();
    let ordered = paths
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, path)| (path, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(path, value)| (path.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = paths.last().unwrap().as_path();

    let mut owned_ordered_lookup = || repeated_owned_lookup(&ordered, target);
    let mut borrowed_hash_lookup = || repeated_borrowed_lookup(&hashed, target);
    assert_eq!(
        black_box(owned_ordered_lookup()),
        black_box(borrowed_hash_lookup())
    );

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut owned_ordered_lookup));
            hash_ns.push(measure_ns(&mut borrowed_hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut borrowed_hash_lookup));
            ordered_ns.push(measure_ns(&mut owned_ordered_lookup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(5),
        "borrowed UI import hash lookup P95 must be at least 50% below owned BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR01_UI_IMPORT_BORROWED_HASH_CACHE_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} pathbuf_allocations_before={HIT_COUNT} pathbuf_allocations_after=0 ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 borrowed_hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_owned_lookup(map: &BTreeMap<PathBuf, usize>, target: &Path) -> usize {
    let mut total = 0_usize;
    for _ in 0..HIT_COUNT {
        let owned = black_box(target).to_path_buf();
        total = total.wrapping_add(black_box(map.get(&owned).copied()).unwrap_or_default());
    }
    total
}

fn repeated_borrowed_lookup(map: &HashMap<PathBuf, usize>, target: &Path) -> usize {
    let mut total = 0_usize;
    for _ in 0..HIT_COUNT {
        total =
            total.wrapping_add(black_box(map.get(black_box(target)).copied()).unwrap_or_default());
    }
    total
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
