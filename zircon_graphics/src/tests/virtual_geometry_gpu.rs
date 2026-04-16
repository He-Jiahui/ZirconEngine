use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_math::UVec2;
use zircon_scene::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, World,
};

use crate::{
    types::{
        EditorOrRuntimeFrame, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
    ViewportState,
};

#[test]
fn virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        2,
        vec![page(200, true), page(500, true), page(300, false)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 200,
                            slot: 0,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 500,
                            slot: 1,
                            size_bytes: 1024,
                        },
                    ],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 7,
                    }],
                    evictable_pages: vec![VirtualGeometryPreparePage {
                        page_id: 500,
                        slot: 1,
                        size_bytes: 1024,
                    }],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries, vec![(200, 0), (500, 1)]);
    assert_eq!(readback.completed_page_ids, vec![300]);
}

#[test]
fn virtual_geometry_gpu_uploader_readback_respects_budget_without_evictable_pages() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, true), page(300, false)]);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_async_compute(false),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 2048,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 9,
                    }],
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries, vec![(200, 0)]);
    assert_eq!(readback.completed_page_ids, Vec::<u32>::new());
}

fn build_extract(
    viewport_size: UVec2,
    page_budget: u32,
    pages: Vec<RenderVirtualGeometryPage>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.lights.clear();
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 0,
        page_budget,
        clusters: Vec::new(),
        pages,
    });
    extract
}

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: if resident { 2048 } else { 4096 },
    }
}
