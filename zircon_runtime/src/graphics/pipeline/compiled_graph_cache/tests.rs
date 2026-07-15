use crate::core::framework::render::{
    CameraRenderType, CookieProjection, CorePipelineKind, IrradianceVolumeData, LightCookieData,
    RenderCameraTarget, RenderCapabilitySummary, RenderDynamicResolutionSettings,
    RenderFrameExtract, RenderLayerSet, RenderParticleSpriteSnapshot, RenderPipelineHandle,
    RenderViewportRect, RenderWorldSnapshotHandle,
};
use crate::core::math::{Mat4, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::pipeline::{
    CompiledGraphCache, CompiledGraphCacheKey, RenderPipelineAsset, RenderPipelineCompileOptions,
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
    let baseline_fingerprint = fingerprint_for(&baseline);

    let mut resized = baseline.clone();
    resized.apply_viewport_size(UVec2::new(256, 64));
    assert_ne!(baseline_fingerprint, fingerprint_for(&resized));

    let dynamic_resolution = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline)
            .with_dynamic_resolution(RenderDynamicResolutionSettings::fixed_scale(0.5)),
    );
    assert_ne!(baseline_fingerprint, fingerprint_for(&dynamic_resolution));

    let hdr = baseline
        .clone()
        .with_selected_camera_descriptor(selected_descriptor(&baseline).with_hdr(true));
    assert_ne!(baseline_fingerprint, fingerprint_for(&hdr));

    let msaa = baseline
        .clone()
        .with_selected_camera_descriptor(selected_descriptor(&baseline).with_msaa_samples(4));
    assert_ne!(baseline_fingerprint, fingerprint_for(&msaa));

    let mut particles = baseline.clone();
    particles.particles.sprites.push(particle_sprite_snapshot());
    assert_ne!(baseline_fingerprint, fingerprint_for(&particles));

    let mut transmission = baseline.clone();
    transmission
        .lighting
        .advanced_lighting
        .material_features
        .specular_transmission = true;
    assert_ne!(baseline_fingerprint, fingerprint_for(&transmission));

    let mut layered_transmission = transmission.clone();
    layered_transmission
        .lighting
        .advanced_lighting
        .screen_space_transmission =
        crate::core::framework::render::ScreenSpaceTransmissionSettings::new(3);
    assert_ne!(
        fingerprint_for(&transmission),
        fingerprint_for(&layered_transmission)
    );

    let mut late_forward_opaque = baseline;
    late_forward_opaque
        .lighting
        .advanced_lighting
        .material_features
        .late_forward_opaque = true;
    assert_ne!(baseline_fingerprint, fingerprint_for(&late_forward_opaque));
}

#[test]
fn render_graph_compile_frame_fingerprint_tracks_advanced_lighting_pass_presence() {
    let baseline = test_extract();
    let baseline_fingerprint = fingerprint_for(&baseline);

    let mut cookie = baseline.clone();
    cookie
        .lighting
        .advanced_lighting
        .cookies
        .push(LightCookieData {
            light_id: 7,
            texture: ResourceId::from_stable_label("runtime://cache-test/cookie"),
            projection: CookieProjection::Spot,
        });
    assert_ne!(baseline_fingerprint, fingerprint_for(&cookie));

    let mut volume = baseline;
    volume
        .lighting
        .advanced_lighting
        .irradiance_volumes
        .push(IrradianceVolumeData {
            volume_id: 9,
            transform: Mat4::IDENTITY,
            voxels: ResourceId::from_stable_label("runtime://cache-test/irradiance-volume"),
            intensity: 1.0,
            affects_lightmapped_meshes: false,
            priority: 0,
            layer_mask: RenderLayerSet::default(),
        });
    assert_ne!(baseline_fingerprint, fingerprint_for(&volume));
}

#[test]
fn render_graph_compile_frame_fingerprint_tracks_camera_target_and_stack_inputs() {
    let mut baseline = test_extract();
    baseline.apply_viewport_size(UVec2::new(128, 64));
    let baseline_fingerprint = fingerprint_for(&baseline);

    let mut texture = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline).with_target(RenderCameraTarget::Texture(texture_handle(
            "res://target-a.png",
        ))),
    );
    texture.apply_viewport_size(UVec2::new(128, 64));
    assert_ne!(baseline_fingerprint, fingerprint_for(&texture));

    let mut other_texture = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline).with_target(RenderCameraTarget::Texture(texture_handle(
            "res://target-b.png",
        ))),
    );
    other_texture.apply_viewport_size(UVec2::new(128, 64));
    assert_ne!(fingerprint_for(&texture), fingerprint_for(&other_texture));

    let mut headless = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline).with_target(RenderCameraTarget::Headless {
            size: UVec2::new(128, 64),
        }),
    );
    headless.apply_viewport_size(UVec2::new(128, 64));
    assert_ne!(baseline_fingerprint, fingerprint_for(&headless));

    let mut viewport = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline).with_viewport_rect(RenderViewportRect::new(
            UVec2::new(16, 8),
            UVec2::new(64, 32),
        )),
    );
    viewport.apply_viewport_size(UVec2::new(128, 64));
    assert_ne!(baseline_fingerprint, fingerprint_for(&viewport));

    let mut overlay = baseline.clone().with_selected_camera_descriptor(
        selected_descriptor(&baseline).with_render_type(CameraRenderType::Overlay),
    );
    overlay.apply_viewport_size(UVec2::new(128, 64));
    assert_ne!(baseline_fingerprint, fingerprint_for(&overlay));
}

#[test]
fn compiled_render_pipeline_cache_key_tracks_texture_target_format_class() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let texture = texture_handle("res://target-format.png");
    let extract = test_extract();
    let descriptor =
        selected_descriptor(&extract).with_target(RenderCameraTarget::Texture(texture));
    let mut extract = extract.with_selected_camera_descriptor(descriptor);
    extract.apply_viewport_size(UVec2::new(128, 64));
    let srgb_key = key_for_with_camera_target(
        &pipeline,
        &extract,
        super::RenderGraphCompileCameraTargetFingerprint::Texture {
            id: texture.id(),
            width: 128,
            height: 64,
            format: super::RenderGraphCompileTextureTargetFormat::Rgba8UnormSrgb,
        },
        &RenderPipelineCompileOptions::default(),
    );
    let linear_key = key_for_with_camera_target(
        &pipeline,
        &extract,
        super::RenderGraphCompileCameraTargetFingerprint::Texture {
            id: texture.id(),
            width: 128,
            height: 64,
            format: super::RenderGraphCompileTextureTargetFormat::Rgba8Unorm,
        },
        &RenderPipelineCompileOptions::default(),
    );

    assert_ne!(srgb_key, linear_key);
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
    key_for_with_camera_target(
        pipeline,
        extract,
        camera_target_fingerprint_for_extract(extract),
        options,
    )
}

fn key_for_with_camera_target(
    pipeline: &RenderPipelineAsset,
    extract: &RenderFrameExtract,
    camera_target: super::RenderGraphCompileCameraTargetFingerprint,
    options: &RenderPipelineCompileOptions,
) -> CompiledGraphCacheKey {
    CompiledGraphCacheKey::from_inputs(
        pipeline,
        extract,
        camera_target,
        options,
        &RenderCapabilitySummary::default(),
        Default::default(),
    )
}

fn fingerprint_for(extract: &RenderFrameExtract) -> super::RenderGraphCompileFrameFingerprint {
    super::extract_compile_fingerprint(extract, camera_target_fingerprint_for_extract(extract))
}

fn camera_target_fingerprint_for_extract(
    extract: &RenderFrameExtract,
) -> super::RenderGraphCompileCameraTargetFingerprint {
    let view_size = extract.view.effective_view_size();
    let target = &selected_descriptor(extract).target;
    super::RenderGraphCompileCameraTargetFingerprint::from_target_and_view_size(target, view_size)
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
        environment_ibl_bake_request: None,
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
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
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
    fn with_dynamic_resolution(self, dynamic_resolution: RenderDynamicResolutionSettings) -> Self;
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
