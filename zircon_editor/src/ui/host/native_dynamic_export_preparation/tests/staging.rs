use std::hint::black_box;
use std::time::Instant;

use super::*;

fn legacy_is_native_dynamic_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "dll" | "so" | "dylib" | "pdb" | "dbg" | "dsym"
    )
}

#[test]
fn optimization_batch_et_native_artifact_extensions_preserve_case_insensitive_matching() {
    for name in [
        "plugin.dll",
        "plugin.SO",
        "plugin.DyLiB",
        "plugin.PDB",
        "plugin.dbg",
        "plugin.DSYM",
        "plugin.txt",
        "plugin",
    ] {
        let path = Path::new(name);
        assert_eq!(
            is_native_dynamic_artifact(path),
            legacy_is_native_dynamic_artifact(path),
            "extension classification diverged for {name}"
        );
    }
}

#[test]
#[ignore = "release-only allocation-free artifact extension benchmark"]
fn optimization_batch_et_native_artifact_extension_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const CHECKS_PER_SAMPLE: usize = 262_144;

    fn measure(path: &Path, classify: fn(&Path) -> bool) -> u128 {
        let started = Instant::now();
        let mut matched = 0_usize;
        for _ in 0..CHECKS_PER_SAMPLE {
            matched = matched.wrapping_add(usize::from(classify(black_box(path))));
        }
        black_box(matched);
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

    let path = Path::new("plugins/render_backend.DSYM");
    assert_eq!(
        is_native_dynamic_artifact(path),
        legacy_is_native_dynamic_artifact(path)
    );
    for _ in 0..4 {
        black_box(measure(path, legacy_is_native_dynamic_artifact));
        black_box(measure(path, is_native_dynamic_artifact));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(path, legacy_is_native_dynamic_artifact));
            optimized_samples.push(measure(path, is_native_dynamic_artifact));
        } else {
            optimized_samples.push(measure(path, is_native_dynamic_artifact));
            legacy_samples.push(measure(path, legacy_is_native_dynamic_artifact));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR382_ALLOCATION_FREE_ARTIFACT_EXTENSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             checks_per_sample={CHECKS_PER_SAMPLE} extension=DSYM \
             pair_order=alternating_legacy_even legacy_lowercase_allocations_per_check=1 \
             optimized_lowercase_allocations_per_check=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(55),
        "allocation-free artifact extension matching must reduce P95 by at least 45%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
fn unchanged_warm_staging_copies_zero_files_and_bytes() {
    let fixture = StagingFixture::new();
    fixture.write_source("plugin.toml", b"id = 'fixture'");
    fixture.write_source("assets/scene.bin", b"scene");

    let first = fixture.sync();
    let second = fixture.sync();

    assert_eq!(first.copied_files, 2);
    assert!(first.copied_bytes > 0);
    assert_eq!(second.copied_files, 0);
    assert_eq!(second.copied_bytes, 0);
    assert_eq!(second.removed_files, 0);
}

#[test]
fn changed_deleted_and_renamed_sources_update_the_staging_tree() {
    let fixture = StagingFixture::new();
    fixture.write_source("plugin.toml", b"id = 'fixture'");
    fixture.write_source("assets/changed.bin", b"before!!");
    fixture.write_source("assets/deleted.bin", b"deleted");
    fixture.write_source("assets/old-name.bin", b"renamed");
    fixture.sync();

    std::thread::sleep(std::time::Duration::from_millis(2));
    fixture.write_source("assets/changed.bin", b"after!!!");
    fs::remove_file(fixture.source.join("assets/deleted.bin")).unwrap();
    fs::rename(
        fixture.source.join("assets/old-name.bin"),
        fixture.source.join("assets/new-name.bin"),
    )
    .unwrap();
    let stats = fixture.sync();

    assert_eq!(stats.copied_files, 2);
    assert_eq!(stats.removed_files, 2);
    assert_eq!(
        fs::read(fixture.destination.join("assets/changed.bin")).unwrap(),
        b"after!!!"
    );
    assert!(!fixture.destination.join("assets/deleted.bin").exists());
    assert!(!fixture.destination.join("assets/old-name.bin").exists());
    assert_eq!(
        fs::read(fixture.destination.join("assets/new-name.bin")).unwrap(),
        b"renamed"
    );
}

struct StagingFixture {
    root: PathBuf,
    source: PathBuf,
    destination: PathBuf,
    manifest: PathBuf,
    inventory_cache: PathBuf,
}

impl StagingFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-native-staging-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = fs::remove_dir_all(&root);
        let source = root.join("source");
        let destination = root.join("destination");
        fs::create_dir_all(&source).unwrap();
        Self {
            source,
            destination,
            manifest: root.join("manifests/fixture.json"),
            inventory_cache: root.join("inventory.json"),
            root,
        }
    }

    fn write_source(&self, relative: &str, bytes: &[u8]) {
        let path = self.source.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn sync(&self) -> NativeStagingStats {
        let mut inventory =
            ExportGenerationInventory::with_persistent_cache(self.inventory_cache.clone());
        sync_native_package(
            &self.source,
            &self.destination,
            &self.manifest,
            &mut inventory,
        )
        .unwrap()
    }
}

impl Drop for StagingFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
