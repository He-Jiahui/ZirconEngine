use std::collections::BTreeSet;
use std::convert::Infallible;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

struct NoopVisitor;

const DIRECTORY_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_DIRECTORY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn canonical_directories() -> Vec<PathBuf> {
    (0..DIRECTORY_ADMISSION_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "generated/plugins/with/a/long/shared/prefix/package_{:05}",
                (index * 4_099) % UNIQUE_DIRECTORY_COUNT
            ))
        })
        .collect()
}

fn ordered_unique_count(paths: &[PathBuf]) -> usize {
    let mut visited = BTreeSet::new();
    let mut admitted = 0;
    for path in paths {
        if !visited.contains(path) {
            visited.insert(path.clone());
            admitted += 1;
        }
    }
    admitted
}

fn hash_unique_count(paths: &[PathBuf]) -> usize {
    let mut visited = HashSet::new();
    let mut admitted = 0;
    for path in paths {
        if !visited.contains(path) {
            visited.insert(path.clone());
            admitted += 1;
        }
    }
    admitted
}

impl NativePluginManifestTraversalVisitor for NoopVisitor {
    type Error = Infallible;

    fn checkpoint(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reserve_scratch(&mut self, _total_bytes: u64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn manifest(&mut self, _manifest_path: PathBuf) -> Result<(), Self::Error> {
        Ok(())
    }

    fn diagnostic(
        &mut self,
        _build: impl FnOnce() -> NativePluginManifestTraversalDiagnostic,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn manifest_traversal_reports_enumerate_root_with_typed_error() {
    let missing_root = std::env::temp_dir().join(format!(
        "zircon-missing-native-plugin-root-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    let error = match traverse_plugin_manifests(&missing_root, &mut NoopVisitor) {
        Err(NativePluginManifestTraversalError::Collection(error)) => error,
        Err(NativePluginManifestTraversalError::Visitor(never)) => match never {},
        Ok(_) => panic!("missing root should report typed manifest collection error"),
    };

    match error {
        NativePluginManifestCollectionError::EnumerateRoot { root, .. } => {
            assert_eq!(root, missing_root);
        }
        NativePluginManifestCollectionError::InspectEntry { .. } => {
            panic!("missing root should fail while enumerating root")
        }
    }
}

#[test]
fn manifest_collection_typed_error_preserves_inspect_entry_message() {
    let root = PathBuf::from("native-plugin-root");
    let error = NativePluginManifestCollectionError::InspectEntry {
        root: root.clone(),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "entry unavailable"),
    };

    assert_eq!(
        error.to_string(),
        "failed to inspect native plugin entry under native-plugin-root: entry unavailable"
    );
    assert!(
        std::error::Error::source(&error).is_some(),
        "inspect-entry error should preserve io::Error source"
    );
}

#[test]
fn optimization_batch_20260826ac_runtime07_hash_traversal_visits_each_directory_once() {
    let fixture = TraversalFixture::new();
    fs::create_dir_all(fixture.root.join("alpha/nested")).unwrap();
    fs::create_dir_all(fixture.root.join("beta")).unwrap();

    let traversal = match traverse_plugin_manifests(&fixture.root, &mut NoopVisitor) {
        Ok(traversal) => traversal,
        Err(NativePluginManifestTraversalError::Collection(error)) => {
            panic!("fixture traversal failed: {error}")
        }
        Err(NativePluginManifestTraversalError::Visitor(never)) => match never {},
    };

    assert_eq!(traversal.enumerated_directories, 4);
    assert_eq!(traversal.inspected_entries, 3);
}

#[test]
fn optimization_batch_20260826ac_runtime07_native_plugin_traversal_uses_hash_membership() {
    let source = include_str!("../collect_manifests.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{HashSet, VecDeque};"));
    assert!(production.contains("HashSet::from([canonical_root.clone()])"));
    assert!(production.contains("visited.contains(&canonical_child)"));
    assert!(production.contains("visited.insert(canonical_child.clone())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
// 比较相同目录集合的成员去重成本；它不测磁盘 I/O，也不证明真实发现端到端延迟。
fn optimization_batch_20260826ac_runtime07_native_plugin_traversal_hash_membership_performance_evidence(
) {
    let directories = canonical_directories();
    assert_eq!(
        ordered_unique_count(&directories),
        hash_unique_count(&directories)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&directories)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&directories)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&directories)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&directories)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME07_NATIVE_PLUGIN_TRAVERSAL_HASH_MEMBERSHIP_BENCH_V1 \
             admissions={DIRECTORY_ADMISSION_COUNT} unique_directories={UNIQUE_DIRECTORY_COUNT} \
             clone_only_on_first_visit=true ordered_p95_ns={} hash_p95_ns={}",
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

struct TraversalFixture {
    root: PathBuf,
}

impl TraversalFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-native-plugin-traversal-hash-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

impl Drop for TraversalFixture {
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
