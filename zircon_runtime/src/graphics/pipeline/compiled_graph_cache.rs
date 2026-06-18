use std::collections::HashMap;
use std::sync::Arc;

use super::declarations::{
    CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions,
};
use crate::core::framework::render::{
    CameraRenderType, RenderCameraTarget, RenderCapabilitySummary, RenderFrameExtract,
    RenderPipelineHandle, ShaderQualityTier,
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
        options: &RenderPipelineCompileOptions,
        capabilities: &RenderCapabilitySummary,
        shader_quality: ShaderQualityTier,
    ) -> Self {
        Self {
            pipeline: pipeline.handle,
            pipeline_revision: pipeline.revision,
            shader_quality,
            frame: extract_compile_fingerprint(extract),
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
}

pub fn extract_compile_fingerprint(
    extract: &RenderFrameExtract,
) -> RenderGraphCompileFrameFingerprint {
    let view_size = extract.view.effective_view_size();
    let render_size = extract.view.effective_render_size();
    let camera = extract
        .view
        .selected_camera_descriptor()
        .expect("render frame extract must carry a selected camera descriptor");
    RenderGraphCompileFrameFingerprint {
        core_pipeline: extract.view.core_pipeline,
        camera_target: RenderGraphCompileCameraTargetFingerprint::from_target(&camera.target),
        camera_render_type: camera.render_type,
        viewport_rect_present: camera.viewport_rect.is_some(),
        view_width: view_size.x,
        view_height: view_size.y,
        render_width: render_size.x,
        render_height: render_size.y,
        camera_hdr: camera.camera.hdr,
        camera_msaa_samples: camera.camera.msaa_samples,
        has_particle_sprites: !extract.particles.sprites.is_empty(),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphCompileCameraTargetFingerprint {
    #[default]
    PrimarySurface,
    Texture {
        id: ResourceId,
    },
    Headless {
        width: u32,
        height: u32,
    },
}

impl RenderGraphCompileCameraTargetFingerprint {
    fn from_target(target: &RenderCameraTarget) -> Self {
        match target {
            RenderCameraTarget::PrimarySurface => Self::PrimarySurface,
            RenderCameraTarget::Texture(handle) => Self::Texture { id: handle.id() },
            RenderCameraTarget::Headless { size } => Self::Headless {
                width: size.x,
                height: size.y,
            },
        }
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
mod tests {
    use crate::core::framework::render::{
        CameraRenderType, CorePipelineKind, RenderCameraTarget, RenderCapabilitySummary,
        RenderDynamicResolutionSettings, RenderFrameExtract, RenderParticleSpriteSnapshot,
        RenderPipelineHandle, RenderViewportRect, RenderWorldSnapshotHandle,
    };
    use crate::core::math::{UVec2, Vec2, Vec3, Vec4};
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::pipeline::{
        CompiledGraphCache, CompiledGraphCacheKey, RenderPipelineAsset,
        RenderPipelineCompileOptions,
    };
    use crate::render_graph::RenderGraphBuilder;
    use crate::scene::world::World;

    #[test]
    fn compiled_render_pipeline_cache_hits_on_identical_key() {
        let pipeline = RenderPipelineAsset::default_forward_plus();
        let extract = test_extract();
        let key = key_for(
            &pipeline,
            &extract,
            &RenderPipelineCompileOptions::default(),
        );
        let mut cache = CompiledGraphCache::default();
        let mut compile_count = 0;

        let first = cache
            .get_or_compile_with_status(key.clone(), || {
                compile_count += 1;
                Ok(empty_compiled_pipeline(&pipeline))
            })
            .unwrap()
            .pipeline;
        let second = cache
            .get_or_compile_with_status(key, || {
                compile_count += 1;
                Ok(empty_compiled_pipeline(&pipeline))
            })
            .unwrap()
            .pipeline;

        assert!(std::sync::Arc::ptr_eq(&first, &second));
        assert_eq!(compile_count, 1);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn compiled_render_pipeline_cache_reports_lookup_status() {
        let pipeline = RenderPipelineAsset::default_forward_plus();
        let extract = test_extract();
        let key = key_for(
            &pipeline,
            &extract,
            &RenderPipelineCompileOptions::default(),
        );
        let mut cache = CompiledGraphCache::default();

        let first = cache
            .get_or_compile_with_status(key.clone(), || Ok(empty_compiled_pipeline(&pipeline)))
            .unwrap();
        let second = cache
            .get_or_compile_with_status(key, || Ok(empty_compiled_pipeline(&pipeline)))
            .unwrap();

        assert_eq!(first.status, super::CompiledGraphCacheLookupStatus::Miss);
        assert_eq!(second.status, super::CompiledGraphCacheLookupStatus::Hit);
        assert!(second.status.is_hit());
        assert!(std::sync::Arc::ptr_eq(&first.pipeline, &second.pipeline));
    }

    #[test]
    fn render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs() {
        let mut baseline = test_extract();
        baseline.apply_viewport_size(UVec2::new(128, 64));
        let baseline_fingerprint = super::extract_compile_fingerprint(&baseline);

        let mut resized = baseline.clone();
        resized.apply_viewport_size(UVec2::new(256, 64));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&resized)
        );

        let dynamic_resolution = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline)
                .with_dynamic_resolution(RenderDynamicResolutionSettings::fixed_scale(0.5)),
        );
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&dynamic_resolution)
        );

        let hdr = baseline
            .clone()
            .with_selected_camera_descriptor(selected_descriptor(&baseline).with_hdr(true));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&hdr)
        );

        let msaa = baseline
            .clone()
            .with_selected_camera_descriptor(selected_descriptor(&baseline).with_msaa_samples(4));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&msaa)
        );

        let mut particles = baseline;
        particles.particles.sprites.push(particle_sprite_snapshot());
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&particles)
        );
    }

    #[test]
    fn render_graph_compile_frame_fingerprint_tracks_camera_target_and_stack_inputs() {
        let mut baseline = test_extract();
        baseline.apply_viewport_size(UVec2::new(128, 64));
        let baseline_fingerprint = super::extract_compile_fingerprint(&baseline);

        let mut texture = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline).with_target(RenderCameraTarget::Texture(
                texture_handle("res://target-a.png"),
            )),
        );
        texture.apply_viewport_size(UVec2::new(128, 64));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&texture)
        );

        let mut other_texture = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline).with_target(RenderCameraTarget::Texture(
                texture_handle("res://target-b.png"),
            )),
        );
        other_texture.apply_viewport_size(UVec2::new(128, 64));
        assert_ne!(
            super::extract_compile_fingerprint(&texture),
            super::extract_compile_fingerprint(&other_texture)
        );

        let mut headless = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline).with_target(RenderCameraTarget::Headless {
                size: UVec2::new(128, 64),
            }),
        );
        headless.apply_viewport_size(UVec2::new(128, 64));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&headless)
        );

        let mut viewport = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline).with_viewport_rect(RenderViewportRect::new(
                UVec2::new(16, 8),
                UVec2::new(64, 32),
            )),
        );
        viewport.apply_viewport_size(UVec2::new(128, 64));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&viewport)
        );

        let mut overlay = baseline.clone().with_selected_camera_descriptor(
            selected_descriptor(&baseline).with_render_type(CameraRenderType::Overlay),
        );
        overlay.apply_viewport_size(UVec2::new(128, 64));
        assert_ne!(
            baseline_fingerprint,
            super::extract_compile_fingerprint(&overlay)
        );
    }

    #[test]
    fn compiled_render_pipeline_cache_misses_on_feature_set_change() {
        let pipeline = RenderPipelineAsset::default_forward_plus();
        let extract = test_extract();
        let mut cache = CompiledGraphCache::default();
        let first_key = key_for(
            &pipeline,
            &extract,
            &RenderPipelineCompileOptions::default(),
        );
        let second_key = key_for(
            &pipeline,
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_disabled(crate::graphics::BuiltinRenderFeature::Bloom),
        );

        cache
            .get_or_compile_with_status(first_key, || Ok(empty_compiled_pipeline(&pipeline)))
            .unwrap();
        cache
            .get_or_compile_with_status(second_key, || Ok(empty_compiled_pipeline(&pipeline)))
            .unwrap();

        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 2);
        assert_eq!(cache.stats().entries, 2);
    }

    #[test]
    fn compiled_render_pipeline_cache_misses_on_viewport_resize() {
        let pipeline = RenderPipelineAsset::default_forward_plus();
        let mut first_extract = test_extract();
        first_extract.apply_viewport_size(crate::core::math::UVec2::new(64, 64));
        let mut second_extract = test_extract();
        second_extract.apply_viewport_size(crate::core::math::UVec2::new(128, 64));
        let mut cache = CompiledGraphCache::default();

        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline,
                    &first_extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline)),
            )
            .unwrap();
        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline,
                    &second_extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline)),
            )
            .unwrap();

        assert_eq!(cache.stats().misses, 2);
        assert_eq!(cache.stats().entries, 2);
    }

    #[test]
    fn compiled_render_pipeline_cache_invalidates_on_pipeline_revision_bump() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        let extract = test_extract();
        let mut cache = CompiledGraphCache::default();

        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline,
                    &extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline)),
            )
            .unwrap();
        pipeline.bump_revision();
        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline,
                    &extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline)),
            )
            .unwrap();

        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 2);
    }

    #[test]
    fn compiled_render_pipeline_cache_evicts_least_recently_used_entry() {
        let pipeline_a = RenderPipelineAsset::default_forward_plus();
        let pipeline_b = RenderPipelineAsset::default_deferred();
        let mut pipeline_c = RenderPipelineAsset::default_core2d();
        pipeline_c.handle = RenderPipelineHandle::new(42);
        pipeline_c.core_pipeline = CorePipelineKind::Core3d;
        let extract = test_extract();
        let mut cache = CompiledGraphCache::with_capacity(2);

        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline_a,
                    &extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline_a)),
            )
            .unwrap();
        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline_b,
                    &extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline_b)),
            )
            .unwrap();
        cache
            .get_or_compile_with_status(
                key_for(
                    &pipeline_c,
                    &extract,
                    &RenderPipelineCompileOptions::default(),
                ),
                || Ok(empty_compiled_pipeline(&pipeline_c)),
            )
            .unwrap();

        assert_eq!(cache.stats().evictions, 1);
        assert_eq!(cache.stats().entries, 2);
    }

    fn key_for(
        pipeline: &RenderPipelineAsset,
        extract: &RenderFrameExtract,
        options: &RenderPipelineCompileOptions,
    ) -> CompiledGraphCacheKey {
        CompiledGraphCacheKey::from_inputs(
            pipeline,
            extract,
            options,
            &RenderCapabilitySummary::default(),
            Default::default(),
        )
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }

    fn empty_compiled_pipeline(
        pipeline: &RenderPipelineAsset,
    ) -> crate::graphics::pipeline::CompiledRenderPipeline {
        crate::graphics::pipeline::CompiledRenderPipeline {
            handle: pipeline.handle,
            name: pipeline.name.clone(),
            renderer_name: pipeline.renderer.name.clone(),
            stages: Vec::new(),
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: RenderGraphBuilder::new("cache-test").compile().unwrap(),
        }
    }

    fn particle_sprite_snapshot() -> RenderParticleSpriteSnapshot {
        RenderParticleSpriteSnapshot {
            entity: 7,
            stable_sprite_key: 11,
            position: Vec3::new(0.0, 0.0, 0.0),
            size: 1.0,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
            depth_test: true,
            material: None,
            texture: None,
        }
    }

    fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }

    fn selected_descriptor(
        extract: &RenderFrameExtract,
    ) -> crate::core::framework::render::CameraRenderDescriptor {
        extract.view.selected_camera_descriptor().unwrap().clone()
    }

    trait DescriptorTestExt {
        fn with_target(self, target: RenderCameraTarget) -> Self;
        fn with_viewport_rect(self, viewport_rect: RenderViewportRect) -> Self;
        fn with_render_type(self, render_type: CameraRenderType) -> Self;
        fn with_dynamic_resolution(
            self,
            dynamic_resolution: RenderDynamicResolutionSettings,
        ) -> Self;
        fn with_hdr(self, hdr: bool) -> Self;
        fn with_msaa_samples(self, msaa_samples: u32) -> Self;
    }

    impl DescriptorTestExt for crate::core::framework::render::CameraRenderDescriptor {
        fn with_target(mut self, target: RenderCameraTarget) -> Self {
            self.target = target;
            self
        }

        fn with_viewport_rect(mut self, viewport_rect: RenderViewportRect) -> Self {
            self.viewport_rect = Some(viewport_rect);
            self
        }

        fn with_render_type(mut self, render_type: CameraRenderType) -> Self {
            self.render_type = render_type;
            self
        }

        fn with_dynamic_resolution(
            mut self,
            dynamic_resolution: RenderDynamicResolutionSettings,
        ) -> Self {
            self.camera.dynamic_resolution = dynamic_resolution;
            self
        }

        fn with_hdr(mut self, hdr: bool) -> Self {
            self.camera.hdr = hdr;
            self
        }

        fn with_msaa_samples(mut self, msaa_samples: u32) -> Self {
            self.camera.msaa_samples = msaa_samples;
            self
        }
    }
}
