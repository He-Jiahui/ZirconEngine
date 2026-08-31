use std::collections::BTreeSet;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;

const INVENTORY_PATH_COUNT: usize = 8_192;
const MEMBERSHIP_LOOKUP_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn inventory_paths() -> Vec<PathBuf> {
    (0..INVENTORY_PATH_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "generated/export/inventory/with/a/long/shared/prefix/artifact_{index:05}.bin"
            ))
        })
        .collect()
}

fn membership_lookups(paths: &[PathBuf]) -> Vec<PathBuf> {
    (0..MEMBERSHIP_LOOKUP_COUNT)
        .map(|index| paths[(index * 4_099) % paths.len()].clone())
        .collect()
}

fn ordered_match_count(paths: &[PathBuf], lookups: &[PathBuf]) -> usize {
    let values = paths.iter().cloned().collect::<BTreeSet<_>>();
    lookups
        .iter()
        .filter(|path| values.contains(path.as_path()))
        .count()
}

fn hash_match_count(paths: &[PathBuf], lookups: &[PathBuf]) -> usize {
    let values = paths.iter().cloned().collect::<HashSet<_>>();
    lookups
        .iter()
        .filter(|path| values.contains(path.as_path()))
        .count()
}

#[test]
fn optimization_batch_20260826aa_editor15_inventory_hash_membership_preserves_digest_cache() {
    let fixture = InventoryFixture::new();
    let artifact = fixture.root.join("artifact.bin");
    std::fs::write(&artifact, b"stable export artifact").unwrap();

    let mut inventory = ExportGenerationInventory::default();
    let first = inventory.digest_path(&artifact).unwrap();
    let second = inventory.digest_path(&artifact).unwrap();

    assert_eq!(first, second);
    assert_eq!(inventory.seen_file_paths.len(), 1);
    assert_eq!(inventory.file_reads, 1);
}

#[test]
fn optimization_batch_20260826aa_editor15_inventory_uses_hash_membership_without_reordering_output()
{
    let source = include_str!("../inventory.rs");

    assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(source.contains("visiting_directories: HashSet<PathBuf>"));
    assert!(source.contains("seen_file_paths: HashSet<PathBuf>"));
    assert!(source.contains("children.sort();"));
    assert!(!source.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826aa_editor15_export_inventory_hash_membership_performance_evidence() {
    let paths = inventory_paths();
    let lookups = membership_lookups(&paths);
    assert_eq!(
        ordered_match_count(&paths, &lookups),
        hash_match_count(&paths, &lookups)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_match_count(black_box(&paths), black_box(&lookups)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_match_count(black_box(&paths), black_box(&lookups)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_match_count(black_box(&paths), black_box(&lookups)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_match_count(black_box(&paths), black_box(&lookups)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR15_EXPORT_INVENTORY_HASH_MEMBERSHIP_BENCH_V1 \
         inventory_paths={INVENTORY_PATH_COUNT} lookups={MEMBERSHIP_LOOKUP_COUNT} \
         ordered_lookup_class=log_n hash_lookup_class=average_constant \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

struct InventoryFixture {
    root: PathBuf,
}

impl InventoryFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-export-hash-membership-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
