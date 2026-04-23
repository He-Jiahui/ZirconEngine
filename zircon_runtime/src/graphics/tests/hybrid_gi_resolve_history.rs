use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{RenderDirectionalLightSnapshot, RenderMeshSnapshot};
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderSceneSnapshot,
    RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot;
use crate::scene::world::World;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareSurfaceCachePageContent,
        HybridGiResolveRuntime, HybridGiScenePrepareFrame, ViewportFrame, ViewportRenderFrame,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn hybrid_gi_resolve_rejects_history_when_scene_driven_exact_runtime_truth_changes() {
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
    let history_handle = crate::FrameHistoryHandle::new(16);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_resolve_runtime(Some(scene_driven_runtime(
                    [240, 96, 48],
                    [224, 112, 64],
                ))),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    let with_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_resolve_runtime(Some(scene_driven_runtime(
                    [48, 96, 240],
                    [64, 112, 224],
                ))),
            &compiled,
            Some(history_handle),
        )
        .unwrap();
    let without_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare()))
                .with_hybrid_gi_resolve_runtime(Some(scene_driven_runtime(
                    [48, 96, 240],
                    [64, 112, 224],
                ))),
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
        "expected GI history to reject stale warm exact-runtime scene truth once the scene-driven runtime source turns cool, instead of preserving old red energy; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
    assert!(
        with_history_blue + 0.05 > without_history_blue,
        "expected GI history to keep the new cool exact-runtime scene truth visible instead of over-reusing stale warm history; with_history_blue={with_history_blue:.2}, without_history_blue={without_history_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gives_scene_driven_exact_runtime_truth_more_history_reuse_than_continuation() {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);

    let (scene_driven_with_history, scene_driven_without_history) =
        render_second_frame_with_and_without_history(
            extract.clone(),
            viewport_size,
            runtime_with_scene_truth_flags([160, 160, 160], [160, 160, 160]),
        );
    let (continuation_with_history, continuation_without_history) =
        render_second_frame_with_and_without_history(
            extract,
            viewport_size,
            runtime_without_scene_truth_flags([160, 160, 160], [160, 160, 160]),
        );

    let scene_driven_with_history_red = average_region_channel(
        &scene_driven_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let scene_driven_without_history_red = average_region_channel(
        &scene_driven_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let continuation_with_history_red = average_region_channel(
        &continuation_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let continuation_without_history_red = average_region_channel(
        &continuation_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let scene_driven_history_red_boost =
        scene_driven_with_history_red - scene_driven_without_history_red;
    let continuation_history_red_boost =
        continuation_with_history_red - continuation_without_history_red;

    assert!(
        (scene_driven_without_history_red - continuation_without_history_red).abs() < 0.05,
        "expected current-frame GI to stay materially aligned before temporal reuse weighting changes; scene_driven_without_history_red={scene_driven_without_history_red:.2}, continuation_without_history_red={continuation_without_history_red:.2}"
    );
    assert!(
        scene_driven_history_red_boost > continuation_history_red_boost + 0.04,
        "expected scene-driven exact runtime truth to keep stronger GI history reuse than non-scene-driven continuation when the temporal signature stays fixed; scene_driven_history_red_boost={scene_driven_history_red_boost:.2}, continuation_history_red_boost={continuation_history_red_boost:.2}"
    );
    assert!(
        scene_driven_with_history_red > continuation_with_history_red + 0.04,
        "expected scene-driven exact runtime truth to preserve more warm history than continuation under the same current cool probe frame; scene_driven_with_history_red={scene_driven_with_history_red:.2}, continuation_with_history_red={continuation_with_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_exact_runtime_truth_keeps_history_when_only_probe_identity_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let stable_probe_id = 200;
    let changed_probe_id = 275;
    let stable_extract = build_extract_for_probe(viewport_size, stable_probe_id);

    let (stable_with_history, stable_without_history) =
        render_second_frame_with_and_without_history(
            stable_extract,
            viewport_size,
            runtime_with_scene_truth_flags_for_probe(
                stable_probe_id,
                [160, 160, 160],
                [160, 160, 160],
            ),
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_probe_identity_transition_with_and_without_history(
            viewport_size,
            build_extract_for_probe(viewport_size, stable_probe_id),
            resident_prepare_for_probe_with_irradiance(stable_probe_id, [240, 96, 48]),
            runtime_with_scene_truth_flags_for_probe(
                stable_probe_id,
                [160, 160, 160],
                [160, 160, 160],
            ),
            build_extract_for_probe(viewport_size, changed_probe_id),
            resident_prepare_for_probe_with_irradiance(changed_probe_id, [48, 96, 240]),
            runtime_with_scene_truth_flags_for_probe(
                changed_probe_id,
                [160, 160, 160],
                [160, 160, 160],
            ),
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 0.05,
        "expected the current cool frame to stay materially aligned before temporal identity handling changes history reuse; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven exact runtime truth to preserve nearly the same warm history even when only the legacy probe id changes, instead of treating authored probe identity as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven exact runtime truth to keep visible warm history after a pure authored probe-id transition when runtime scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_exact_runtime_truth_keeps_history_when_only_descendant_identity_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let probe_id = 200;
    let stable_descendant_probe_id = 260;
    let changed_descendant_probe_id = 275;
    let stable_runtime = runtime_with_scene_truth_flags_for_probe_and_descendant(
        probe_id,
        stable_descendant_probe_id,
        [160, 160, 160],
        [160, 160, 160],
    );

    let (stable_with_history, stable_without_history) =
        render_second_frame_with_and_without_history(
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                stable_descendant_probe_id,
            ),
            viewport_size,
            stable_runtime,
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_probe_identity_transition_with_and_without_history(
            viewport_size,
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                stable_descendant_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [240, 96, 48]),
            runtime_with_scene_truth_flags_for_probe_and_descendant(
                probe_id,
                stable_descendant_probe_id,
                [160, 160, 160],
                [160, 160, 160],
            ),
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                changed_descendant_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [48, 96, 240]),
            runtime_with_scene_truth_flags_for_probe_and_descendant(
                probe_id,
                changed_descendant_probe_id,
                [160, 160, 160],
                [160, 160, 160],
            ),
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 0.05,
        "expected the current cool frame to stay materially aligned before descendant identity handling changes history reuse; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven exact runtime truth to preserve nearly the same warm history even when only a descendant probe id changes, instead of treating descendant lineage identity as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven exact runtime truth to keep visible warm history after a pure descendant probe-id transition when current exact scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_descendant_identity_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let probe_id = 200;
    let stable_descendant_probe_id = 260;
    let changed_descendant_probe_id = 275;
    let stable_runtime = runtime_with_scene_truth_flags_for_descendant_only(
        probe_id,
        stable_descendant_probe_id,
        [160, 160, 160],
    );

    let (stable_with_history, stable_without_history) =
        render_second_frame_with_and_without_history(
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                stable_descendant_probe_id,
            ),
            viewport_size,
            stable_runtime,
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_probe_identity_transition_with_and_without_history(
            viewport_size,
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                stable_descendant_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [240, 96, 48]),
            runtime_with_scene_truth_flags_for_descendant_only(
                probe_id,
                stable_descendant_probe_id,
                [160, 160, 160],
            ),
            build_extract_for_probe_with_descendant(
                viewport_size,
                probe_id,
                changed_descendant_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [48, 96, 240]),
            runtime_with_scene_truth_flags_for_descendant_only(
                probe_id,
                changed_descendant_probe_id,
                [160, 160, 160],
            ),
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 0.05,
        "expected the current cool frame to stay materially aligned before descendant-runtime identity handling changes history reuse; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven descendant runtime truth to preserve nearly the same warm history even when only a descendant probe id changes, instead of treating descendant runtime lineage identity as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven descendant runtime truth to keep visible warm history after a pure descendant probe-id transition when current descendant scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_descendant_depth_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let probe_id = 200;
    let intermediate_probe_id = 240;
    let descendant_probe_id = 260;
    let runtime = runtime_with_scene_truth_flags_for_descendant_rt_and_irradiance(
        probe_id,
        descendant_probe_id,
        [160, 160, 160],
        [160, 160, 160],
    );

    let (stable_with_history, stable_without_history) =
        render_second_frame_with_and_without_history(
            build_extract_for_probe_with_descendant(viewport_size, probe_id, descendant_probe_id),
            viewport_size,
            runtime.clone(),
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_probe_identity_transition_with_and_without_history(
            viewport_size,
            build_extract_for_probe_with_descendant(viewport_size, probe_id, descendant_probe_id),
            resident_prepare_for_probe_with_irradiance(probe_id, [240, 96, 48]),
            runtime.clone(),
            build_extract_for_probe_with_descendant_grandchild(
                viewport_size,
                probe_id,
                intermediate_probe_id,
                descendant_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [48, 96, 240]),
            runtime,
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 0.75,
        "expected the current cool frame to stay materially aligned before authored descendant-depth handling changes GI, because the same leaf scene truth still drives the parent probe; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven descendant runtime truth to preserve nearly the same warm history even when only an intermediate authored descendant node is inserted, instead of treating descendant depth as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven descendant runtime truth to keep visible warm history after a pure authored descendant-depth transition when current leaf scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_history_when_only_scene_prepare_surface_cache_tint_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let probe_id = 200;
    let descendant_probe_id = 260;
    let extract =
        build_extract_for_probe_with_descendant(viewport_size, probe_id, descendant_probe_id);
    let runtime = runtime_with_scene_truth_flags_for_descendant_only(
        probe_id,
        descendant_probe_id,
        [160, 160, 160],
    );

    let (stable_with_history, stable_without_history) =
        render_second_frame_after_scene_prepare_transition_with_and_without_history(
            extract.clone(),
            viewport_size,
            runtime.clone(),
            centered_surface_cache_scene_prepare([240, 96, 48, 255], [224, 112, 64, 255]),
            centered_surface_cache_scene_prepare([240, 96, 48, 255], [224, 112, 64, 255]),
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_scene_prepare_transition_with_and_without_history(
            extract,
            viewport_size,
            runtime,
            centered_surface_cache_scene_prepare([240, 96, 48, 255], [224, 112, 64, 255]),
            centered_surface_cache_scene_prepare([48, 96, 240, 255], [64, 112, 224, 255]),
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 1.5,
        "expected current-frame descendant scene-truth GI to stay materially aligned when only the current surface-cache page tint changes, before temporal reuse is considered; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven descendant runtime truth to preserve nearly the same warm history even when only the current surface-cache page tint changes, instead of treating current page tint as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven descendant runtime truth to keep visible warm history after a pure current-page tint transition when runtime scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_history_when_only_ancestor_depth_changes(
) {
    let viewport_size = UVec2::new(96, 64);
    let probe_id = 200;
    let intermediate_probe_id = 150;
    let ancestor_probe_id = 100;
    let runtime = runtime_with_scene_truth_flags_for_probe(
        ancestor_probe_id,
        [160, 160, 160],
        [160, 160, 160],
    );

    let (stable_with_history, stable_without_history) =
        render_second_frame_with_and_without_history(
            build_extract_for_probe_with_parent(viewport_size, probe_id, ancestor_probe_id),
            viewport_size,
            runtime.clone(),
        );
    let (changed_with_history, changed_without_history) =
        render_second_frame_after_probe_identity_transition_with_and_without_history(
            viewport_size,
            build_extract_for_probe_with_parent(viewport_size, probe_id, ancestor_probe_id),
            resident_prepare_for_probe_with_irradiance(probe_id, [240, 96, 48]),
            runtime.clone(),
            build_extract_for_probe_with_grandparent(
                viewport_size,
                probe_id,
                intermediate_probe_id,
                ancestor_probe_id,
            ),
            resident_prepare_for_probe_with_irradiance(probe_id, [48, 96, 240]),
            runtime,
        );

    let stable_with_history_red = average_region_channel(
        &stable_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_without_history_red = average_region_channel(
        &stable_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_with_history_red = average_region_channel(
        &changed_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let changed_without_history_red = average_region_channel(
        &changed_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let stable_history_red_boost = stable_with_history_red - stable_without_history_red;
    let changed_history_red_boost = changed_with_history_red - changed_without_history_red;

    assert!(
        (stable_without_history_red - changed_without_history_red).abs() < 0.75,
        "expected the current cool frame to stay materially aligned before authored ancestor-depth handling changes GI, because the same inherited scene truth still reaches the current probe; stable_without_history_red={stable_without_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
    assert!(
        changed_history_red_boost > stable_history_red_boost - 0.04,
        "expected scene-driven inherited runtime truth to preserve nearly the same warm history even when only an intermediate authored ancestor node is inserted, instead of treating ancestor depth as temporal truth; stable_history_red_boost={stable_history_red_boost:.2}, changed_history_red_boost={changed_history_red_boost:.2}"
    );
    assert!(
        changed_with_history_red > changed_without_history_red + 0.04,
        "expected scene-driven inherited runtime truth to keep visible warm history after a pure authored ancestor-depth transition when current inherited scene truth itself stayed fixed; changed_with_history_red={changed_with_history_red:.2}, changed_without_history_red={changed_without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gives_higher_quality_scene_driven_runtime_truth_more_history_reuse() {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);

    let (high_quality_with_history, high_quality_without_history) =
        render_second_frame_with_and_without_history(
            extract.clone(),
            viewport_size,
            runtime_with_scene_truth_support_and_quality(
                [160, 160, 160],
                [160, 160, 160],
                0.52,
                1.0,
            ),
        );
    let (low_quality_with_history, low_quality_without_history) =
        render_second_frame_with_and_without_history(
            extract,
            viewport_size,
            runtime_with_scene_truth_support_and_quality(
                [160, 160, 160],
                [160, 160, 160],
                0.52,
                0.2,
            ),
        );

    let high_quality_with_history_red = average_region_channel(
        &high_quality_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let high_quality_without_history_red = average_region_channel(
        &high_quality_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let low_quality_with_history_red = average_region_channel(
        &low_quality_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let low_quality_without_history_red = average_region_channel(
        &low_quality_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let high_quality_history_red_boost =
        high_quality_with_history_red - high_quality_without_history_red;
    let low_quality_history_red_boost =
        low_quality_with_history_red - low_quality_without_history_red;

    assert!(
        (high_quality_without_history_red - low_quality_without_history_red).abs() < 0.05,
        "expected current-frame GI to stay materially aligned before temporal quality weighting changes history reuse; high_quality_without_history_red={high_quality_without_history_red:.2}, low_quality_without_history_red={low_quality_without_history_red:.2}"
    );
    assert!(
        high_quality_history_red_boost > low_quality_history_red_boost + 0.04,
        "expected higher-quality scene-driven runtime truth to preserve more history than lower-quality truth at the same runtime support; high_quality_history_red_boost={high_quality_history_red_boost:.2}, low_quality_history_red_boost={low_quality_history_red_boost:.2}"
    );
    assert!(
        high_quality_with_history_red > low_quality_with_history_red + 0.04,
        "expected higher-quality scene-driven runtime truth to keep more warm history visible than lower-quality truth under the same current cool probe frame; high_quality_with_history_red={high_quality_with_history_red:.2}, low_quality_with_history_red={low_quality_with_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gives_clean_surface_cache_scene_truth_more_history_reuse_than_dirty_surface_cache_truth(
) {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);

    let (clean_with_history, clean_without_history) = render_second_frame_with_and_without_history(
        extract.clone(),
        viewport_size,
        runtime_from_surface_cache_scene_truth(true),
    );
    let (dirty_with_history, dirty_without_history) = render_second_frame_with_and_without_history(
        extract,
        viewport_size,
        runtime_from_surface_cache_scene_truth(false),
    );

    let clean_with_history_red = average_region_channel(
        &clean_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let clean_without_history_red = average_region_channel(
        &clean_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let dirty_with_history_red = average_region_channel(
        &dirty_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let dirty_without_history_red = average_region_channel(
        &dirty_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let clean_history_red_boost = clean_with_history_red - clean_without_history_red;
    let dirty_history_red_boost = dirty_with_history_red - dirty_without_history_red;

    assert!(
        (clean_without_history_red - dirty_without_history_red).abs() < 0.05,
        "expected the current cool frame to stay materially aligned before surface-cache freshness changes only the temporal reuse weight; clean_without_history_red={clean_without_history_red:.2}, dirty_without_history_red={dirty_without_history_red:.2}"
    );
    assert!(
        clean_history_red_boost > dirty_history_red_boost + 0.04,
        "expected clean surface-cache scene truth to preserve more warm history than dirty surface-cache truth when the current frame GI is otherwise the same; clean_history_red_boost={clean_history_red_boost:.2}, dirty_history_red_boost={dirty_history_red_boost:.2}"
    );
    assert!(
        clean_with_history_red > dirty_with_history_red + 0.04,
        "expected clean surface-cache scene truth to keep more warm history visible than dirty surface-cache truth under the same current cool probe frame; clean_with_history_red={clean_with_history_red:.2}, dirty_with_history_red={dirty_with_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gives_clean_voxel_scene_truth_more_history_reuse_than_dirty_voxel_scene_truth()
{
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);

    let (clean_with_history, clean_without_history) = render_second_frame_with_and_without_history(
        extract.clone(),
        viewport_size,
        runtime_from_voxel_scene_truth(true),
    );
    let (dirty_with_history, dirty_without_history) = render_second_frame_with_and_without_history(
        extract,
        viewport_size,
        runtime_from_voxel_scene_truth(false),
    );

    let clean_with_history_red = average_region_channel(
        &clean_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let clean_without_history_red = average_region_channel(
        &clean_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let dirty_with_history_red = average_region_channel(
        &dirty_with_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let dirty_without_history_red = average_region_channel(
        &dirty_without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let clean_history_red_boost = clean_with_history_red - clean_without_history_red;
    let dirty_history_red_boost = dirty_with_history_red - dirty_without_history_red;

    assert!(
        (clean_without_history_red - dirty_without_history_red).abs() < 0.05,
        "expected the current cool frame to stay materially aligned before voxel freshness changes only the temporal reuse weight; clean_without_history_red={clean_without_history_red:.2}, dirty_without_history_red={dirty_without_history_red:.2}"
    );
    assert!(
        clean_history_red_boost > dirty_history_red_boost + 0.04,
        "expected clean voxel scene truth to preserve more warm history than dirty voxel truth when the current frame GI is otherwise the same; clean_history_red_boost={clean_history_red_boost:.2}, dirty_history_red_boost={dirty_history_red_boost:.2}"
    );
    assert!(
        clean_with_history_red > dirty_with_history_red + 0.04,
        "expected clean voxel scene truth to keep more warm history visible than dirty voxel truth under the same current cool probe frame; clean_with_history_red={clean_with_history_red:.2}, dirty_with_history_red={dirty_with_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_history_when_surface_cache_scene_truth_freshness_changes_without_rgb_change(
) {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let clean_runtime = runtime_from_surface_cache_scene_truth(true);
    let dirty_runtime = runtime_from_surface_cache_scene_truth(false);

    assert_eq!(
        clean_runtime.hierarchy_irradiance(200),
        dirty_runtime.hierarchy_irradiance(200),
        "expected the surface-cache freshness transition test to isolate temporal validity only, not current-frame irradiance color/support changes"
    );
    assert_eq!(
        clean_runtime.hierarchy_rt_lighting(200),
        dirty_runtime.hierarchy_rt_lighting(200),
        "expected the surface-cache freshness transition test to keep exact runtime RT lighting identical so any second-frame difference comes from history rejection only"
    );

    let with_history = render_second_frame_after_runtime_transition(
        extract.clone(),
        viewport_size,
        clean_runtime,
        dirty_runtime.clone(),
    );
    let without_history = render_second_frame(extract, viewport_size, dirty_runtime, None);

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

    assert!(
        with_history_red < without_history_red + 0.05,
        "expected a clean-to-dirty surface-cache scene-truth transition to invalidate stale warm GI history even when runtime RGB/support stay fixed; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_history_when_voxel_scene_truth_freshness_changes_without_rgb_change() {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let clean_runtime = runtime_from_voxel_scene_truth(true);
    let dirty_runtime = runtime_from_voxel_scene_truth(false);

    assert_eq!(
        clean_runtime.hierarchy_irradiance(200),
        dirty_runtime.hierarchy_irradiance(200),
        "expected the voxel freshness transition test to isolate temporal validity only, not current-frame irradiance color/support changes"
    );
    assert_eq!(
        clean_runtime.hierarchy_rt_lighting(200),
        dirty_runtime.hierarchy_rt_lighting(200),
        "expected the voxel freshness transition test to keep exact runtime RT lighting identical so any second-frame difference comes from history rejection only"
    );

    let with_history = render_second_frame_after_runtime_transition(
        extract.clone(),
        viewport_size,
        clean_runtime,
        dirty_runtime.clone(),
    );
    let without_history = render_second_frame(extract, viewport_size, dirty_runtime, None);

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

    assert!(
        with_history_red < without_history_red + 0.05,
        "expected a clean-to-dirty voxel scene-truth transition to invalidate stale warm GI history even when runtime RGB/support stay fixed; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_history_when_surface_cache_scene_truth_revision_changes_without_rgb_or_freshness_change(
) {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let baseline_runtime = runtime_from_surface_cache_scene_truth_clean_with_light_node_id(300);
    let transitioned_runtime =
        runtime_from_surface_cache_scene_truth_clean_after_light_node_id_transition(300, 301);

    assert_eq!(
        baseline_runtime.hierarchy_irradiance(200),
        transitioned_runtime.hierarchy_irradiance(200),
        "expected the surface-cache revision transition test to isolate temporal validity only, not current-frame irradiance color/support changes"
    );
    assert_eq!(
        baseline_runtime.hierarchy_rt_lighting(200),
        transitioned_runtime.hierarchy_rt_lighting(200),
        "expected the surface-cache revision transition test to keep exact runtime RT lighting identical so any second-frame difference comes from history rejection only"
    );
    assert_eq!(
        baseline_runtime.hierarchy_irradiance_scene_truth_freshness(200),
        transitioned_runtime.hierarchy_irradiance_scene_truth_freshness(200),
        "expected the surface-cache revision transition test to keep irradiance freshness fixed"
    );
    assert_eq!(
        baseline_runtime.hierarchy_rt_lighting_scene_truth_freshness(200),
        transitioned_runtime.hierarchy_rt_lighting_scene_truth_freshness(200),
        "expected the surface-cache revision transition test to keep RT freshness fixed"
    );
    assert_ne!(
        baseline_runtime.hierarchy_irradiance_scene_truth_revision(200),
        transitioned_runtime.hierarchy_irradiance_scene_truth_revision(200),
        "expected the light-node-id transition to produce a different surface-cache scene-truth revision even though the final capture RGB/support/freshness stay the same"
    );
    assert_ne!(
        baseline_runtime.hierarchy_rt_lighting_scene_truth_revision(200),
        transitioned_runtime.hierarchy_rt_lighting_scene_truth_revision(200),
        "expected the light-node-id transition to produce a different RT scene-truth revision even though the final capture RGB/support/freshness stay the same"
    );

    let with_history = render_second_frame_after_runtime_transition(
        extract.clone(),
        viewport_size,
        baseline_runtime,
        transitioned_runtime.clone(),
    );
    let without_history = render_second_frame(extract, viewport_size, transitioned_runtime, None);

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

    assert!(
        with_history_red < without_history_red + 0.05,
        "expected a stabilized light-topology transition to invalidate stale warm GI history even after surface-cache freshness has returned clean when runtime RGB/support stay fixed; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_history_when_voxel_scene_truth_revision_changes_without_rgb_or_freshness_change(
) {
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let baseline_runtime = runtime_from_voxel_scene_truth_clean_with_light_node_id(300);
    let transitioned_runtime =
        runtime_from_voxel_scene_truth_clean_after_light_node_id_transition(300, 301);

    assert_eq!(
        baseline_runtime.hierarchy_irradiance(200),
        transitioned_runtime.hierarchy_irradiance(200),
        "expected the voxel revision transition test to isolate temporal validity only, not current-frame irradiance color/support changes"
    );
    assert_eq!(
        baseline_runtime.hierarchy_rt_lighting(200),
        transitioned_runtime.hierarchy_rt_lighting(200),
        "expected the voxel revision transition test to keep exact runtime RT lighting identical so any second-frame difference comes from history rejection only"
    );
    assert_eq!(
        baseline_runtime.hierarchy_irradiance_scene_truth_freshness(200),
        transitioned_runtime.hierarchy_irradiance_scene_truth_freshness(200),
        "expected the voxel revision transition test to keep irradiance freshness fixed"
    );
    assert_eq!(
        baseline_runtime.hierarchy_rt_lighting_scene_truth_freshness(200),
        transitioned_runtime.hierarchy_rt_lighting_scene_truth_freshness(200),
        "expected the voxel revision transition test to keep RT freshness fixed"
    );
    assert_eq!(
        baseline_runtime.hierarchy_irradiance_scene_truth_revision(200),
        transitioned_runtime.hierarchy_irradiance_scene_truth_revision(200),
        "expected voxel-only revision coverage to stay scoped to exact RT scene truth; exact irradiance scene truth still comes from the surface-cache path and should remain absent in this fixture"
    );
    assert_ne!(
        baseline_runtime.hierarchy_rt_lighting_scene_truth_revision(200),
        transitioned_runtime.hierarchy_rt_lighting_scene_truth_revision(200),
        "expected the light-node-id transition to produce a different voxel RT revision even though the final runtime RGB/support/freshness stay the same"
    );

    let with_history = render_second_frame_after_runtime_transition(
        extract.clone(),
        viewport_size,
        baseline_runtime,
        transitioned_runtime.clone(),
    );
    let without_history = render_second_frame(extract, viewport_size, transitioned_runtime, None);

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

    assert!(
        with_history_red < without_history_red + 0.05,
        "expected a stabilized light-topology transition to invalidate stale warm GI history even after voxel freshness has returned clean when runtime RGB/support stay fixed; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

fn build_extract(viewport_size: UVec2) -> RenderFrameExtract {
    build_extract_for_probe(viewport_size, 200)
}

fn build_extract_for_probe(viewport_size: UVec2, probe_id: u32) -> RenderFrameExtract {
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
        probes: vec![probe(probe_id, true, 128, Vec3::ZERO, 1.8)],
        trace_regions: Vec::new(),
    });
    extract
}

fn build_extract_for_probe_with_descendant(
    viewport_size: UVec2,
    probe_id: u32,
    descendant_probe_id: u32,
) -> RenderFrameExtract {
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
        probes: vec![
            probe(probe_id, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(descendant_probe_id, probe_id, false, 96, Vec3::ZERO, 1.2),
        ],
        trace_regions: Vec::new(),
    });
    extract
}

fn build_extract_for_probe_with_parent(
    viewport_size: UVec2,
    probe_id: u32,
    parent_probe_id: u32,
) -> RenderFrameExtract {
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
        probes: vec![
            probe(parent_probe_id, false, 128, Vec3::ZERO, 2.1),
            probe_with_parent(probe_id, parent_probe_id, true, 128, Vec3::ZERO, 1.8),
        ],
        trace_regions: Vec::new(),
    });
    extract
}

fn build_extract_for_probe_with_grandparent(
    viewport_size: UVec2,
    probe_id: u32,
    parent_probe_id: u32,
    grandparent_probe_id: u32,
) -> RenderFrameExtract {
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
        probes: vec![
            probe(grandparent_probe_id, false, 128, Vec3::ZERO, 2.3),
            probe_with_parent(
                parent_probe_id,
                grandparent_probe_id,
                false,
                128,
                Vec3::ZERO,
                2.1,
            ),
            probe_with_parent(probe_id, parent_probe_id, true, 128, Vec3::ZERO, 1.8),
        ],
        trace_regions: Vec::new(),
    });
    extract
}

fn build_extract_for_probe_with_descendant_grandchild(
    viewport_size: UVec2,
    probe_id: u32,
    intermediate_probe_id: u32,
    descendant_probe_id: u32,
) -> RenderFrameExtract {
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
        probes: vec![
            probe(probe_id, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(intermediate_probe_id, probe_id, false, 96, Vec3::ZERO, 1.5),
            probe_with_parent(
                descendant_probe_id,
                intermediate_probe_id,
                false,
                96,
                Vec3::ZERO,
                1.2,
            ),
        ],
        trace_regions: Vec::new(),
    });
    extract
}

fn resident_prepare() -> HybridGiPrepareFrame {
    resident_prepare_for_probe_with_irradiance(200, [112, 112, 112])
}

fn resident_prepare_with_irradiance(irradiance_rgb: [u8; 3]) -> HybridGiPrepareFrame {
    resident_prepare_for_probe_with_irradiance(200, irradiance_rgb)
}

fn resident_prepare_for_probe_with_irradiance(
    probe_id: u32,
    irradiance_rgb: [u8; 3],
) -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb,
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

fn probe_with_parent(
    probe_id: u32,
    parent_probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: Vec3,
    radius: f32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe_id),
        ..probe(probe_id, resident, ray_budget, position, radius)
    }
}

fn scene_driven_runtime(
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    runtime_with_scene_truth_flags_for_probe(200, rt_lighting_rgb, irradiance_rgb)
}

fn runtime_with_scene_truth_flags(
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    runtime_with_scene_truth_support_for_probe(200, rt_lighting_rgb, irradiance_rgb, 0.52)
}

fn runtime_with_scene_truth_flags_for_probe(
    probe_id: u32,
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    runtime_with_scene_truth_support_for_probe(probe_id, rt_lighting_rgb, irradiance_rgb, 0.52)
}

fn runtime_with_scene_truth_support_for_probe(
    probe_id: u32,
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
    irradiance_weight: f32,
) -> HybridGiResolveRuntime {
    runtime_with_scene_truth_support_and_quality_for_probe(
        probe_id,
        rt_lighting_rgb,
        irradiance_rgb,
        irradiance_weight,
        1.0,
    )
}

fn runtime_with_scene_truth_support_and_quality(
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
    irradiance_weight: f32,
    irradiance_quality: f32,
) -> HybridGiResolveRuntime {
    runtime_with_scene_truth_support_and_quality_for_probe(
        200,
        rt_lighting_rgb,
        irradiance_rgb,
        irradiance_weight,
        irradiance_quality,
    )
}

fn runtime_with_scene_truth_support_and_quality_for_probe(
    probe_id: u32,
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
    irradiance_weight: f32,
    irradiance_quality: f32,
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntime {
        probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
        )]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    irradiance_rgb[0] as f32 / 255.0,
                    irradiance_rgb[1] as f32 / 255.0,
                    irradiance_rgb[2] as f32 / 255.0,
                ],
                irradiance_weight,
            ),
        )]),
        probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    rt_lighting_rgb[0] as f32 / 255.0,
                    rt_lighting_rgb[1] as f32 / 255.0,
                    rt_lighting_rgb[2] as f32 / 255.0,
                ],
                0.0,
            ),
        )]),
        probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([probe_id]),
        probe_scene_driven_hierarchy_irradiance_quality_q8: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_scene_truth_quality_q8(irradiance_quality),
        )]),
        probe_scene_driven_hierarchy_rt_lighting_ids: std::collections::BTreeSet::new(),
        ..Default::default()
    }
}

fn runtime_with_scene_truth_flags_for_probe_and_descendant(
    probe_id: u32,
    descendant_probe_id: u32,
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntime {
        probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
        )]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([
            (
                probe_id,
                HybridGiResolveRuntime::pack_rgb_and_weight(
                    [
                        irradiance_rgb[0] as f32 / 255.0,
                        irradiance_rgb[1] as f32 / 255.0,
                        irradiance_rgb[2] as f32 / 255.0,
                    ],
                    0.52,
                ),
            ),
            (
                descendant_probe_id,
                HybridGiResolveRuntime::pack_rgb_and_weight(
                    [
                        irradiance_rgb[0] as f32 / 255.0,
                        irradiance_rgb[1] as f32 / 255.0,
                        irradiance_rgb[2] as f32 / 255.0,
                    ],
                    0.46,
                ),
            ),
        ]),
        probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    rt_lighting_rgb[0] as f32 / 255.0,
                    rt_lighting_rgb[1] as f32 / 255.0,
                    rt_lighting_rgb[2] as f32 / 255.0,
                ],
                0.0,
            ),
        )]),
        probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
            probe_id,
            descendant_probe_id,
        ]),
        probe_scene_driven_hierarchy_irradiance_quality_q8: std::collections::BTreeMap::from([
            (
                probe_id,
                HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
            ),
            (
                descendant_probe_id,
                HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
            ),
        ]),
        probe_scene_driven_hierarchy_rt_lighting_ids: std::collections::BTreeSet::new(),
        ..Default::default()
    }
}

fn runtime_with_scene_truth_flags_for_descendant_only(
    probe_id: u32,
    descendant_probe_id: u32,
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntime {
        probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
        )]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    irradiance_rgb[0] as f32 / 255.0,
                    irradiance_rgb[1] as f32 / 255.0,
                    irradiance_rgb[2] as f32 / 255.0,
                ],
                0.46,
            ),
        )]),
        probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
            descendant_probe_id,
        ]),
        probe_scene_driven_hierarchy_irradiance_quality_q8: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
        )]),
        probe_scene_driven_hierarchy_rt_lighting_ids: std::collections::BTreeSet::new(),
        ..Default::default()
    }
}

fn runtime_with_scene_truth_flags_for_descendant_rt_and_irradiance(
    probe_id: u32,
    descendant_probe_id: u32,
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntime {
        probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
            probe_id,
            HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
        )]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    irradiance_rgb[0] as f32 / 255.0,
                    irradiance_rgb[1] as f32 / 255.0,
                    irradiance_rgb[2] as f32 / 255.0,
                ],
                0.46,
            ),
        )]),
        probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    rt_lighting_rgb[0] as f32 / 255.0,
                    rt_lighting_rgb[1] as f32 / 255.0,
                    rt_lighting_rgb[2] as f32 / 255.0,
                ],
                0.44,
            ),
        )]),
        probe_scene_driven_hierarchy_irradiance_ids: std::collections::BTreeSet::from([
            descendant_probe_id,
        ]),
        probe_scene_driven_hierarchy_rt_lighting_ids: std::collections::BTreeSet::from([
            descendant_probe_id,
        ]),
        probe_scene_driven_hierarchy_irradiance_quality_q8: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
        )]),
        probe_scene_driven_hierarchy_rt_lighting_quality_q8: std::collections::BTreeMap::from([(
            descendant_probe_id,
            HybridGiResolveRuntime::pack_scene_truth_quality_q8(1.0),
        )]),
        ..Default::default()
    }
}

fn runtime_without_scene_truth_flags(
    rt_lighting_rgb: [u8; 3],
    irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntime {
        probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_resolve_weight_q8(0.9),
        )]),
        probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    irradiance_rgb[0] as f32 / 255.0,
                    irradiance_rgb[1] as f32 / 255.0,
                    irradiance_rgb[2] as f32 / 255.0,
                ],
                0.52,
            ),
        )]),
        probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight(
                [
                    rt_lighting_rgb[0] as f32 / 255.0,
                    rt_lighting_rgb[1] as f32 / 255.0,
                    rt_lighting_rgb[2] as f32 / 255.0,
                ],
                0.0,
            ),
        )]),
        ..Default::default()
    }
}

fn render_second_frame_with_and_without_history(
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    runtime: HybridGiResolveRuntime,
) -> (ViewportFrame, ViewportFrame) {
    let with_history = render_second_frame(
        extract.clone(),
        viewport_size,
        runtime.clone(),
        Some(crate::FrameHistoryHandle::new(16)),
    );
    let without_history = render_second_frame(extract, viewport_size, runtime, None);
    (with_history, without_history)
}

fn render_second_frame(
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    runtime: HybridGiResolveRuntime,
    history_handle: Option<crate::FrameHistoryHandle>,
) -> ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
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

    if let Some(history_handle) = history_handle {
        renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                    .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([240, 96, 48])))
                    .with_hybrid_gi_resolve_runtime(Some(runtime.clone())),
                &compiled,
                Some(history_handle),
            )
            .unwrap();

        return renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(extract, viewport_size)
                    .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([48, 96, 240])))
                    .with_hybrid_gi_resolve_runtime(Some(runtime)),
                &compiled,
                Some(history_handle),
            )
            .unwrap();
    }

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([48, 96, 240])))
                .with_hybrid_gi_resolve_runtime(Some(runtime)),
            &compiled,
            None,
        )
        .unwrap()
}

fn render_second_frame_after_probe_identity_transition_with_and_without_history(
    viewport_size: UVec2,
    first_extract: RenderFrameExtract,
    first_prepare: HybridGiPrepareFrame,
    first_runtime: HybridGiResolveRuntime,
    second_extract: RenderFrameExtract,
    second_prepare: HybridGiPrepareFrame,
    second_runtime: HybridGiResolveRuntime,
) -> (ViewportFrame, ViewportFrame) {
    let with_history = render_second_frame_after_probe_identity_transition(
        viewport_size,
        first_extract.clone(),
        first_prepare.clone(),
        first_runtime.clone(),
        second_extract.clone(),
        second_prepare.clone(),
        second_runtime.clone(),
        Some(crate::FrameHistoryHandle::new(24)),
    );
    let without_history = render_second_frame_after_probe_identity_transition(
        viewport_size,
        first_extract,
        first_prepare,
        first_runtime,
        second_extract,
        second_prepare,
        second_runtime,
        None,
    );
    (with_history, without_history)
}

fn render_second_frame_after_probe_identity_transition(
    viewport_size: UVec2,
    first_extract: RenderFrameExtract,
    first_prepare: HybridGiPrepareFrame,
    first_runtime: HybridGiResolveRuntime,
    second_extract: RenderFrameExtract,
    second_prepare: HybridGiPrepareFrame,
    second_runtime: HybridGiResolveRuntime,
    history_handle: Option<crate::FrameHistoryHandle>,
) -> ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &first_extract,
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

    if let Some(history_handle) = history_handle {
        renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(first_extract, viewport_size)
                    .with_hybrid_gi_prepare(Some(first_prepare))
                    .with_hybrid_gi_resolve_runtime(Some(first_runtime)),
                &compiled,
                Some(history_handle),
            )
            .unwrap();

        return renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(second_extract, viewport_size)
                    .with_hybrid_gi_prepare(Some(second_prepare))
                    .with_hybrid_gi_resolve_runtime(Some(second_runtime)),
                &compiled,
                Some(history_handle),
            )
            .unwrap();
    }

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(second_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(second_prepare))
                .with_hybrid_gi_resolve_runtime(Some(second_runtime)),
            &compiled,
            None,
        )
        .unwrap()
}

fn render_second_frame_after_scene_prepare_transition_with_and_without_history(
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    runtime: HybridGiResolveRuntime,
    first_scene_prepare: HybridGiScenePrepareFrame,
    second_scene_prepare: HybridGiScenePrepareFrame,
) -> (ViewportFrame, ViewportFrame) {
    let with_history = render_second_frame_after_scene_prepare_transition(
        extract.clone(),
        viewport_size,
        runtime.clone(),
        first_scene_prepare.clone(),
        second_scene_prepare.clone(),
        Some(crate::FrameHistoryHandle::new(20)),
    );
    let without_history = render_second_frame_after_scene_prepare_transition(
        extract,
        viewport_size,
        runtime,
        first_scene_prepare,
        second_scene_prepare,
        None,
    );
    (with_history, without_history)
}

fn render_second_frame_after_scene_prepare_transition(
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    runtime: HybridGiResolveRuntime,
    first_scene_prepare: HybridGiScenePrepareFrame,
    second_scene_prepare: HybridGiScenePrepareFrame,
    history_handle: Option<crate::FrameHistoryHandle>,
) -> ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
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

    if let Some(history_handle) = history_handle {
        renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                    .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([240, 96, 48])))
                    .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                    .with_hybrid_gi_scene_prepare(Some(first_scene_prepare)),
                &compiled,
                Some(history_handle),
            )
            .unwrap();

        return renderer
            .render_frame_with_pipeline(
                &ViewportRenderFrame::from_extract(extract, viewport_size)
                    .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([48, 96, 240])))
                    .with_hybrid_gi_resolve_runtime(Some(runtime))
                    .with_hybrid_gi_scene_prepare(Some(second_scene_prepare)),
                &compiled,
                Some(history_handle),
            )
            .unwrap();
    }

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([48, 96, 240])))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(second_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap()
}

fn render_second_frame_after_runtime_transition(
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    first_runtime: HybridGiResolveRuntime,
    second_runtime: HybridGiResolveRuntime,
) -> ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
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
    let history_handle = crate::FrameHistoryHandle::new(16);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([240, 96, 48])))
                .with_hybrid_gi_resolve_runtime(Some(first_runtime)),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(resident_prepare_with_irradiance([48, 96, 240])))
                .with_hybrid_gi_resolve_runtime(Some(second_runtime)),
            &compiled,
            Some(history_handle),
        )
        .unwrap()
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

fn centered_surface_cache_scene_prepare(
    capture_sample_rgba: [u8; 4],
    atlas_sample_rgba: [u8; 4],
) -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            atlas_sample_rgba,
            capture_sample_rgba,
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    }
}

fn runtime_from_surface_cache_scene_truth(clean: bool) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(0);
    let meshes = [mesh_at(
        11,
        "res://materials/history-surface-cache-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let lights = [directional_light(300, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    state.apply_scene_prepare_resources(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));
    if clean {
        state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
        assert!(
            state.scene_dirty_page_ids().is_empty(),
            "expected the clean surface-cache setup to clear dirty pages after the unchanged follow-up scene registration"
        );
    } else {
        assert_eq!(
            state.scene_dirty_page_ids(),
            vec![0],
            "expected the dirty surface-cache setup to keep the seeded resident page marked dirty before recapture"
        );
    }

    let runtime = state.build_resolve_runtime();
    assert!(
        runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected surface-cache fallback runtime to remain scene-driven for the tracked child probe"
    );
    runtime
}

fn runtime_from_surface_cache_scene_truth_clean_with_light_node_id(
    light_node_id: u64,
) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(0);
    let meshes = [mesh_at(
        11,
        "res://materials/history-surface-cache-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let lights = [directional_light(light_node_id, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    state.apply_scene_prepare_resources(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    assert!(
        state.scene_dirty_page_ids().is_empty(),
        "expected the stabilized surface-cache runtime to end clean after the repeated light registration"
    );
    state.build_resolve_runtime()
}

fn runtime_from_surface_cache_scene_truth_clean_after_light_node_id_transition(
    initial_light_node_id: u64,
    final_light_node_id: u64,
) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(0);
    let meshes = [mesh_at(
        11,
        "res://materials/history-surface-cache-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let initial_lights = [directional_light(initial_light_node_id, 1.0)];
    let final_lights = [directional_light(final_light_node_id, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &initial_lights, &[], &[]);
    state.apply_scene_prepare_resources(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));
    state.register_scene_extract(Some(&extract), &meshes, &final_lights, &[], &[]);
    assert_eq!(
        state.scene_dirty_page_ids(),
        vec![0],
        "expected the light-node-id transition to dirty the resident surface-cache page before the follow-up clean frame"
    );
    state.register_scene_extract(Some(&extract), &meshes, &final_lights, &[], &[]);
    assert!(
        state.scene_dirty_page_ids().is_empty(),
        "expected the follow-up light registration to return the transitioned surface-cache runtime to a clean state"
    );
    state.build_resolve_runtime()
}

fn runtime_from_voxel_scene_truth(clean: bool) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(1);
    let meshes = [mesh_at(
        11,
        "res://materials/history-voxel-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let lights = [directional_light(300, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    if clean {
        state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
        assert!(
            state.scene_dirty_clipmap_ids().is_empty(),
            "expected the clean voxel setup to clear dirty clipmaps after the unchanged follow-up scene registration"
        );
    } else {
        assert_eq!(
            state.scene_dirty_clipmap_ids(),
            vec![0],
            "expected the dirty voxel setup to keep the resident clipmap marked dirty on the first scene registration"
        );
    }

    let runtime = state.build_resolve_runtime();
    assert!(
        runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected voxel fallback runtime to remain scene-driven for the tracked child probe"
    );
    runtime
}

fn runtime_from_voxel_scene_truth_clean_with_light_node_id(
    light_node_id: u64,
) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(1);
    let meshes = [mesh_at(
        11,
        "res://materials/history-voxel-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let lights = [directional_light(light_node_id, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    state.register_scene_extract(Some(&extract), &meshes, &lights, &[], &[]);
    assert!(
        state.scene_dirty_clipmap_ids().is_empty(),
        "expected the stabilized voxel runtime to end clean after the repeated light registration"
    );
    state.build_resolve_runtime()
}

fn runtime_from_voxel_scene_truth_clean_after_light_node_id_transition(
    initial_light_node_id: u64,
    final_light_node_id: u64,
) -> HybridGiResolveRuntime {
    let extract = runtime_scene_truth_extract(1);
    let meshes = [mesh_at(
        11,
        "res://materials/history-voxel-freshness.mat",
        Vec3::ZERO,
        2.0,
    )];
    let initial_lights = [directional_light(initial_light_node_id, 1.0)];
    let final_lights = [directional_light(final_light_node_id, 1.0)];
    let mut state = HybridGiRuntimeState::default();
    state.register_scene_extract(Some(&extract), &meshes, &initial_lights, &[], &[]);
    state.register_scene_extract(Some(&extract), &meshes, &final_lights, &[], &[]);
    assert_eq!(
        state.scene_dirty_clipmap_ids(),
        vec![0],
        "expected the light-node-id transition to dirty the resident voxel clipmap before the follow-up clean frame"
    );
    state.register_scene_extract(Some(&extract), &meshes, &final_lights, &[], &[]);
    assert!(
        state.scene_dirty_clipmap_ids().is_empty(),
        "expected the follow-up light registration to return the transitioned voxel runtime to a clean state"
    );
    state.build_resolve_runtime()
}

fn runtime_scene_truth_extract(voxel_budget: u32) -> RenderHybridGiExtract {
    RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![
            probe(100, true, 96, Vec3::ZERO, 1.6),
            RenderHybridGiProbe {
                parent_probe_id: Some(100),
                ..probe(200, true, 88, Vec3::ZERO, 1.8)
            },
        ],
        trace_regions: Vec::new(),
    }
}

fn mesh_at(
    entity: u64,
    material: &str,
    translation: Vec3,
    uniform_scale: f32,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: entity,
        transform: Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale)),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/card.obj",
        )),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(material)),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}

fn directional_light(node_id: u64, intensity: f32) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id,
        direction: Vec3::new(-0.4, -1.0, -0.2),
        color: Vec3::new(1.0, 0.95, 0.9),
        intensity,
    }
}

fn scene_prepare_resources_snapshot(
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
) -> HybridGiScenePrepareResourcesSnapshot {
    let occupied_atlas_slots = atlas_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect::<Vec<_>>();
    let occupied_capture_slots = capture_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect::<Vec<_>>();
    HybridGiScenePrepareResourcesSnapshot {
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
        atlas_slot_count: 4,
        capture_slot_count: 4,
        atlas_texture_extent: (16, 16),
        capture_texture_extent: (16, 16),
        capture_layer_count: 1,
    }
}
