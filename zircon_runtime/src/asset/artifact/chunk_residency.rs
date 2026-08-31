use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, hash_map::Entry};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use serde::{Deserialize, Serialize};

use crate::asset::AssetKind;

pub(super) const ARTIFACT_CHUNK_DIRECTORY: &str = "chunks";
pub(super) const ARTIFACT_CHUNK_EXTENSION: &str = "zchunk";
pub(super) const ARTIFACT_CHUNK_BYTES: usize = 64 * 1024;
const DEFAULT_ARTIFACT_CHUNK_RESIDENCY_BYTES: usize = 64 * 1024 * 1024;
// Keep lease-tracker metadata finite even if consumers never request diagnostics.
const MAX_RETIRED_EXTERNAL_LEASES: usize =
    DEFAULT_ARTIFACT_CHUNK_RESIDENCY_BYTES / ARTIFACT_CHUNK_BYTES;
// Rebuild the lazy eviction index before stale cache-hit candidates outnumber
// live entries by more than one additional candidate per resident entry.
const MAX_EVICTION_INDEX_CANDIDATES_PER_RESIDENT_ENTRY: usize = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactChunkDescriptor {
    pub(super) content_hash: Arc<str>,
    pub(super) compressed_bytes: u32,
}

impl ArtifactChunkDescriptor {
    pub(super) fn new(content_hash: Arc<str>, compressed_bytes: u32) -> Self {
        Self {
            content_hash,
            compressed_bytes,
        }
    }

    pub fn content_hash(&self) -> &str {
        self.content_hash.as_ref()
    }

    pub const fn compressed_bytes(&self) -> u32 {
        self.compressed_bytes
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactChunkInventory {
    chunk_root: Arc<PathBuf>,
    kind: AssetKind,
    revision: u64,
    content_hash: String,
    raw_bytes: u64,
    compressed_bytes: u64,
    chunks: Arc<[ArtifactChunkDescriptor]>,
}

impl ArtifactChunkInventory {
    pub(super) fn new(
        artifact_root: &Path,
        kind: AssetKind,
        revision: u64,
        content_hash: String,
        raw_bytes: u64,
        compressed_bytes: u64,
        chunks: Vec<ArtifactChunkDescriptor>,
    ) -> Self {
        Self {
            chunk_root: Arc::new(artifact_root.join(ARTIFACT_CHUNK_DIRECTORY)),
            kind,
            revision,
            content_hash,
            raw_bytes,
            compressed_bytes,
            chunks: chunks.into(),
        }
    }

    pub const fn kind(&self) -> AssetKind {
        self.kind
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    pub const fn raw_bytes(&self) -> u64 {
        self.raw_bytes
    }

    pub const fn compressed_bytes(&self) -> u64 {
        self.compressed_bytes
    }

    pub fn chunks(&self) -> &[ArtifactChunkDescriptor] {
        &self.chunks
    }

    pub fn chunk(&self, index: usize) -> Option<&ArtifactChunkDescriptor> {
        self.chunks.get(index)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactChunkResidencyDiagnostics {
    /// Chunks and bytes currently owned by the LRU cache itself.
    pub resident_chunks: usize,
    pub resident_bytes: usize,
    pub max_resident_bytes: usize,
    /// Payloads with at least one caller-owned `Arc` reference. These bytes can
    /// overlap `resident_bytes` while the cache also retains the payload.
    pub externally_leased_chunks: usize,
    pub externally_leased_bytes: usize,
    /// Unique payload allocations known to be alive through either the cache
    /// or an evicted caller-owned lease. This excludes allocator overhead and
    /// must not be interpreted as process RSS.
    pub tracked_payload_chunks: usize,
    pub tracked_payload_bytes: usize,
    /// Number of live external lease records displaced after the fixed tracker
    /// metadata budget was exhausted. Nonzero means the external and tracked
    /// payload counters are lower bounds.
    pub external_lease_tracking_overflows: u64,
    pub cache_hits: u64,
    pub disk_reads: u64,
    pub disk_read_bytes: u64,
    pub evictions: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactChunkResidencyTrimReport {
    /// Cache-owned chunks removed by the explicit trim operation.
    pub released_cache_chunks: usize,
    /// Cache-owned payload bytes removed by the explicit trim operation.
    /// This excludes caller-owned `Arc` leases and allocator page retention.
    pub released_cache_bytes: usize,
}

#[derive(Clone, Debug)]
pub(super) struct ArtifactChunkResidency {
    inner: Arc<ArtifactChunkResidencyInner>,
}

pub(super) struct ChunkReader {
    inventory: ArtifactChunkInventory,
    residency: ArtifactChunkResidency,
    next_chunk: usize,
    current: Option<OpenArtifactChunk>,
    content_hasher: blake3::Hasher,
    finished: bool,
}

impl ChunkReader {
    pub(super) fn new(
        inventory: ArtifactChunkInventory,
        residency: ArtifactChunkResidency,
    ) -> Self {
        Self {
            inventory,
            residency,
            next_chunk: 0,
            current: None,
            content_hasher: blake3::Hasher::new(),
            finished: false,
        }
    }

    fn open_next_chunk(&mut self) -> io::Result<()> {
        if self.inventory.chunk(self.next_chunk).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "artifact chunk list ended early",
            ));
        }
        let bytes = self.residency.read(&self.inventory, self.next_chunk)?;
        self.next_chunk += 1;
        self.current = Some(OpenArtifactChunk { bytes, offset: 0 });
        Ok(())
    }

    fn finish_current_chunk(&mut self) -> io::Result<()> {
        let Some(current) = self.current.take() else {
            return Ok(());
        };
        if current.offset != current.bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact chunk was not consumed to its manifest boundary",
            ));
        }
        Ok(())
    }

    fn finish_content(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        if self.content_hasher.finalize().to_hex().as_str() != self.inventory.content_hash() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact content hash does not match its manifest",
            ));
        }
        self.finished = true;
        Ok(())
    }

    pub(super) fn verify_complete(&mut self) -> io::Result<()> {
        let mut buffer = [0; ARTIFACT_CHUNK_BYTES];
        while self.read(&mut buffer)? != 0 {}
        Ok(())
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        loop {
            if self
                .current
                .as_ref()
                .is_some_and(|current| current.offset == current.bytes.len())
            {
                self.finish_current_chunk()?;
                continue;
            }

            if let Some(current) = self.current.as_mut() {
                let bytes_read = (current.bytes.len() - current.offset).min(buffer.len());
                let end = current.offset + bytes_read;
                buffer[..bytes_read].copy_from_slice(&current.bytes[current.offset..end]);
                current.offset = end;
                self.content_hasher.update(&buffer[..bytes_read]);
                return Ok(bytes_read);
            }
            if self.next_chunk == self.inventory.len() {
                self.finish_content()?;
                return Ok(0);
            }
            self.open_next_chunk()?;
        }
    }
}

struct OpenArtifactChunk {
    bytes: Arc<[u8]>,
    offset: usize,
}

impl Default for ArtifactChunkResidency {
    fn default() -> Self {
        Self::with_max_resident_bytes(DEFAULT_ARTIFACT_CHUNK_RESIDENCY_BYTES)
    }
}

impl ArtifactChunkResidency {
    pub(super) fn with_max_resident_bytes(max_resident_bytes: usize) -> Self {
        Self {
            inner: Arc::new(ArtifactChunkResidencyInner {
                max_resident_bytes,
                state: Mutex::new(ArtifactChunkResidencyState::default()),
            }),
        }
    }

    pub(super) fn read(
        &self,
        inventory: &ArtifactChunkInventory,
        index: usize,
    ) -> io::Result<Arc<[u8]>> {
        let descriptor = inventory.chunk(index).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "artifact chunk index {index} is outside inventory length {}",
                    inventory.len()
                ),
            )
        })?;
        let key = ArtifactChunkCacheKey {
            chunk_root: Arc::clone(&inventory.chunk_root),
            content_hash: Arc::clone(&descriptor.content_hash),
        };
        if let Some(bytes) = self.cached(&key, descriptor.compressed_bytes as usize)? {
            return Ok(bytes);
        }

        let path = chunk_path(
            inventory.chunk_root.as_ref(),
            descriptor.content_hash.as_ref(),
        );
        let bytes = read_and_verify_chunk(&path, descriptor)?;
        self.publish(key, bytes, descriptor.compressed_bytes as usize)
    }

    pub(super) fn diagnostics(&self) -> io::Result<ArtifactChunkResidencyDiagnostics> {
        let mut state = self.lock_state()?;
        let (retired_external_chunks, retired_external_bytes) =
            state.collect_retired_external_leases();
        let (cached_external_chunks, cached_external_bytes) = state
            .entries
            .values()
            .filter(|entry| Arc::strong_count(&entry.bytes) > 1)
            .fold((0_usize, 0_usize), |(chunks, bytes), entry| {
                (
                    chunks.saturating_add(1),
                    bytes.saturating_add(entry.bytes.len()),
                )
            });
        Ok(ArtifactChunkResidencyDiagnostics {
            resident_chunks: state.entries.len(),
            resident_bytes: state.resident_bytes,
            max_resident_bytes: self.inner.max_resident_bytes,
            externally_leased_chunks: cached_external_chunks
                .saturating_add(retired_external_chunks),
            externally_leased_bytes: cached_external_bytes.saturating_add(retired_external_bytes),
            tracked_payload_chunks: state.entries.len().saturating_add(retired_external_chunks),
            tracked_payload_bytes: state.resident_bytes.saturating_add(retired_external_bytes),
            external_lease_tracking_overflows: state.external_lease_tracking_overflows,
            cache_hits: state.cache_hits,
            disk_reads: state.disk_reads,
            disk_read_bytes: state.disk_read_bytes,
            evictions: state.evictions,
        })
    }

    pub(super) fn trim(&self) -> io::Result<ArtifactChunkResidencyTrimReport> {
        let mut state = self.lock_state()?;
        let released_cache_chunks = state.entries.len();
        let released_cache_bytes = state.resident_bytes;
        let entries = std::mem::take(&mut state.entries);
        state.resident_bytes = 0;
        state.eviction_candidates = BinaryHeap::new();

        for entry in entries.into_values() {
            state.track_external_lease(&entry.bytes);
        }

        Ok(ArtifactChunkResidencyTrimReport {
            released_cache_chunks,
            released_cache_bytes,
        })
    }

    fn cached(
        &self,
        key: &ArtifactChunkCacheKey,
        expected_bytes: usize,
    ) -> io::Result<Option<Arc<[u8]>>> {
        let mut state = self.lock_state()?;
        let next_access = state.access_clock.saturating_add(1);
        let cached = {
            let Some(entry) = state.entries.get_mut(key) else {
                return Ok(None);
            };
            if entry.bytes.len() != expected_bytes {
                None
            } else {
                entry.last_access = next_access;
                Some((Arc::clone(&entry.cache_key), Arc::clone(&entry.bytes)))
            }
        };
        let Some((cache_key, bytes)) = cached else {
            if let Some(removed) = state.entries.remove(key) {
                state.resident_bytes = state.resident_bytes.saturating_sub(removed.bytes.len());
                state.track_external_lease(&removed.bytes);
            }
            return Ok(None);
        };
        state.access_clock = next_access;
        state.record_eviction_candidate(cache_key, next_access);
        state.cache_hits = state.cache_hits.saturating_add(1);
        Ok(Some(bytes))
    }

    fn publish(
        &self,
        key: ArtifactChunkCacheKey,
        bytes: Arc<[u8]>,
        expected_bytes: usize,
    ) -> io::Result<Arc<[u8]>> {
        let mut state = self.lock_state()?;
        state.disk_reads = state.disk_reads.saturating_add(1);
        state.disk_read_bytes = state
            .disk_read_bytes
            .saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));

        let access = state.next_access();
        if state
            .entries
            .get(&key)
            .is_some_and(|entry| entry.bytes.len() != expected_bytes)
        {
            if let Some(removed) = state.entries.remove(&key) {
                state.resident_bytes = state.resident_bytes.saturating_sub(removed.bytes.len());
                state.track_external_lease(&removed.bytes);
            }
        }
        if let Some((cache_key, cached_bytes)) = state.entries.get_mut(&key).map(|entry| {
            entry.last_access = access;
            (Arc::clone(&entry.cache_key), Arc::clone(&entry.bytes))
        }) {
            state.record_eviction_candidate(cache_key, access);
            return Ok(cached_bytes);
        }
        if expected_bytes > self.inner.max_resident_bytes {
            let caller_bytes = Arc::clone(&bytes);
            state.track_external_lease(&bytes);
            return Ok(caller_bytes);
        }
        while state.resident_bytes.saturating_add(expected_bytes) > self.inner.max_resident_bytes {
            let Some(oldest_key) = state.pop_oldest_resident_key() else {
                break;
            };
            if let Some(evicted) = state.entries.remove(&oldest_key) {
                state.resident_bytes = state.resident_bytes.saturating_sub(evicted.bytes.len());
                state.track_external_lease(&evicted.bytes);
                state.evictions = state.evictions.saturating_add(1);
            }
        }
        state.resident_bytes = state.resident_bytes.saturating_add(expected_bytes);
        let cache_key = Arc::new(key);
        state.entries.insert(
            Arc::clone(&cache_key),
            ResidentArtifactChunk {
                cache_key: Arc::clone(&cache_key),
                bytes: Arc::clone(&bytes),
                last_access: access,
            },
        );
        state.record_eviction_candidate(cache_key, access);
        Ok(bytes)
    }

    fn lock_state(&self) -> io::Result<std::sync::MutexGuard<'_, ArtifactChunkResidencyState>> {
        self.inner.state.lock().map_err(|_| {
            io::Error::other("artifact chunk residency state was poisoned by a prior panic")
        })
    }
}

#[derive(Debug)]
struct ArtifactChunkResidencyInner {
    max_resident_bytes: usize,
    state: Mutex<ArtifactChunkResidencyState>,
}

#[derive(Debug, Default)]
struct ArtifactChunkResidencyState {
    entries: HashMap<Arc<ArtifactChunkCacheKey>, ResidentArtifactChunk>,
    resident_bytes: usize,
    access_clock: u64,
    cache_hits: u64,
    disk_reads: u64,
    disk_read_bytes: u64,
    evictions: u64,
    // A bounded lazy min-index avoids scanning every resident entry for each
    // budget eviction while preserving exact least-recently-used selection.
    eviction_candidates: BinaryHeap<Reverse<(u64, Arc<ArtifactChunkCacheKey>)>>,
    eviction_index_rebuilds: u64,
    // The map keeps externally leased payloads unique without scanning all
    // live leases on every cache eviction. Its non-owning key is valid only
    // while the paired weak reference remains alive.
    retired_external_leases: HashMap<usize, RetiredArtifactChunkLease>,
    retired_external_lease_slots: Vec<Option<usize>>,
    free_retired_external_lease_slots: Vec<usize>,
    next_retired_external_lease_slot: usize,
    external_lease_tracking_overflows: u64,
}

impl ArtifactChunkResidencyState {
    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }

    fn record_eviction_candidate(&mut self, key: Arc<ArtifactChunkCacheKey>, access: u64) {
        self.eviction_candidates.push(Reverse((access, key)));
        let max_candidates = self
            .entries
            .len()
            .saturating_mul(MAX_EVICTION_INDEX_CANDIDATES_PER_RESIDENT_ENTRY);
        if self.eviction_candidates.len() > max_candidates {
            self.rebuild_eviction_index();
        }
    }

    fn rebuild_eviction_index(&mut self) {
        self.eviction_candidates = self
            .entries
            .iter()
            .map(|(key, entry)| Reverse((entry.last_access, Arc::clone(key))))
            .collect();
        self.eviction_index_rebuilds = self.eviction_index_rebuilds.saturating_add(1);
    }

    fn pop_oldest_resident_key(&mut self) -> Option<Arc<ArtifactChunkCacheKey>> {
        while let Some(Reverse((access, key))) = self.eviction_candidates.pop() {
            if self
                .entries
                .get(&key)
                .is_some_and(|entry| entry.last_access == access)
            {
                return Some(key);
            }
        }
        None
    }

    fn track_external_lease(&mut self, bytes: &Arc<[u8]>) {
        if Arc::strong_count(bytes) <= 1 {
            return;
        }
        let payload_identity = Arc::as_ptr(bytes).cast::<u8>() as usize;
        let existing_slot = match self.retired_external_leases.entry(payload_identity) {
            Entry::Occupied(occupied) if occupied.get().bytes.strong_count() > 0 => return,
            Entry::Occupied(occupied) => Some(occupied.get().slot_index),
            Entry::Vacant(_) => None,
        };
        let slot_index = existing_slot.unwrap_or_else(|| self.reserve_external_lease_slot());
        self.retired_external_lease_slots[slot_index] = Some(payload_identity);
        self.retired_external_leases.insert(
            payload_identity,
            RetiredArtifactChunkLease {
                bytes: Arc::downgrade(bytes),
                byte_size: bytes.len(),
                slot_index,
            },
        );
    }

    fn reserve_external_lease_slot(&mut self) -> usize {
        if let Some(slot_index) = self.free_retired_external_lease_slots.pop() {
            return slot_index;
        }
        if self.retired_external_lease_slots.len() < MAX_RETIRED_EXTERNAL_LEASES {
            let slot_index = self.retired_external_lease_slots.len();
            self.retired_external_lease_slots.push(None);
            return slot_index;
        }

        let slot_index = self.next_retired_external_lease_slot;
        self.next_retired_external_lease_slot = (slot_index + 1) % MAX_RETIRED_EXTERNAL_LEASES;
        if let Some(replaced_identity) = self.retired_external_lease_slots[slot_index].take() {
            if let Some(replaced) = self.retired_external_leases.remove(&replaced_identity) {
                if replaced.bytes.strong_count() > 0 {
                    self.external_lease_tracking_overflows =
                        self.external_lease_tracking_overflows.saturating_add(1);
                }
            }
        }
        slot_index
    }

    fn collect_retired_external_leases(&mut self) -> (usize, usize) {
        let mut chunks = 0_usize;
        let mut bytes = 0_usize;
        let (leases, slots, free_slots) = (
            &mut self.retired_external_leases,
            &mut self.retired_external_lease_slots,
            &mut self.free_retired_external_lease_slots,
        );
        leases.retain(|_, lease| {
            if lease.bytes.strong_count() == 0 {
                slots[lease.slot_index] = None;
                free_slots.push(lease.slot_index);
                return false;
            }
            chunks = chunks.saturating_add(1);
            bytes = bytes.saturating_add(lease.byte_size);
            true
        });
        (chunks, bytes)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ArtifactChunkCacheKey {
    chunk_root: Arc<PathBuf>,
    content_hash: Arc<str>,
}

#[derive(Debug)]
struct ResidentArtifactChunk {
    cache_key: Arc<ArtifactChunkCacheKey>,
    bytes: Arc<[u8]>,
    last_access: u64,
}

#[derive(Debug)]
struct RetiredArtifactChunkLease {
    bytes: Weak<[u8]>,
    byte_size: usize,
    slot_index: usize,
}

fn read_and_verify_chunk(
    path: &Path,
    descriptor: &ArtifactChunkDescriptor,
) -> io::Result<Arc<[u8]>> {
    let expected_bytes = u64::from(descriptor.compressed_bytes);
    let file = File::open(path)?;
    if file.metadata()?.len() != expected_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact chunk byte count does not match its manifest",
        ));
    }
    let mut bytes = Vec::with_capacity(descriptor.compressed_bytes as usize);
    file.take(expected_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() != descriptor.compressed_bytes as usize {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "artifact chunk ended before its manifest size",
        ));
    }
    if blake3::hash(&bytes).to_hex().as_str() != descriptor.content_hash.as_ref() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact chunk content hash does not match its manifest",
        ));
    }
    Ok(bytes.into())
}

pub(super) fn chunk_path(chunk_root: &Path, content_hash: &str) -> PathBuf {
    chunk_root.join(format!("{content_hash}.{ARTIFACT_CHUNK_EXTENSION}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_lease_tracker_reuses_a_slot_for_an_expired_address_record() {
        let bytes: Arc<[u8]> = Arc::from([1_u8, 2, 3]);
        let _consumer = Arc::clone(&bytes);
        let payload_identity = Arc::as_ptr(&bytes).cast::<u8>() as usize;
        let expired_lease = {
            let released: Arc<[u8]> = Arc::from([9_u8]);
            RetiredArtifactChunkLease {
                bytes: Arc::downgrade(&released),
                byte_size: released.len(),
                slot_index: 0,
            }
        };
        let mut state = ArtifactChunkResidencyState::default();
        state
            .retired_external_lease_slots
            .push(Some(payload_identity));
        state
            .retired_external_leases
            .insert(payload_identity, expired_lease);

        state.track_external_lease(&bytes);

        let tracked = state
            .retired_external_leases
            .get(&payload_identity)
            .unwrap();
        assert_eq!(tracked.slot_index, 0);
        assert_eq!(tracked.bytes.upgrade().as_deref(), Some(bytes.as_ref()));
        assert_eq!(tracked.byte_size, bytes.len());
    }

    #[test]
    fn external_lease_tracker_overwrites_one_live_record_at_its_metadata_cap() {
        let leases = (0..MAX_RETIRED_EXTERNAL_LEASES)
            .map(|_| Arc::<[u8]>::from([1_u8]))
            .collect::<Vec<_>>();
        let _consumers = leases.iter().map(Arc::clone).collect::<Vec<_>>();
        let first_identity = Arc::as_ptr(&leases[0]).cast::<u8>() as usize;
        let mut state = ArtifactChunkResidencyState::default();
        for lease in &leases {
            state.track_external_lease(lease);
        }

        let replacement: Arc<[u8]> = Arc::from([2_u8]);
        let _replacement_consumer = Arc::clone(&replacement);
        let replacement_identity = Arc::as_ptr(&replacement).cast::<u8>() as usize;
        state.track_external_lease(&replacement);

        assert_eq!(
            state.retired_external_leases.len(),
            MAX_RETIRED_EXTERNAL_LEASES
        );
        assert!(!state.retired_external_leases.contains_key(&first_identity));
        assert!(
            state
                .retired_external_leases
                .contains_key(&replacement_identity)
        );
        assert_eq!(state.external_lease_tracking_overflows, 1);
    }

    #[test]
    fn external_lease_tracker_skips_a_cache_owned_payload_without_a_consumer() {
        let bytes: Arc<[u8]> = Arc::from([1_u8, 2, 3]);
        let mut state = ArtifactChunkResidencyState::default();

        state.track_external_lease(&bytes);

        assert!(state.retired_external_leases.is_empty());
        assert!(state.retired_external_lease_slots.is_empty());

        let _consumer = Arc::clone(&bytes);
        state.track_external_lease(&bytes);

        assert_eq!(state.retired_external_leases.len(), 1);
        assert_eq!(state.retired_external_lease_slots.len(), 1);
    }

    #[test]
    fn eviction_index_skips_stale_accesses_and_selects_the_oldest_resident_key() {
        let first = Arc::new(test_cache_key("first"));
        let second = Arc::new(test_cache_key("second"));
        let mut state = ArtifactChunkResidencyState::default();
        state.entries.insert(
            Arc::clone(&first),
            test_resident_chunk(Arc::clone(&first), 1, 1),
        );
        state.entries.insert(
            Arc::clone(&second),
            test_resident_chunk(Arc::clone(&second), 2, 2),
        );
        state.record_eviction_candidate(Arc::clone(&first), 1);
        state.record_eviction_candidate(Arc::clone(&second), 2);

        state.entries.get_mut(&first).unwrap().last_access = 3;
        state.record_eviction_candidate(Arc::clone(&first), 3);

        assert_eq!(state.pop_oldest_resident_key(), Some(second));
    }

    #[test]
    fn eviction_index_reuses_the_resident_cache_key_allocation() {
        let key = Arc::new(test_cache_key("shared-key"));
        let mut state = ArtifactChunkResidencyState::default();
        state.entries.insert(
            Arc::clone(&key),
            test_resident_chunk(Arc::clone(&key), 1, 1),
        );

        state.record_eviction_candidate(Arc::clone(&key), 1);

        let Reverse((_, candidate_key)) = state
            .eviction_candidates
            .peek()
            .expect("resident entry should add one eviction candidate");
        assert!(Arc::ptr_eq(candidate_key, &key));
    }

    #[test]
    fn eviction_index_stays_bounded_during_hot_cache_hits() {
        let key = Arc::new(test_cache_key("hot"));
        let mut state = ArtifactChunkResidencyState::default();
        state.entries.insert(
            Arc::clone(&key),
            test_resident_chunk(Arc::clone(&key), 1, 0),
        );

        for access in 1..=16 {
            state.entries.get_mut(&key).unwrap().last_access = access;
            state.record_eviction_candidate(Arc::clone(&key), access);
        }

        assert!(
            state.eviction_candidates.len() <= MAX_EVICTION_INDEX_CANDIDATES_PER_RESIDENT_ENTRY
        );
        assert!(state.eviction_index_rebuilds > 0);
    }

    fn test_cache_key(content_hash: &str) -> ArtifactChunkCacheKey {
        ArtifactChunkCacheKey {
            chunk_root: Arc::new(PathBuf::from("artifact-cache-test")),
            content_hash: Arc::from(content_hash),
        }
    }

    fn test_resident_chunk(
        cache_key: Arc<ArtifactChunkCacheKey>,
        byte: u8,
        last_access: u64,
    ) -> ResidentArtifactChunk {
        ResidentArtifactChunk {
            cache_key,
            bytes: Arc::from([byte]),
            last_access,
        }
    }
}
