use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::render_graph::{
    RenderGraphComputePipelineFallbackPolicy, RenderGraphComputePipelineResolution,
};
use crate::rhi::{TextureDesc, TextureDimension, TextureFormat};

mod family_publication;

use super::{RenderPassDeviceEpoch, RenderPassGpuResourceFactory};
use family_publication::{ComputePipelineFamilyKey, ComputePipelineFamilyPublicationCache};

// Compute pass variants are runtime/plugin supplied, so this must remain
// bounded even when a scene continuously introduces new specializations.
const DEFAULT_COMPUTE_PIPELINE_CACHE_CAPACITY: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ComputePipelineBindingLayout {
    binding: u32,
    kind: ComputePipelineBindingKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ComputePipelineBindingKind {
    UniformBuffer,
    StorageBufferRead,
    StorageBufferReadWrite,
    SampledTexture {
        view_dimension: ComputeTextureViewDimension,
        sample_kind: ComputeTextureSampleKind,
        multisampled: bool,
    },
    StorageTextureWrite {
        view_dimension: ComputeTextureViewDimension,
        format: ComputeStorageTextureFormat,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ComputeTextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    D3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ComputeTextureSampleKind {
    Float,
    Depth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ComputeStorageTextureFormat {
    R32Float,
    Rgba8Unorm,
    Rgba16Float,
    Rgba32Float,
}

impl ComputePipelineBindingLayout {
    pub(super) const fn uniform_buffer(binding: u32) -> Self {
        Self {
            binding,
            kind: ComputePipelineBindingKind::UniformBuffer,
        }
    }

    pub(super) const fn storage_buffer_read(binding: u32) -> Self {
        Self {
            binding,
            kind: ComputePipelineBindingKind::StorageBufferRead,
        }
    }

    pub(super) const fn storage_buffer_read_write(binding: u32) -> Self {
        Self {
            binding,
            kind: ComputePipelineBindingKind::StorageBufferReadWrite,
        }
    }

    pub(super) fn sampled_texture(binding: u32, desc: &TextureDesc) -> Result<Self, String> {
        let view_dimension = compute_texture_view_dimension(desc.dimension);
        let multisampled = desc.sample_count > 1;
        if multisampled && !matches!(view_dimension, ComputeTextureViewDimension::D2) {
            return Err(format!(
                "compute multisampled texture binding `{binding}` requires a 2D texture, found {:?}",
                desc.dimension
            ));
        }
        Ok(Self {
            binding,
            kind: ComputePipelineBindingKind::SampledTexture {
                view_dimension,
                sample_kind: compute_texture_sample_kind(desc.format),
                multisampled,
            },
        })
    }

    pub(super) fn storage_texture_write(binding: u32, desc: &TextureDesc) -> Result<Self, String> {
        let view_dimension = compute_texture_view_dimension(desc.dimension);
        if matches!(view_dimension, ComputeTextureViewDimension::Cube) {
            return Err(format!(
                "compute storage texture binding `{binding}` cannot use cube texture dimension"
            ));
        }
        let Some(format) = compute_storage_texture_format(desc.format) else {
            return Err(format!(
                "compute storage texture binding `{binding}` does not support format {:?}",
                desc.format
            ));
        };
        Ok(Self {
            binding,
            kind: ComputePipelineBindingKind::StorageTextureWrite {
                view_dimension,
                format,
            },
        })
    }

    pub(super) const fn binding(&self) -> u32 {
        self.binding
    }

    fn wgpu_binding_type(&self) -> wgpu::BindingType {
        match self.kind {
            ComputePipelineBindingKind::UniformBuffer => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            ComputePipelineBindingKind::StorageBufferRead => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            ComputePipelineBindingKind::StorageBufferReadWrite => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            ComputePipelineBindingKind::SampledTexture {
                view_dimension,
                sample_kind,
                multisampled,
            } => wgpu::BindingType::Texture {
                sample_type: match sample_kind {
                    ComputeTextureSampleKind::Float => {
                        wgpu::TextureSampleType::Float { filterable: false }
                    }
                    ComputeTextureSampleKind::Depth => wgpu::TextureSampleType::Depth,
                },
                view_dimension: view_dimension.wgpu(),
                multisampled,
            },
            ComputePipelineBindingKind::StorageTextureWrite {
                view_dimension,
                format,
            } => wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: format.wgpu(),
                view_dimension: view_dimension.wgpu(),
            },
        }
    }
}

impl ComputeTextureViewDimension {
    const fn wgpu(self) -> wgpu::TextureViewDimension {
        match self {
            Self::D1 => wgpu::TextureViewDimension::D1,
            Self::D2 => wgpu::TextureViewDimension::D2,
            Self::D2Array => wgpu::TextureViewDimension::D2Array,
            Self::Cube => wgpu::TextureViewDimension::Cube,
            Self::D3 => wgpu::TextureViewDimension::D3,
        }
    }
}

impl ComputeStorageTextureFormat {
    const fn wgpu(self) -> wgpu::TextureFormat {
        match self {
            Self::R32Float => wgpu::TextureFormat::R32Float,
            Self::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            Self::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            Self::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

fn compute_texture_view_dimension(dimension: TextureDimension) -> ComputeTextureViewDimension {
    match dimension {
        TextureDimension::D1 => ComputeTextureViewDimension::D1,
        TextureDimension::D2 => ComputeTextureViewDimension::D2,
        TextureDimension::D2Array => ComputeTextureViewDimension::D2Array,
        TextureDimension::Cube => ComputeTextureViewDimension::Cube,
        TextureDimension::D3 => ComputeTextureViewDimension::D3,
    }
}

fn compute_texture_sample_kind(format: TextureFormat) -> ComputeTextureSampleKind {
    if matches!(
        format,
        TextureFormat::Depth24Plus
            | TextureFormat::Depth24PlusStencil8
            | TextureFormat::Depth32Float
    ) {
        ComputeTextureSampleKind::Depth
    } else {
        ComputeTextureSampleKind::Float
    }
}

fn compute_storage_texture_format(format: TextureFormat) -> Option<ComputeStorageTextureFormat> {
    match format {
        TextureFormat::R32Float => Some(ComputeStorageTextureFormat::R32Float),
        TextureFormat::Rgba8Unorm => Some(ComputeStorageTextureFormat::Rgba8Unorm),
        TextureFormat::Rgba16Float => Some(ComputeStorageTextureFormat::Rgba16Float),
        TextureFormat::Rgba32Float => Some(ComputeStorageTextureFormat::Rgba32Float),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComputePipelineCacheKey {
    shader_source: String,
    entry_point: String,
    bindings: Vec<ComputePipelineBindingLayout>,
}

impl ComputePipelineCacheKey {
    fn new(source: &str, entry_point: &str, bindings: &[ComputePipelineBindingLayout]) -> Self {
        Self {
            shader_source: source.to_string(),
            entry_point: entry_point.to_string(),
            bindings: bindings.to_vec(),
        }
    }

    fn matches(
        &self,
        source: &str,
        entry_point: &str,
        bindings: &[ComputePipelineBindingLayout],
    ) -> bool {
        self.shader_source == source && self.entry_point == entry_point && self.bindings == bindings
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ComputePipelineCacheBucketKey {
    shader_source_hash: u64,
    entry_point_hash: u64,
    bindings_hash: u64,
}

impl ComputePipelineCacheBucketKey {
    fn new(source: &str, entry_point: &str, bindings: &[ComputePipelineBindingLayout]) -> Self {
        Self {
            shader_source_hash: cache_hash(source),
            entry_point_hash: cache_hash(entry_point),
            bindings_hash: cache_hash(bindings),
        }
    }
}

fn cache_hash<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

struct CachedComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    workgroup_size: [u32; 3],
}

pub(super) struct ResolvedComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub resolution: RenderGraphComputePipelineResolution,
}

enum ComputePipelineCacheEntry {
    Ready(CachedComputePipeline),
    Failed(String),
}

struct ComputePipelineCacheRecord {
    key: ComputePipelineCacheKey,
    entry: ComputePipelineCacheEntry,
    last_used: u64,
}

pub(super) struct ComputePipelineCache {
    active_device_epoch: Option<RenderPassDeviceEpoch>,
    scene_bind_group_layout: Option<wgpu::BindGroupLayout>,
    capacity: usize,
    use_counter: u64,
    mru_bucket_key: Option<ComputePipelineCacheBucketKey>,
    pipelines: HashMap<ComputePipelineCacheBucketKey, Vec<ComputePipelineCacheRecord>>,
    published_families: ComputePipelineFamilyPublicationCache,
}

impl ComputePipelineCache {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve(
        &mut self,
        device: &wgpu::Device,
        resource_factory: &impl RenderPassGpuResourceFactory,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        label: &str,
        source: &str,
        entry_point: &str,
        expected_workgroup_size: [u32; 3],
        bindings: &[ComputePipelineBindingLayout],
        fallback_policy: &RenderGraphComputePipelineFallbackPolicy,
        device_epoch: Option<RenderPassDeviceEpoch>,
    ) -> Result<ResolvedComputePipeline, String> {
        fallback_policy.validate()?;
        if matches!(
            fallback_policy,
            RenderGraphComputePipelineFallbackPolicy::LastGood(_)
        ) && device_epoch.is_none()
        {
            return Err(format!(
                "compute pipeline `{label}` last-good policy requires a materialized device epoch"
            ));
        }
        self.update_device_epoch(device_epoch);
        let artifact_fingerprint =
            compute_pipeline_artifact_fingerprint(source, entry_point, bindings);
        let family_key = fallback_policy.family().cloned().map(|family| {
            ComputePipelineFamilyKey::new(family, entry_point, expected_workgroup_size, bindings)
        });
        let candidate = self.get_or_create_with_factory(
            device,
            resource_factory,
            scene_bind_group_layout,
            label,
            source,
            entry_point,
            expected_workgroup_size,
            bindings,
        );
        match candidate {
            Ok((pipeline, bind_group_layout)) => {
                if let Some(family_key) = family_key {
                    let use_counter = self.next_use_counter();
                    self.published_families.publish(
                        family_key,
                        &pipeline,
                        &bind_group_layout,
                        artifact_fingerprint,
                        use_counter,
                    );
                }
                Ok(ResolvedComputePipeline {
                    pipeline,
                    bind_group_layout,
                    resolution: RenderGraphComputePipelineResolution::ready(
                        fallback_policy,
                        artifact_fingerprint,
                        device_epoch.map(RenderPassDeviceEpoch::raw_parts),
                    ),
                })
            }
            Err(candidate_failure) => {
                let Some(family_key) = family_key else {
                    return Err(candidate_failure);
                };
                let use_counter = self.next_use_counter();
                let family = family_key.family().clone();
                let Some(published) = self.published_families.resolve(&family_key, use_counter)
                else {
                    return Err(candidate_failure);
                };
                let device_epoch = device_epoch.ok_or_else(|| {
                    format!("compute pipeline `{label}` last-good resolution lost its device epoch")
                })?;
                Ok(ResolvedComputePipeline {
                    pipeline: published.pipeline,
                    bind_group_layout: published.bind_group_layout,
                    resolution: RenderGraphComputePipelineResolution::using_last_good(
                        family,
                        artifact_fingerprint,
                        published.artifact_fingerprint,
                        device_epoch.raw_parts(),
                        candidate_failure,
                    ),
                })
            }
        }
    }

    pub(super) fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        label: &str,
        source: &str,
        entry_point: &str,
        expected_workgroup_size: [u32; 3],
        bindings: &[ComputePipelineBindingLayout],
    ) -> Result<(wgpu::ComputePipeline, wgpu::BindGroupLayout), String> {
        self.get_or_create_with_factory(
            device,
            device,
            scene_bind_group_layout,
            label,
            source,
            entry_point,
            expected_workgroup_size,
            bindings,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn get_or_create_with_factory(
        &mut self,
        device: &wgpu::Device,
        resource_factory: &impl RenderPassGpuResourceFactory,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        label: &str,
        source: &str,
        entry_point: &str,
        expected_workgroup_size: [u32; 3],
        bindings: &[ComputePipelineBindingLayout],
    ) -> Result<(wgpu::ComputePipeline, wgpu::BindGroupLayout), String> {
        if entry_point.is_empty() {
            return Err(format!(
                "compute pipeline `{label}` has an empty entry point"
            ));
        }
        self.update_scene_bind_group_layout(scene_bind_group_layout);
        let use_counter = self.next_use_counter();
        if let Some(cached) = self.matching_mru_entry(source, entry_point, bindings, use_counter) {
            return cached_pipeline_result(label, entry_point, expected_workgroup_size, cached);
        }
        // The bucket avoids cloning shader text on fallback hits; matching_entry still
        // compares complete keys so equal hashes can never select a wrong pipeline.
        let bucket_key = ComputePipelineCacheBucketKey::new(source, entry_point, bindings);
        if let Some(cached) =
            self.matching_entry(&bucket_key, source, entry_point, bindings, use_counter)
        {
            return cached_pipeline_result(label, entry_point, expected_workgroup_size, cached);
        }

        let key = ComputePipelineCacheKey::new(source, entry_point, bindings);
        let entry = {
            let workgroup_size =
                match compute_entry_point_workgroup_size(label, source, entry_point) {
                    Ok(workgroup_size) => workgroup_size,
                    Err(error) => {
                        self.insert_entry(
                            bucket_key,
                            key,
                            ComputePipelineCacheEntry::Failed(error.clone()),
                            use_counter,
                        );
                        return Err(error);
                    }
                };
            if let Err(error) =
                validate_compute_workgroup_limits(label, workgroup_size, &device.limits())
            {
                self.insert_entry(
                    bucket_key,
                    key,
                    ComputePipelineCacheEntry::Failed(error.clone()),
                    use_counter,
                );
                return Err(error);
            }
            validate_expected_workgroup_size(
                label,
                entry_point,
                workgroup_size,
                expected_workgroup_size,
            )?;
            let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
            let bind_group_layout =
                resource_factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(label),
                    entries: &bindings
                        .iter()
                        .map(|binding| wgpu::BindGroupLayoutEntry {
                            binding: binding.binding(),
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: binding.wgpu_binding_type(),
                            count: None,
                        })
                        .collect::<Vec<_>>(),
                });
            let pipeline_layout =
                resource_factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: &[Some(scene_bind_group_layout), Some(&bind_group_layout)],
                    immediate_size: 0,
                });
            let shader = resource_factory.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            let pipeline =
                resource_factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some(entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });
            let validation_error = {
                crate::profile_scope!(
                    "render",
                    "shader_pipeline",
                    "compute_pipeline_error_scope_pop"
                );
                pollster::block_on(error_scope.pop()).map(|error| error.to_string())
            };
            if let Some(validation_error) = validation_error {
                let error = format!(
                    "compute pipeline `{label}` WGPU validation failed: {validation_error}"
                );
                self.insert_entry(
                    bucket_key,
                    key,
                    ComputePipelineCacheEntry::Failed(error.clone()),
                    use_counter,
                );
                return Err(error);
            }
            ComputePipelineCacheEntry::Ready(CachedComputePipeline {
                pipeline,
                bind_group_layout,
                workgroup_size,
            })
        };
        self.insert_entry(bucket_key, key, entry, use_counter);
        let cached = self
            .matching_entry(&bucket_key, source, entry_point, bindings, use_counter)
            .ok_or_else(|| {
                format!("compute pipeline `{label}` was not available after cache insertion")
            })?;
        cached_pipeline_result(label, entry_point, expected_workgroup_size, cached)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            active_device_epoch: None,
            scene_bind_group_layout: None,
            capacity: capacity.max(1),
            use_counter: 0,
            mru_bucket_key: None,
            pipelines: HashMap::new(),
            published_families: ComputePipelineFamilyPublicationCache::with_capacity(capacity),
        }
    }

    fn entry_count(&self) -> usize {
        self.pipelines.values().map(Vec::len).sum()
    }

    fn matching_entry(
        &mut self,
        bucket_key: &ComputePipelineCacheBucketKey,
        source: &str,
        entry_point: &str,
        bindings: &[ComputePipelineBindingLayout],
        use_counter: u64,
    ) -> Option<&ComputePipelineCacheEntry> {
        let candidate = self
            .pipelines
            .get_mut(bucket_key)?
            .iter_mut()
            .find(|candidate| candidate.key.matches(source, entry_point, bindings))?;
        candidate.last_used = use_counter;
        self.mru_bucket_key = Some(*bucket_key);
        Some(&candidate.entry)
    }

    fn matching_mru_entry(
        &mut self,
        source: &str,
        entry_point: &str,
        bindings: &[ComputePipelineBindingLayout],
        use_counter: u64,
    ) -> Option<&ComputePipelineCacheEntry> {
        let bucket_key = self.mru_bucket_key?;
        let candidate = self
            .pipelines
            .get_mut(&bucket_key)?
            .iter_mut()
            .find(|candidate| candidate.key.matches(source, entry_point, bindings))?;
        candidate.last_used = use_counter;
        Some(&candidate.entry)
    }

    fn insert_entry(
        &mut self,
        bucket_key: ComputePipelineCacheBucketKey,
        key: ComputePipelineCacheKey,
        entry: ComputePipelineCacheEntry,
        use_counter: u64,
    ) {
        if self.entry_count() >= self.capacity {
            self.evict_lru();
        }
        self.pipelines
            .entry(bucket_key)
            .or_default()
            .push(ComputePipelineCacheRecord {
                key,
                entry,
                last_used: use_counter,
            });
        self.mru_bucket_key = Some(bucket_key);
    }

    fn next_use_counter(&mut self) -> u64 {
        self.use_counter = self.use_counter.saturating_add(1);
        self.use_counter
    }

    fn evict_lru(&mut self) {
        let Some((bucket_key, record_index)) = self
            .pipelines
            .iter()
            .flat_map(|(bucket_key, records)| {
                records
                    .iter()
                    .enumerate()
                    .map(move |(index, record)| (*bucket_key, index, record.last_used))
            })
            .min_by_key(|(_, _, last_used)| *last_used)
            .map(|(bucket_key, record_index, _)| (bucket_key, record_index))
        else {
            return;
        };
        let remove_bucket = self.pipelines.get_mut(&bucket_key).is_some_and(|records| {
            records.remove(record_index);
            records.is_empty()
        });
        if remove_bucket {
            self.pipelines.remove(&bucket_key);
            if self.mru_bucket_key == Some(bucket_key) {
                self.mru_bucket_key = None;
            }
        }
    }

    fn update_scene_bind_group_layout(&mut self, scene_bind_group_layout: &wgpu::BindGroupLayout) {
        if let Some(cached) = self.scene_bind_group_layout.as_ref() {
            if cached == scene_bind_group_layout {
                return;
            }
            self.pipelines.clear();
            self.published_families.clear();
            self.mru_bucket_key = None;
        }
        self.scene_bind_group_layout = Some(scene_bind_group_layout.clone());
    }

    fn update_device_epoch(&mut self, device_epoch: Option<RenderPassDeviceEpoch>) {
        let Some(device_epoch) = device_epoch else {
            return;
        };
        if self.active_device_epoch == Some(device_epoch) {
            return;
        }
        self.active_device_epoch = Some(device_epoch);
        self.scene_bind_group_layout = None;
        self.pipelines.clear();
        self.published_families.clear();
        self.mru_bucket_key = None;
    }
}

fn compute_pipeline_artifact_fingerprint(
    source: &str,
    entry_point: &str,
    bindings: &[ComputePipelineBindingLayout],
) -> u64 {
    cache_hash(&(source, entry_point, bindings))
}

fn cached_pipeline_result(
    label: &str,
    entry_point: &str,
    expected_workgroup_size: [u32; 3],
    cached: &ComputePipelineCacheEntry,
) -> Result<(wgpu::ComputePipeline, wgpu::BindGroupLayout), String> {
    match cached {
        ComputePipelineCacheEntry::Ready(cached) => {
            validate_expected_workgroup_size(
                label,
                entry_point,
                cached.workgroup_size,
                expected_workgroup_size,
            )?;
            Ok((cached.pipeline.clone(), cached.bind_group_layout.clone()))
        }
        ComputePipelineCacheEntry::Failed(error) => Err(error.clone()),
    }
}

fn compute_entry_point_workgroup_size(
    label: &str,
    source: &str,
    entry_point: &str,
) -> Result<[u32; 3], String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|error| {
        format!(
            "compute pipeline `{label}` WGSL parse failed: {}",
            error.emit_to_string(source)
        )
    })?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).map_err(|error| {
        format!(
            "compute pipeline `{label}` WGSL validation failed: {}",
            error
        )
    })?;
    module
        .entry_points
        .iter()
        .find(|candidate| {
            candidate.name == entry_point && candidate.stage == naga::ShaderStage::Compute
        })
        .map(|candidate| candidate.workgroup_size)
        .ok_or_else(|| {
            format!(
                "compute pipeline `{label}` does not define a compute entry point `{entry_point}`"
            )
        })
}

fn validate_compute_workgroup_limits(
    label: &str,
    workgroup_size: [u32; 3],
    limits: &wgpu::Limits,
) -> Result<(), String> {
    if workgroup_size.iter().any(|dimension| *dimension == 0) {
        return Err(format!(
            "compute pipeline `{label}` workgroup size {workgroup_size:?} must have positive dimensions"
        ));
    }
    let dimension_limits = [
        limits.max_compute_workgroup_size_x,
        limits.max_compute_workgroup_size_y,
        limits.max_compute_workgroup_size_z,
    ];
    if workgroup_size
        .iter()
        .zip(dimension_limits)
        .any(|(actual, maximum)| *actual > maximum)
    {
        return Err(format!(
            "compute pipeline `{label}` workgroup size {workgroup_size:?} exceeds device dimension limits {dimension_limits:?}"
        ));
    }
    let invocation_count = workgroup_size
        .iter()
        .map(|dimension| u64::from(*dimension))
        .product::<u64>();
    if invocation_count > u64::from(limits.max_compute_invocations_per_workgroup) {
        return Err(format!(
            "compute pipeline `{label}` workgroup size {workgroup_size:?} has {invocation_count} invocations, exceeding the device limit {}",
            limits.max_compute_invocations_per_workgroup
        ));
    }
    Ok(())
}

fn validate_expected_workgroup_size(
    label: &str,
    entry_point: &str,
    actual_workgroup_size: [u32; 3],
    expected_workgroup_size: [u32; 3],
) -> Result<(), String> {
    if actual_workgroup_size != expected_workgroup_size {
        return Err(format!(
            "compute pipeline `{label}` entry point `{entry_point}` declares workgroup size {actual_workgroup_size:?}, but the render graph workload declares {expected_workgroup_size:?}"
        ));
    }
    Ok(())
}

impl Default for ComputePipelineCache {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_COMPUTE_PIPELINE_CACHE_CAPACITY)
    }
}

#[cfg(test)]
mod tests;
