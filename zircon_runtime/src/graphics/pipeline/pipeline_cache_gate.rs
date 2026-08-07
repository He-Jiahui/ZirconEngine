use std::fs::File;
use std::io::Read;
use std::mem::size_of;
use std::path::{Path, PathBuf};

use crate::core::resource::io::atomic_file::atomic_write;
use crate::graphics::shader::ShaderVariantCacheDisk;

const PIPELINE_CACHE_MAGIC: &[u8; 8] = b"ZRPSOC01";
const PIPELINE_CACHE_DIGEST_BYTES: usize = 32;
const PIPELINE_CACHE_HEADER_BYTES: usize =
    PIPELINE_CACHE_MAGIC.len() + size_of::<u64>() + PIPELINE_CACHE_DIGEST_BYTES;
const PIPELINE_CACHE_SCHEMA_DIR: &str = "pipeline-cache-v1";
const MAX_PIPELINE_CACHE_SEED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineCacheGate {
    Enabled,
    UnsupportedBackend,
    UnsupportedDeviceFeature,
    MissingSeedData,
}

pub(crate) const fn pipeline_cache_gate(
    backend: wgpu::Backend,
    has_seed_data: bool,
    pipeline_cache_feature_enabled: bool,
) -> PipelineCacheGate {
    if !matches!(backend, wgpu::Backend::Vulkan) {
        return PipelineCacheGate::UnsupportedBackend;
    }
    if !pipeline_cache_feature_enabled {
        return PipelineCacheGate::UnsupportedDeviceFeature;
    }
    if !has_seed_data {
        return PipelineCacheGate::MissingSeedData;
    }
    PipelineCacheGate::Enabled
}

/// Owns the Vulkan-only WGPU pipeline cache and persists driver data using an
/// adapter-specific key. A missing seed creates a cold cache for the next run.
pub(crate) struct RuntimePipelineCache {
    cache: Option<wgpu::PipelineCache>,
    path: Option<PathBuf>,
}

impl RuntimePipelineCache {
    pub(crate) const fn disabled() -> Self {
        Self {
            cache: None,
            path: None,
        }
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        adapter_info: &wgpu::AdapterInfo,
        project_root: &Path,
    ) -> Self {
        let Some(cache_key) = wgpu::util::pipeline_cache_key(adapter_info) else {
            return Self::disabled();
        };
        let path = ShaderVariantCacheDisk::default_project_root(project_root)
            .join(PIPELINE_CACHE_SCHEMA_DIR)
            .join(format!("{cache_key}.bin"));
        let seed = read_pipeline_cache_seed(&path);
        let gate = pipeline_cache_gate(
            adapter_info.backend,
            seed.is_some(),
            device.features().contains(wgpu::Features::PIPELINE_CACHE),
        );
        if matches!(
            gate,
            PipelineCacheGate::UnsupportedBackend | PipelineCacheGate::UnsupportedDeviceFeature
        ) {
            return Self::disabled();
        }

        // SAFETY: decoded seed bytes only come from PipelineCache::get_data,
        // protected by a versioned envelope and content digest. `fallback`
        // lets WGPU discard driver-invalidated data after driver updates.
        let cache = unsafe {
            device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                label: Some("zircon-runtime-pipeline-cache"),
                data: seed.as_deref(),
                fallback: true,
            })
        };
        Self {
            cache: Some(cache),
            path: Some(path),
        }
    }

    pub(crate) const fn cache(&self) -> Option<&wgpu::PipelineCache> {
        self.cache.as_ref()
    }

    fn persist(&self) -> std::io::Result<()> {
        let (Some(cache), Some(path)) = (&self.cache, &self.path) else {
            return Ok(());
        };
        let Some(data) = cache.get_data() else {
            return Ok(());
        };
        if data.len() > MAX_PIPELINE_CACHE_SEED_BYTES {
            return Ok(());
        }
        atomic_write(path, &encode_pipeline_cache_seed(&data))
    }
}

impl Drop for RuntimePipelineCache {
    fn drop(&mut self) {
        let _ = self.persist();
    }
}

fn encode_pipeline_cache_seed(data: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(PIPELINE_CACHE_HEADER_BYTES + data.len());
    encoded.extend_from_slice(PIPELINE_CACHE_MAGIC);
    encoded.extend_from_slice(&(data.len() as u64).to_le_bytes());
    encoded.extend_from_slice(blake3::hash(data).as_bytes());
    encoded.extend_from_slice(data);
    encoded
}

fn read_pipeline_cache_seed(path: &Path) -> Option<Vec<u8>> {
    read_pipeline_cache_seed_with_limit(path, MAX_PIPELINE_CACHE_SEED_BYTES)
}

fn read_pipeline_cache_seed_with_limit(path: &Path, max_seed_bytes: usize) -> Option<Vec<u8>> {
    let max_encoded_bytes = PIPELINE_CACHE_HEADER_BYTES.checked_add(max_seed_bytes)?;
    let read_limit = max_encoded_bytes.checked_add(1)?;
    let file = File::open(path).ok()?;
    let mut encoded = Vec::with_capacity(read_limit.min(1024 * 1024));
    file.take(read_limit as u64)
        .read_to_end(&mut encoded)
        .ok()?;
    if encoded.len() > max_encoded_bytes {
        return None;
    }
    decode_pipeline_cache_seed_owned(encoded)
}

fn decode_pipeline_cache_seed_owned(mut encoded: Vec<u8>) -> Option<Vec<u8>> {
    pipeline_cache_seed_data(&encoded)?;
    encoded.drain(..PIPELINE_CACHE_HEADER_BYTES);
    Some(encoded)
}

fn decode_pipeline_cache_seed(encoded: &[u8]) -> Option<Vec<u8>> {
    pipeline_cache_seed_data(encoded).map(<[u8]>::to_vec)
}

fn pipeline_cache_seed_data(encoded: &[u8]) -> Option<&[u8]> {
    if encoded.len() < PIPELINE_CACHE_HEADER_BYTES
        || encoded.get(..PIPELINE_CACHE_MAGIC.len())? != PIPELINE_CACHE_MAGIC
    {
        return None;
    }
    let length_offset = PIPELINE_CACHE_MAGIC.len();
    let digest_offset = length_offset + size_of::<u64>();
    let data_offset = digest_offset + PIPELINE_CACHE_DIGEST_BYTES;
    let declared_length =
        u64::from_le_bytes(encoded.get(length_offset..digest_offset)?.try_into().ok()?);
    let data = encoded.get(data_offset..)?;
    if declared_length != data.len() as u64 {
        return None;
    }
    let expected_digest = encoded.get(digest_offset..data_offset)?;
    if blake3::hash(data).as_bytes() != expected_digest {
        return None;
    }
    Some(data)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        decode_pipeline_cache_seed, encode_pipeline_cache_seed, pipeline_cache_gate,
        read_pipeline_cache_seed_with_limit, PipelineCacheGate,
    };

    #[test]
    fn render_perf_pipeline_cache_gate_is_vulkan_only_and_reports_cold_seed() {
        assert_eq!(
            pipeline_cache_gate(wgpu::Backend::Vulkan, true, true),
            PipelineCacheGate::Enabled
        );
        assert_eq!(
            pipeline_cache_gate(wgpu::Backend::Vulkan, false, true),
            PipelineCacheGate::MissingSeedData
        );
        assert_eq!(
            pipeline_cache_gate(wgpu::Backend::Dx12, true, true),
            PipelineCacheGate::UnsupportedBackend
        );
        assert_eq!(
            pipeline_cache_gate(wgpu::Backend::Vulkan, true, false),
            PipelineCacheGate::UnsupportedDeviceFeature
        );
    }

    #[test]
    fn render_perf_pipeline_cache_seed_rejects_corruption_and_truncation() {
        let mut encoded = encode_pipeline_cache_seed(b"driver-cache-data");
        assert_eq!(
            decode_pipeline_cache_seed(&encoded),
            Some(b"driver-cache-data".to_vec())
        );

        let last = encoded.len() - 1;
        encoded[last] ^= 0x5a;
        assert_eq!(decode_pipeline_cache_seed(&encoded), None);
        assert_eq!(decode_pipeline_cache_seed(&encoded[..10]), None);
    }

    #[test]
    fn render_perf_pipeline_cache_seed_read_is_bounded_without_second_decode_copy() {
        let root = std::env::temp_dir().join(format!(
            "zircon-pipeline-cache-read-bound-{}",
            std::process::id()
        ));
        let path = root.join("seed.bin");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, encode_pipeline_cache_seed(b"12345")).unwrap();

        assert_eq!(
            read_pipeline_cache_seed_with_limit(&path, 5),
            Some(b"12345".to_vec())
        );
        assert_eq!(read_pipeline_cache_seed_with_limit(&path, 4), None);

        let _ = fs::remove_dir_all(root);
    }
}
