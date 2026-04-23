use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderSceneSnapshot,
    RenderWorldSnapshotHandle,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::world::World;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiResolveRuntime,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn hybrid_gi_resolve_uses_atlas_only_persisted_surface_cache_page_sample_without_runtime_voxel_support(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();

    let warm_atlas_rgba = [240, 96, 48, 255];
    let cool_atlas_rgba = [48, 96, 240, 255];
    let warm_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, warm_atlas_rgba)],
            Vec::new(),
        ),
    );
    let cool_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, cool_atlas_rgba)],
            Vec::new(),
        ),
    );

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_resources = warm_readback
        .scene_prepare_resources
        .expect("expected warm scene-prepare resources");
    let cool_resources = cool_readback
        .scene_prepare_resources
        .expect("expected cool scene-prepare resources");
    assert_eq!(
        warm_resources.atlas_slot_rgba_samples,
        vec![(0, warm_atlas_rgba)]
    );
    assert_eq!(
        cool_resources.atlas_slot_rgba_samples,
        vec![(0, cool_atlas_rgba)]
    );
    assert!(
        warm_resources.capture_slot_rgba_samples.is_empty()
            && cool_resources.capture_slot_rgba_samples.is_empty(),
        "expected atlas-only persisted pages to stay atlas-only at render level instead of fabricating capture-side truth"
    );

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.1,
        "expected atlas-only persisted page truth to drive warm GI without runtime voxel support; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.1,
        "expected atlas-only persisted page truth to preserve blue authority without runtime voxel support; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_prefers_capture_surface_cache_page_sample_over_atlas_without_runtime_voxel_support(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();

    let warm_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [48, 96, 240, 255])],
            vec![(0, [240, 96, 48, 255])],
        ),
    );
    let cool_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [240, 96, 48, 255])],
            vec![(0, [48, 96, 240, 255])],
        ),
    );

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_resources = warm_readback
        .scene_prepare_resources
        .expect("expected warm scene-prepare resources");
    let cool_resources = cool_readback
        .scene_prepare_resources
        .expect("expected cool scene-prepare resources");
    assert_eq!(
        warm_resources.capture_slot_rgba_samples,
        vec![(0, [240, 96, 48, 255])]
    );
    assert_eq!(
        cool_resources.capture_slot_rgba_samples,
        vec![(0, [48, 96, 240, 255])]
    );

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.1,
        "expected capture-side persisted page truth to stay authoritative over atlas-side truth without runtime voxel support; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.1,
        "expected capture-side persisted page truth to preserve blue authority over atlas-side truth without runtime voxel support; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_atlas_only_runtime_scene_voxel_radiance_rehydrated_from_persisted_page_sample_on_clean_frame(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();

    let warm_scene_prepare =
        stripped_surface_cache_page_contents(runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [240, 96, 48, 255])],
            Vec::new(),
        ));
    let cool_scene_prepare =
        stripped_surface_cache_page_contents(runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [48, 96, 240, 255])],
            Vec::new(),
        ));

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_resources = warm_readback
        .scene_prepare_resources
        .expect("expected warm scene-prepare resources");
    let cool_resources = cool_readback
        .scene_prepare_resources
        .expect("expected cool scene-prepare resources");
    assert!(
        warm_resources.capture_slot_rgba_samples.is_empty()
            && warm_resources.atlas_slot_rgba_samples.is_empty()
            && cool_resources.capture_slot_rgba_samples.is_empty()
            && cool_resources.atlas_slot_rgba_samples.is_empty(),
        "expected atlas-only runtime-voxel rehydration proof to strip page-content fallback from renderer input so the GI difference must come from runtime voxel radiance"
    );

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected atlas-only persisted page samples to rehydrate warm runtime voxel radiance on clean frames; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected atlas-only persisted page samples to rehydrate cool runtime voxel radiance on clean frames; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_prefers_capture_over_atlas_in_runtime_scene_voxel_radiance_rehydration_on_clean_frame(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();

    let warm_scene_prepare =
        stripped_surface_cache_page_contents(runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [48, 96, 240, 255])],
            vec![(0, [240, 96, 48, 255])],
        ));
    let cool_scene_prepare =
        stripped_surface_cache_page_contents(runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [240, 96, 48, 255])],
            vec![(0, [48, 96, 240, 255])],
        ));

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_resources = warm_readback
        .scene_prepare_resources
        .expect("expected warm scene-prepare resources");
    let cool_resources = cool_readback
        .scene_prepare_resources
        .expect("expected cool scene-prepare resources");
    assert!(
        warm_resources.capture_slot_rgba_samples.is_empty()
            && warm_resources.atlas_slot_rgba_samples.is_empty()
            && cool_resources.capture_slot_rgba_samples.is_empty()
            && cool_resources.atlas_slot_rgba_samples.is_empty(),
        "expected capture-preferred runtime-voxel rehydration proof to strip page-content fallback from renderer input so the GI difference must come from runtime voxel radiance"
    );

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected capture-side persisted page truth to stay authoritative when clean-frame runtime voxel radiance is rehydrated; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected capture-side persisted page truth to preserve blue authority when clean-frame runtime voxel radiance is rehydrated; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_surface_cache_page_truth_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();
    let history_handle = crate::FrameHistoryHandle::new(15);

    let warm_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [240, 96, 48, 255])],
            Vec::new(),
        ),
    );
    let cool_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [48, 96, 240, 255])],
            Vec::new(),
        ),
    );

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    let with_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare.clone())),
            &compiled,
            Some(history_handle),
        )
        .unwrap();
    let without_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let with_history_blue =
        average_region_channel(&with_history.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let without_history_blue = average_region_channel(
        &without_history.rgba,
        viewport_size,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        with_history_red < without_history_red + 0.05,
        "expected GI history to reject stale warm surface-cache page truth once the dominant page sample turns cool, instead of preserving the old red energy across frames; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
    assert!(
        with_history_blue + 0.05 > without_history_blue,
        "expected GI history to keep the new cool surface-cache page truth visible instead of reusing too much stale warm history; with_history_blue={with_history_blue:.2}, without_history_blue={without_history_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_exact_runtime_rt_with_current_surface_cache_truth_when_trace_schedule_is_empty(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap();
    let neutral_runtime = HybridGiResolveRuntime {
        probe_rt_lighting_rgb: std::collections::BTreeMap::from([(200, [120, 120, 120])]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.47, 0.47, 0.47], 0.62),
        )]),
        ..Default::default()
    };
    let warm_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [224, 112, 64, 255])],
            vec![(0, [240, 96, 48, 255])],
        ),
    );
    let cool_scene_prepare = scene_prepare_without_runtime_voxel_support(
        runtime_scene_prepare_from_persisted_page_samples(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            vec![(0, [64, 112, 224, 255])],
            vec![(0, [48, 96, 240, 255])],
        ),
    );

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare))
                .with_hybrid_gi_resolve_runtime(Some(neutral_runtime.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare))
                .with_hybrid_gi_resolve_runtime(Some(neutral_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.08,
        "expected exact runtime RT history to keep blending with current warm surface-cache truth when there is no current trace schedule, instead of flattening warm/cool frames to the same runtime-only resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.08,
        "expected exact runtime RT history to keep blending with current cool surface-cache truth when there is no current trace schedule, instead of flattening warm/cool frames to the same runtime-only resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

fn build_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.directional_lights.clear();
    snapshot.preview.clear_color = Vec4::ZERO;
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 0,
        probes: vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        trace_regions: Vec::new(),
    });
    extract
}

fn resident_prepare() -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    }
}

fn probe(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: Vec3,
    radius: f32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position,
        radius,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn scene_prepare_without_runtime_voxel_support(
    scene_prepare: HybridGiScenePrepareFrame,
) -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
        ..scene_prepare
    }
}

fn stripped_surface_cache_page_contents(
    scene_prepare: HybridGiScenePrepareFrame,
) -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        surface_cache_page_contents: Vec::new(),
        ..scene_prepare
    }
}

fn runtime_scene_prepare_from_persisted_page_samples(
    tint_rgb: [f32; 3],
    translation: Vec3,
    uniform_scale: f32,
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
) -> HybridGiScenePrepareFrame {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    runtime.register_scene_extract(
        Some(&extract),
        &[crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/runtime-voxel-persisted-page.mat",
            )),
            tint: Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );

    let occupied_atlas_slots = atlas_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect::<Vec<_>>();
    let occupied_capture_slots = capture_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect::<Vec<_>>();
    runtime.apply_scene_prepare_resources(
        &crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot {
            card_capture_request_count: occupied_capture_slots.len() as u32,
            voxel_clipmap_ids: Vec::new(),
            occupied_atlas_slots,
            occupied_capture_slots,
            atlas_slot_rgba_samples,
            capture_slot_rgba_samples,
            voxel_clipmap_rgba_samples: Vec::new(),
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples: Vec::new(),
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
            atlas_slot_count: 0,
            capture_slot_count: 0,
            atlas_texture_extent: (0, 0),
            capture_texture_extent: (0, 0),
            capture_layer_count: 0,
        },
    );
    runtime.register_scene_extract(
        Some(&extract),
        &[crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/runtime-voxel-persisted-page.mat",
            )),
            tint: Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );

    let frame = runtime.build_scene_prepare_frame();
    assert!(
        frame.card_capture_requests.is_empty(),
        "expected the second unchanged scene registration to become a clean frame"
    );
    frame
}

fn average_region_channel(
    rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let start_x = ((width as f32) * x_min.clamp(0.0, 1.0)).floor() as usize;
    let end_x = ((width as f32) * x_max.clamp(0.0, 1.0)).ceil() as usize;
    let start_y = ((height as f32) * y_min.clamp(0.0, 1.0)).floor() as usize;
    let end_y = ((height as f32) * y_max.clamp(0.0, 1.0)).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in start_y.min(height)..end_y.min(height).max(start_y.min(height) + 1) {
        for x in start_x.min(width)..end_x.min(width).max(start_x.min(width) + 1) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }

    total / count as f32
}
