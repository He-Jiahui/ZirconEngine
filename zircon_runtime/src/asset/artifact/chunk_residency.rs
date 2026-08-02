use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::asset::AssetKind;

pub(super) const ARTIFACT_CHUNK_DIRECTORY: &str = "chunks";
pub(super) const ARTIFACT_CHUNK_EXTENSION: &str = "zchunk";
pub(super) const ARTIFACT_CHUNK_BYTES: usize = 64 * 1024;
const DEFAULT_ARTIFACT_CHUNK_RESIDENCY_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactChunkDescriptor {
    pub(super) content_hash: String,
    pub(super) compressed_bytes: u32,
}

impl ArtifactChunkDescriptor {
    pub(super) fn new(content_hash: String, compressed_bytes: u32) -> Self {
        Self {
            content_hash,
            compressed_bytes,
        }
    }

    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    pub const fn compressed_bytes(&self) -> u32 {
        self.compressed_bytes
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactChunkInventory {
    chunk_root: PathBuf,
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
            chunk_root: artifact_root.join(ARTIFACT_CHUNK_DIRECTORY),
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
    pub resident_chunks: usize,
    pub resident_bytes: usize,
    pub max_resident_bytes: usize,
    pub cache_hits: u64,
    pub disk_reads: u64,
    pub disk_read_bytes: u64,
    pub evictions: u64,
}

#[derive(Clone, Debug)]
pub(super) struct ArtifactChunkResidency {
    inner: Arc<ArtifactChunkResidencyInner>,
}

pub(super) struct ChunkReader {
    inventory: ArtifactChunkInventory,
    residency: ArtifactChunkResidency,
    expected_content_hash: String,
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
            expected_content_hash: inventory.content_hash().to_owned(),
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
        if self.content_hasher.finalize().to_hex().as_str() != self.expected_content_hash.as_str() {
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
            chunk_root: inventory.chunk_root.clone(),
            content_hash: descriptor.content_hash.clone(),
        };
        if let Some(bytes) = self.cached(&key, descriptor.compressed_bytes as usize)? {
            return Ok(bytes);
        }

        let path = chunk_path(&inventory.chunk_root, &descriptor.content_hash);
        let bytes = read_and_verify_chunk(&path, descriptor)?;
        self.publish(key, bytes, descriptor.compressed_bytes as usize)
    }

    pub(super) fn diagnostics(&self) -> io::Result<ArtifactChunkResidencyDiagnostics> {
        let state = self.lock_state()?;
        Ok(ArtifactChunkResidencyDiagnostics {
            resident_chunks: state.entries.len(),
            resident_bytes: state.resident_bytes,
            max_resident_bytes: self.inner.max_resident_bytes,
            cache_hits: state.cache_hits,
            disk_reads: state.disk_reads,
            disk_read_bytes: state.disk_read_bytes,
            evictions: state.evictions,
        })
    }

    fn cached(
        &self,
        key: &ArtifactChunkCacheKey,
        expected_bytes: usize,
    ) -> io::Result<Option<Arc<[u8]>>> {
        let mut state = self.lock_state()?;
        let Some(cached_bytes) = state.entries.get(key).map(|entry| entry.bytes.len()) else {
            return Ok(None);
        };
        if cached_bytes != expected_bytes {
            if let Some(removed) = state.entries.remove(key) {
                state.resident_bytes = state.resident_bytes.saturating_sub(removed.bytes.len());
            }
            return Ok(None);
        }
        let access = state.next_access();
        let Some(entry) = state.entries.get_mut(key) else {
            return Ok(None);
        };
        entry.last_access = access;
        let bytes = Arc::clone(&entry.bytes);
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
            }
        }
        if let Some(entry) = state.entries.get_mut(&key) {
            entry.last_access = access;
            return Ok(Arc::clone(&entry.bytes));
        }
        if expected_bytes > self.inner.max_resident_bytes {
            return Ok(bytes);
        }
        while state.resident_bytes.saturating_add(expected_bytes) > self.inner.max_resident_bytes {
            let Some(oldest_key) = state
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_access)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(evicted) = state.entries.remove(&oldest_key) {
                state.resident_bytes = state.resident_bytes.saturating_sub(evicted.bytes.len());
                state.evictions = state.evictions.saturating_add(1);
            }
        }
        state.resident_bytes = state.resident_bytes.saturating_add(expected_bytes);
        state.entries.insert(
            key,
            ResidentArtifactChunk {
                bytes: Arc::clone(&bytes),
                last_access: access,
            },
        );
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
    entries: HashMap<ArtifactChunkCacheKey, ResidentArtifactChunk>,
    resident_bytes: usize,
    access_clock: u64,
    cache_hits: u64,
    disk_reads: u64,
    disk_read_bytes: u64,
    evictions: u64,
}

impl ArtifactChunkResidencyState {
    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ArtifactChunkCacheKey {
    chunk_root: PathBuf,
    content_hash: String,
}

#[derive(Debug)]
struct ResidentArtifactChunk {
    bytes: Arc<[u8]>,
    last_access: u64,
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
    if blake3::hash(&bytes).to_hex().as_str() != descriptor.content_hash {
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
