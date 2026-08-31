use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::SystemTime;

use crate::script::{VmError, VmPluginPackage};

use super::{DiscoveredVmPluginPackage, VmPluginDiscoveryLimits};

#[derive(Debug)]
pub(crate) struct VmPluginPayloadCache {
    limits: VmPluginDiscoveryLimits,
    state: Mutex<PayloadCacheState>,
}

#[derive(Debug, Default)]
struct PayloadCacheState {
    entries: HashMap<PathBuf, Arc<CachedPayload>>,
    retained_bytes: usize,
}

#[derive(Debug)]
struct CachedPayload {
    fingerprint: PayloadFingerprint,
    bytes: OnceLock<Result<Arc<[u8]>, Arc<str>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PayloadFingerprint {
    len: u64,
    modified: Option<SystemTime>,
}

impl Default for VmPluginPayloadCache {
    fn default() -> Self {
        Self::new(VmPluginDiscoveryLimits::default())
    }
}

impl VmPluginPayloadCache {
    pub(crate) fn new(limits: VmPluginDiscoveryLimits) -> Self {
        Self {
            limits,
            state: Mutex::new(PayloadCacheState::default()),
        }
    }

    pub(crate) fn materialize(
        &self,
        discovered: &DiscoveredVmPluginPackage,
    ) -> Result<VmPluginPackage, VmError> {
        if !discovered.package.bytecode.is_empty() || discovered.package.zr_vm_project.is_some() {
            return Ok(discovered.package.clone());
        }
        let Some(bytecode_path) = discovered.source.bytecode_path.as_deref() else {
            return Ok(discovered.package.clone());
        };
        let package_root = discovered.source.package_root.as_deref().ok_or_else(|| {
            VmError::Operation(format!(
                "discovered bytecode {} has no package root",
                bytecode_path.display()
            ))
        })?;
        let canonical_path = contained_regular_file(package_root, bytecode_path, "bytecode")?;
        let bytes = self.load_canonical_path(&canonical_path)?;
        let mut package = discovered.package.clone();
        package.bytecode = bytes.as_ref().to_vec();
        Ok(package)
    }

    #[cfg(test)]
    pub(super) fn load_path(&self, path: &Path) -> Result<Arc<[u8]>, VmError> {
        let parent = path.parent().ok_or_else(|| {
            VmError::Operation(format!("bytecode path has no parent: {}", path.display()))
        })?;
        let canonical_path = contained_regular_file(parent, path, "bytecode")?;
        self.load_canonical_path(&canonical_path)
    }

    fn load_canonical_path(&self, path: &Path) -> Result<Arc<[u8]>, VmError> {
        let metadata = fs::metadata(path).map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin bytecode {}: {error}",
                path.display()
            ))
        })?;
        if metadata.len() > self.limits.max_bytecode_bytes as u64 {
            return Err(VmError::Operation(format!(
                "plugin bytecode {} exceeds bytecode byte budget {}",
                path.display(),
                self.limits.max_bytecode_bytes
            )));
        }
        let fingerprint = PayloadFingerprint {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        };
        let payload_bytes = usize::try_from(fingerprint.len).map_err(|_| {
            VmError::Operation(format!(
                "plugin bytecode size cannot fit host usize: {}",
                path.display()
            ))
        })?;
        let entry = {
            let mut state = self.state_lock();
            if let Some(current) = state.entries.get(path) {
                if current.fingerprint == fingerprint {
                    Arc::clone(current)
                } else {
                    let current_bytes = usize::try_from(current.fingerprint.len).map_err(|_| {
                        VmError::Operation(format!(
                            "cached plugin bytecode size cannot fit host usize: {}",
                            path.display()
                        ))
                    })?;
                    let retained_without_current =
                        state.retained_bytes.saturating_sub(current_bytes);
                    let next_retained = retained_without_current
                        .checked_add(payload_bytes)
                        .ok_or_else(|| {
                            VmError::Operation(
                                "plugin bytecode cache byte counter overflowed".to_string(),
                            )
                        })?;
                    self.check_retained_bytes(next_retained)?;
                    let replacement = Arc::new(CachedPayload {
                        fingerprint,
                        bytes: OnceLock::new(),
                    });
                    state
                        .entries
                        .insert(path.to_path_buf(), Arc::clone(&replacement));
                    state.retained_bytes = next_retained;
                    replacement
                }
            } else {
                if state.entries.len() >= self.limits.max_cached_bytecode_entries {
                    return Err(VmError::Operation(format!(
                        "plugin bytecode cache entry budget {} is exhausted",
                        self.limits.max_cached_bytecode_entries
                    )));
                }
                let next_retained =
                    state
                        .retained_bytes
                        .checked_add(payload_bytes)
                        .ok_or_else(|| {
                            VmError::Operation(
                                "plugin bytecode cache byte counter overflowed".to_string(),
                            )
                        })?;
                self.check_retained_bytes(next_retained)?;
                let inserted = Arc::new(CachedPayload {
                    fingerprint,
                    bytes: OnceLock::new(),
                });
                state
                    .entries
                    .insert(path.to_path_buf(), Arc::clone(&inserted));
                state.retained_bytes = next_retained;
                inserted
            }
        };
        match entry.bytes.get_or_init(|| {
            read_bounded_file_with_expected_bytes(
                path,
                self.limits.max_bytecode_bytes,
                "plugin bytecode",
                Some(payload_bytes),
            )
            .map(Arc::<[u8]>::from)
            .map_err(|error| Arc::<str>::from(error.to_string()))
        }) {
            Ok(bytes) => Ok(Arc::clone(bytes)),
            Err(error) => {
                let mut state = self.state_lock();
                let remove_failed = state
                    .entries
                    .get(path)
                    .is_some_and(|current| Arc::ptr_eq(current, &entry));
                if remove_failed {
                    state.entries.remove(path);
                    state.retained_bytes = state.retained_bytes.saturating_sub(payload_bytes);
                }
                Err(VmError::Operation(error.to_string()))
            }
        }
    }

    fn check_retained_bytes(&self, retained_bytes: usize) -> Result<(), VmError> {
        if retained_bytes > self.limits.max_cached_bytecode_bytes {
            return Err(VmError::Operation(format!(
                "plugin bytecode cache byte budget {} is exhausted",
                self.limits.max_cached_bytecode_bytes
            )));
        }
        Ok(())
    }

    fn state_lock(&self) -> MutexGuard<'_, PayloadCacheState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn read_bounded_file(
    path: &Path,
    max_bytes: usize,
    description: &str,
) -> Result<Vec<u8>, VmError> {
    read_bounded_file_with_expected_bytes(path, max_bytes, description, None)
}

fn read_bounded_file_with_expected_bytes(
    path: &Path,
    max_bytes: usize,
    description: &str,
    expected_bytes: Option<usize>,
) -> Result<Vec<u8>, VmError> {
    let file = File::open(path).map_err(|error| {
        VmError::Operation(format!(
            "failed to read {description} {}: {error}",
            path.display()
        ))
    })?;
    let read_limit = max_bytes.saturating_add(1);
    let expected_bytes = expected_bytes
        .or_else(|| {
            file.metadata()
                .ok()
                .and_then(|metadata| usize::try_from(metadata.len()).ok())
        })
        .unwrap_or(0)
        .min(read_limit);
    let bytes = read_bounded_stream(file, expected_bytes, max_bytes).map_err(|error| {
        VmError::Operation(format!(
            "failed to read {description} {}: {error}",
            path.display()
        ))
    })?;
    if bytes.len() > max_bytes {
        return Err(VmError::Operation(format!(
            "{description} {} exceeds byte budget {max_bytes}",
            path.display()
        )));
    }
    Ok(bytes)
}

fn read_bounded_stream(
    reader: impl Read,
    expected_bytes: usize,
    max_bytes: usize,
) -> io::Result<Vec<u8>> {
    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = Vec::with_capacity(expected_bytes.min(read_limit));
    reader
        .take(u64::try_from(read_limit).unwrap_or(u64::MAX))
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn contained_regular_file(
    package_root: &Path,
    path: &Path,
    description: &str,
) -> Result<PathBuf, VmError> {
    let link_metadata = fs::symlink_metadata(path).map_err(|error| {
        VmError::Operation(format!(
            "failed to inspect plugin {description} {}: {error}",
            path.display()
        ))
    })?;
    if link_metadata.file_type().is_symlink() {
        return Err(VmError::Operation(format!(
            "plugin {description} cannot be a symbolic link: {}",
            path.display()
        )));
    }
    if !link_metadata.is_file() {
        return Err(VmError::Operation(format!(
            "plugin {description} is not a regular file: {}",
            path.display()
        )));
    }
    let canonical_root = package_root.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin package root {}: {error}",
            package_root.display()
        ))
    })?;
    let canonical_path = path.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin {description} {}: {error}",
            path.display()
        ))
    })?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(VmError::Operation(format!(
            "plugin {description} escapes package root {}: {}",
            canonical_root.display(),
            canonical_path.display()
        )));
    }
    Ok(canonical_path)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        hint::black_box,
        io::{self, Read},
        path::{Path, PathBuf},
        sync::{Arc, OnceLock},
        time::{Duration, Instant},
    };

    use super::{read_bounded_stream, CachedPayload, PayloadCacheState, PayloadFingerprint};

    const PERF_SAMPLE_PAIRS: usize = 21;

    struct ChunkedReader<'a> {
        bytes: &'a [u8],
        offset: usize,
        chunk_bytes: usize,
    }

    impl<'a> ChunkedReader<'a> {
        fn new(bytes: &'a [u8], chunk_bytes: usize) -> Self {
            Self {
                bytes,
                offset: 0,
                chunk_bytes,
            }
        }
    }

    impl Read for ChunkedReader<'_> {
        fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
            if self.offset == self.bytes.len() {
                return Ok(0);
            }
            let read_bytes = destination
                .len()
                .min(self.chunk_bytes)
                .min(self.bytes.len() - self.offset);
            destination[..read_bytes]
                .copy_from_slice(&self.bytes[self.offset..self.offset + read_bytes]);
            self.offset += read_bytes;
            Ok(read_bytes)
        }
    }

    fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (percentile * sorted.len()).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn duration_csv(samples: &[Duration]) -> String {
        samples
            .iter()
            .map(Duration::as_nanos)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn vm_payload_cache_uses_hash_index_with_borrowed_paths() {
        fn assert_hash_index(_: &HashMap<PathBuf, Arc<CachedPayload>>) {}

        let path = PathBuf::from("cache/package/main.zrbc");
        let payload = Arc::new(CachedPayload {
            fingerprint: PayloadFingerprint {
                len: 128,
                modified: None,
            },
            bytes: OnceLock::new(),
        });
        let mut state = PayloadCacheState::default();
        state.entries.insert(path.clone(), Arc::clone(&payload));

        assert_hash_index(&state.entries);
        assert!(Arc::ptr_eq(
            state
                .entries
                .get(Path::new("cache/package/main.zrbc"))
                .unwrap(),
            &payload
        ));
    }

    #[test]
    fn vm_payload_bounded_reader_reserves_known_length_and_keeps_overflow_sentinel() {
        let payload = vec![0x5a; 16 * 1024];
        let bytes = read_bounded_stream(
            ChunkedReader::new(&payload, 257),
            payload.len(),
            payload.len(),
        )
        .unwrap();
        assert_eq!(bytes, payload);
        assert!(bytes.capacity() >= payload.len());

        let oversized = vec![0xa5; 1_025];
        let bytes = read_bounded_stream(ChunkedReader::new(&oversized, 31), oversized.len(), 1_024)
            .unwrap();
        assert_eq!(bytes.len(), 1_025);
    }

    #[test]
    #[ignore = "managed Runtime07 performance evidence"]
    fn vm_payload_cache_runtime07_performance_hash_lookup() {
        const ENTRIES: usize = 16_384;
        let paths = (0..ENTRIES)
            .map(|index| PathBuf::from(format!("packages/{index:05}/module/main.zrbc")))
            .collect::<Vec<_>>();
        let legacy = paths
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, path)| (path, index))
            .collect::<BTreeMap<_, _>>();
        let optimized = paths
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, path)| (path, index))
            .collect::<HashMap<_, _>>();

        let legacy_lookup = || {
            (0..ENTRIES).fold(0usize, |sum, index| {
                let query = &paths[(index * 8_191) % ENTRIES];
                sum.wrapping_add(*black_box(legacy.get(query.as_path()).unwrap()))
            })
        };
        let optimized_lookup = || {
            (0..ENTRIES).fold(0usize, |sum, index| {
                let query = &paths[(index * 8_191) % ENTRIES];
                sum.wrapping_add(*black_box(optimized.get(query.as_path()).unwrap()))
            })
        };
        assert_eq!(legacy_lookup(), optimized_lookup());
        black_box(legacy_lookup());
        black_box(optimized_lookup());

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                black_box(legacy_lookup());
                legacy_samples.push(started.elapsed());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                black_box(optimized_lookup());
                optimized_samples.push(started.elapsed());
            };
            if pair % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_csv = duration_csv(&legacy_samples);
        let optimized_csv = duration_csv(&optimized_samples);
        eprintln!(
            "RUNTIME07_PAYLOAD_HASH_LOOKUP_BENCH_V1 entries={ENTRIES} lookups_per_sample={ENTRIES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            optimized_p50.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= legacy_p95.as_nanos().saturating_mul(75),
            "hash payload lookup must reduce P95 by at least 25%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
        );
    }

    #[test]
    #[ignore = "managed Runtime07 performance evidence"]
    fn vm_payload_cache_runtime07_performance_bounded_read_capacity() {
        const PAYLOAD_BYTES: usize = 8 * 1024 * 1024;
        const CHUNK_BYTES: usize = 8 * 1024;
        let payload = vec![0x3c; PAYLOAD_BYTES];

        let legacy_read = || {
            let bytes =
                read_bounded_stream(ChunkedReader::new(&payload, CHUNK_BYTES), 0, PAYLOAD_BYTES)
                    .unwrap();
            black_box(bytes.len() + usize::from(bytes[PAYLOAD_BYTES - 1]))
        };
        let optimized_read = || {
            let bytes = read_bounded_stream(
                ChunkedReader::new(&payload, CHUNK_BYTES),
                PAYLOAD_BYTES,
                PAYLOAD_BYTES,
            )
            .unwrap();
            black_box(bytes.len() + usize::from(bytes[PAYLOAD_BYTES - 1]))
        };
        assert_eq!(legacy_read(), optimized_read());
        black_box(legacy_read());
        black_box(optimized_read());

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                black_box(legacy_read());
                legacy_samples.push(started.elapsed());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                black_box(optimized_read());
                optimized_samples.push(started.elapsed());
            };
            if pair % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_csv = duration_csv(&legacy_samples);
        let optimized_csv = duration_csv(&optimized_samples);
        eprintln!(
            "RUNTIME07_PAYLOAD_READ_CAPACITY_BENCH_V1 payload_bytes={PAYLOAD_BYTES} chunk_bytes={CHUNK_BYTES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_initial_capacity=0 optimized_initial_capacity={PAYLOAD_BYTES} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            optimized_p50.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= legacy_p95.as_nanos().saturating_mul(90),
            "known-length payload read must reduce P95 by at least 10%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
        );
    }
}
