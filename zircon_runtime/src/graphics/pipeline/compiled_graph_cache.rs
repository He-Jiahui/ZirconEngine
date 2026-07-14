use std::collections::HashMap;
use std::sync::Arc;

use super::declarations::{
    CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions,
};
use crate::asset::{RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT};
use crate::core::framework::render::{
    CameraRenderType, RenderCapabilitySummary, RenderFrameExtract, RenderPipelineHandle,
    ShaderQualityTier,
};
use crate::core::resource::ResourceId;

const DEFAULT_COMPILED_GRAPH_CACHE_CAPACITY: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompiledGraphCacheKey {
    pub pipeline: RenderPipelineHandle,
    pub pipeline_revision: u64,
    pub shader_quality: ShaderQualityTier,
    pub frame: RenderGraphCompileFrameFingerprint,
    pub options: RenderPipelineCompileOptions,
    pub capabilities: RenderGraphCompileCapabilityFingerprint,
}

impl CompiledGraphCacheKey {
    pub fn from_inputs(
        pipeline: &RenderPipelineAsset,
        extract: &RenderFrameExtract,
        camera_target: RenderGraphCompileCameraTargetFingerprint,
        options: &RenderPipelineCompileOptions,
        capabilities: &RenderCapabilitySummary,
        shader_quality: ShaderQualityTier,
    ) -> Self {
        Self {
            pipeline: pipeline.handle,
            pipeline_revision: pipeline.revision,
            shader_quality,
            frame: extract_compile_fingerprint(extract, camera_target),
            options: options.clone(),
            capabilities: RenderGraphCompileCapabilityFingerprint::from_capabilities(capabilities),
        }
    }
}

/// Captures every `RenderFrameExtract` field that can change the compiled
/// graph shape, resource descriptors, or built-in compile-time descriptors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphCompileFrameFingerprint {
    pub core_pipeline: crate::core::framework::render::CorePipelineKind,
    pub camera_target: RenderGraphCompileCameraTargetFingerprint,
    pub camera_render_type: CameraRenderType,
    pub viewport_rect_present: bool,
    pub view_width: u32,
    pub view_height: u32,
    pub render_width: u32,
    pub render_height: u32,
    pub camera_hdr: bool,
    pub camera_msaa_samples: u32,
    pub has_particle_sprites: bool,
    pub transmission_draw_step_count: u8,
    pub requires_transmission_scene_copy: bool,
    pub requires_late_forward_opaque_pass: bool,
}

pub fn extract_compile_fingerprint(
    extract: &RenderFrameExtract,
    camera_target: RenderGraphCompileCameraTargetFingerprint,
) -> RenderGraphCompileFrameFingerprint {
    let view_size = extract.view.effective_view_size();
    let render_size = extract.view.effective_render_size();
    let camera = extract
        .view
        .selected_camera_descriptor()
        .expect("render frame extract must carry a selected camera descriptor");
    RenderGraphCompileFrameFingerprint {
        core_pipeline: extract.view.core_pipeline,
        camera_target,
        camera_render_type: camera.render_type,
        viewport_rect_present: camera.viewport_rect.is_some(),
        view_width: view_size.x,
        view_height: view_size.y,
        render_width: render_size.x,
        render_height: render_size.y,
        camera_hdr: camera.camera.hdr,
        camera_msaa_samples: camera.camera.msaa_samples,
        has_particle_sprites: !extract.particles.sprites.is_empty(),
        transmission_draw_step_count: extract
            .lighting
            .advanced_lighting
            .transmission_draw_step_count() as u8,
        requires_transmission_scene_copy: extract
            .lighting
            .advanced_lighting
            .requires_transmission_scene_copy(),
        requires_late_forward_opaque_pass: extract
            .lighting
            .advanced_lighting
            .material_features
            .requires_late_forward_opaque_pass(),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphCompileCameraTargetFingerprint {
    #[default]
    PrimarySurface,
    Texture {
        id: ResourceId,
        width: u32,
        height: u32,
        format: RenderGraphCompileTextureTargetFormat,
    },
    Headless {
        width: u32,
        height: u32,
    },
}

impl RenderGraphCompileCameraTargetFingerprint {
    #[cfg(test)]
    fn from_target_and_view_size(
        target: &crate::core::framework::render::RenderCameraTarget,
        view_size: crate::core::math::UVec2,
    ) -> Self {
        match target {
            crate::core::framework::render::RenderCameraTarget::PrimarySurface => {
                Self::PrimarySurface
            }
            crate::core::framework::render::RenderCameraTarget::Texture(handle) => Self::Texture {
                id: handle.id(),
                width: view_size.x,
                height: view_size.y,
                format: RenderGraphCompileTextureTargetFormat::Rgba8UnormSrgb,
            },
            crate::core::framework::render::RenderCameraTarget::Headless { size } => {
                Self::Headless {
                    width: size.x,
                    height: size.y,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphCompileTextureTargetFormat {
    Rgba8Unorm,
    #[default]
    Rgba8UnormSrgb,
}

impl RenderGraphCompileTextureTargetFormat {
    pub(crate) const fn as_format_label(self) -> &'static str {
        match self {
            Self::Rgba8Unorm => RGBA8_UNORM_FORMAT,
            Self::Rgba8UnormSrgb => RGBA8_UNORM_SRGB_FORMAT,
        }
    }

    pub(crate) fn from_format_label(format: &str) -> Option<Self> {
        let format = format.trim();
        if format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
            return Some(Self::Rgba8Unorm);
        }
        if format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
            return Some(Self::Rgba8UnormSrgb);
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderGraphCompileCapabilityFingerprint {
    pub supports_async_compute: bool,
    pub supports_storage_buffers: bool,
    pub supports_indirect_draw: bool,
    pub supports_buffer_readback: bool,
    pub supports_fxaa: bool,
    pub supports_smaa: bool,
    pub supports_taa: bool,
    pub supports_sparse_texture: bool,
    pub max_supported_msaa_samples: u32,
    pub virtual_geometry_supported: bool,
    pub hybrid_global_illumination_supported: bool,
}

impl RenderGraphCompileCapabilityFingerprint {
    pub const fn from_capabilities(capabilities: &RenderCapabilitySummary) -> Self {
        Self {
            supports_async_compute: capabilities.supports_async_compute,
            supports_storage_buffers: capabilities.supports_storage_buffers,
            supports_indirect_draw: capabilities.supports_indirect_draw,
            supports_buffer_readback: capabilities.supports_buffer_readback,
            supports_fxaa: capabilities.supports_fxaa,
            supports_smaa: capabilities.supports_smaa,
            supports_taa: capabilities.supports_taa,
            supports_sparse_texture: capabilities.supports_sparse_texture,
            max_supported_msaa_samples: capabilities.max_supported_msaa_samples,
            virtual_geometry_supported: capabilities.virtual_geometry_supported,
            hybrid_global_illumination_supported: capabilities.hybrid_global_illumination_supported,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledGraphCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub entries: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledGraphCacheLookupStatus {
    Hit,
    Miss,
}

impl CompiledGraphCacheLookupStatus {
    pub const fn is_hit(self) -> bool {
        matches!(self, Self::Hit)
    }
}

#[derive(Clone, Debug)]
pub struct CompiledGraphCacheLookup {
    pub pipeline: Arc<CompiledRenderPipeline>,
    pub status: CompiledGraphCacheLookupStatus,
}

#[derive(Clone, Debug)]
struct CachedCompiledGraph {
    pipeline: Arc<CompiledRenderPipeline>,
    last_used_frame: u64,
}

#[derive(Clone, Debug)]
pub struct CompiledGraphCache {
    capacity: usize,
    frame_index: u64,
    entries: HashMap<CompiledGraphCacheKey, CachedCompiledGraph>,
    stats: CompiledGraphCacheStats,
}

impl Default for CompiledGraphCache {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_COMPILED_GRAPH_CACHE_CAPACITY)
    }
}

impl CompiledGraphCache {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            frame_index: 0,
            entries: HashMap::new(),
            stats: CompiledGraphCacheStats::default(),
        }
    }

    pub fn get_or_compile_with_status(
        &mut self,
        key: CompiledGraphCacheKey,
        compile: impl FnOnce() -> Result<CompiledRenderPipeline, String>,
    ) -> Result<CompiledGraphCacheLookup, String> {
        self.frame_index = self.frame_index.saturating_add(1);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_used_frame = self.frame_index;
            let pipeline = Arc::clone(&entry.pipeline);
            self.stats.hits = self.stats.hits.saturating_add(1);
            self.stats.entries = self.entries.len();
            return Ok(CompiledGraphCacheLookup {
                pipeline,
                status: CompiledGraphCacheLookupStatus::Hit,
            });
        }

        self.stats.misses = self.stats.misses.saturating_add(1);
        let pipeline = Arc::new(compile()?);
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }
        self.entries.insert(
            key,
            CachedCompiledGraph {
                pipeline: Arc::clone(&pipeline),
                last_used_frame: self.frame_index,
            },
        );
        self.stats.entries = self.entries.len();
        Ok(CompiledGraphCacheLookup {
            pipeline,
            status: CompiledGraphCacheLookupStatus::Miss,
        })
    }

    pub fn invalidate_pipeline(&mut self, pipeline: RenderPipelineHandle) {
        self.entries.retain(|key, _| key.pipeline != pipeline);
        self.stats.entries = self.entries.len();
    }

    pub fn stats(&self) -> CompiledGraphCacheStats {
        CompiledGraphCacheStats {
            entries: self.entries.len(),
            ..self.stats
        }
    }

    fn evict_lru(&mut self) {
        let Some(key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used_frame)
            .map(|(key, _)| key.clone())
        else {
            return;
        };
        self.entries.remove(&key);
        self.stats.evictions = self.stats.evictions.saturating_add(1);
    }
}

#[cfg(test)]
mod tests;
