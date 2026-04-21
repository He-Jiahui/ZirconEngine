use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryInstance,
    RenderVirtualGeometryPage, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderWorldSnapshotHandle,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::scene::world::World;

use crate::{
    types::{
        ViewportRenderFrame, VirtualGeometryClusterSelection, VirtualGeometryPrepareCluster,
        VirtualGeometryPrepareClusterState, VirtualGeometryPrepareFrame,
        VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
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
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
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
    assert_eq!(readback.page_table_entries, vec![(200, 0), (300, 1)]);
    assert_eq!(readback.completed_page_ids, vec![300]);
    assert_eq!(readback.completed_page_assignments, vec![(300, 1)]);
}

#[test]
fn virtual_geometry_gpu_uploader_readback_merges_gpu_completed_assignments_into_page_table_snapshot(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 2, vec![page(200, true), page(300, false)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 2048,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 31,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: vec![5],
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.completed_page_ids, vec![300]);
    assert_eq!(readback.completed_page_assignments, vec![(300, 5)]);
    assert_eq!(
        readback.page_table_entries,
        vec![(200, 0), (300, 5)],
        "expected GPU uploader page-table snapshot to include newly completed page-slot assignments in the same readback"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_reports_actual_recycled_page_for_implicit_evictable_slot_reuse(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, false), page(400, true)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 400,
                        slot: 1,
                        size_bytes: 2048,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 200,
                        size_bytes: 2048,
                        generation: 32,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: vec![VirtualGeometryPreparePage {
                        page_id: 400,
                        slot: 1,
                        size_bytes: 2048,
                    }],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries, vec![(200, 1)]);
    assert_eq!(readback.completed_page_ids, vec![200]);
    assert_eq!(readback.completed_page_assignments, vec![(200, 1)]);
    assert_eq!(
        readback.completed_page_replacements,
        vec![(200, 400)],
        "expected GPU uploader to report the actual resident page recycled through an implicit evictable-slot reuse so runtime completion does not have to infer replacement truth only from page-table aliasing"
    );
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 2048,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 9,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
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
    assert_eq!(
        readback.completed_page_assignments,
        Vec::<(u32, u32)>::new()
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, true)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 2048,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 8192,
                        generation: 11,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 2048,
                    }],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries, vec![(200, 0)]);
    assert_eq!(
        readback.completed_page_ids,
        Vec::<u32>::new(),
        "expected uploader to reject oversized page requests when streaming bytes are insufficient"
    );
    assert_eq!(
        readback.completed_page_assignments,
        Vec::<(u32, u32)>::new()
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(400, false), page(500, false)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 400,
                            size_bytes: 8192,
                            generation: 12,
                            frontier_rank: 0,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 500,
                            size_bytes: 2048,
                            generation: 13,
                            frontier_rank: 1,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                    ],
                    available_slots: vec![0],
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries, vec![(500, 0)]);
    assert_eq!(
        readback.completed_page_ids,
        vec![500],
        "expected uploader to skip oversized requests and complete later requests that fit the streaming budget"
    );
    assert_eq!(readback.completed_page_assignments, vec![(500, 0)]);
}

#[test]
fn virtual_geometry_gpu_uploader_readback_assigns_free_slots_before_recycling_evictable_slots() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 3, vec![page(200, true), page(500, true)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 200,
                            slot: 4,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 500,
                            slot: 7,
                            size_bytes: 1024,
                        },
                    ],
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 300,
                            size_bytes: 2048,
                            generation: 21,
                            frontier_rank: 0,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 600,
                            size_bytes: 1024,
                            generation: 22,
                            frontier_rank: 1,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                    ],
                    available_slots: vec![2],
                    evictable_pages: vec![VirtualGeometryPreparePage {
                        page_id: 500,
                        slot: 7,
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
    assert_eq!(readback.completed_page_ids, vec![300, 600]);
    assert_eq!(
        readback.completed_page_assignments,
        vec![(300, 2), (600, 7)],
        "expected uploader to consume explicit available slots before recycling evictable resident slots"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_prioritizes_explicit_frontier_rank_over_input_order() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(300, false), page(600, false)]);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 600,
                            size_bytes: 2048,
                            generation: 41,
                            frontier_rank: 3,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 300,
                            size_bytes: 2048,
                            generation: 42,
                            frontier_rank: 0,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                    ],
                    available_slots: vec![0],
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(
        readback.completed_page_ids,
        vec![300],
        "expected GPU uploader completion to follow explicit frontier truth rather than raw pending request input order when only one slot is available"
    );
    assert_eq!(
        readback.completed_page_assignments,
        vec![(300, 0)],
        "expected page-table completion to keep the earliest frontier page resident even when a later-rank request appears first in the pending input buffer"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_uses_explicit_request_assigned_slot_over_evictable_input_order(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        3,
        vec![
            page(100, true),
            page(200, false),
            page(400, true),
            page(800, true),
        ],
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 100,
                            slot: 0,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 800,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 200,
                        size_bytes: 2048,
                        generation: 51,
                        frontier_rank: 0,
                        assigned_slot: Some(2),
                        recycled_page_id: Some(800),
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 800,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(
        readback.page_table_entries,
        vec![(100, 0), (400, 1), (200, 2)],
        "expected page-table completion to honor the request's explicit frontier-aware recycle-slot choice instead of consuming evictable slots in raw input order"
    );
    assert_eq!(readback.completed_page_ids, vec![200]);
    assert_eq!(
        readback.completed_page_assignments,
        vec![(200, 2)],
        "expected GPU uploader to keep the nearer descendant slot hot and recycle the explicit farther-frontier slot chosen by the request contract"
    );
    assert_eq!(
        readback.completed_page_replacements,
        vec![(200, 800)],
        "expected GPU readback to preserve the explicit frontier-aware recycled page truth instead of forcing runtime/page-table consumers to infer replacement only from slot aliasing"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_rejects_stale_explicit_recycle_slot_contract() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        vec![
            page(100, true),
            page(200, false),
            page(300, false),
            page(400, true),
            page(900, true),
        ],
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 100,
                            slot: 0,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 900,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 200,
                            size_bytes: 2048,
                            generation: 61,
                            frontier_rank: 0,
                            assigned_slot: Some(2),
                            recycled_page_id: Some(800),
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 300,
                            size_bytes: 2048,
                            generation: 62,
                            frontier_rank: 1,
                            assigned_slot: Some(1),
                            recycled_page_id: Some(400),
                        },
                    ],
                    available_slots: Vec::new(),
                    evictable_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 900,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(
        readback.page_table_entries,
        vec![(100, 0), (300, 1), (900, 2)],
        "expected GPU uploader to reject a stale explicit recycle-slot contract once the claimed recycled page no longer owns that slot, instead of silently evicting the current slot occupant"
    );
    assert_eq!(
        readback.completed_page_ids,
        vec![300],
        "expected uploader to skip the stale explicit recycle-slot request and continue with the next valid pending request under the same page budget"
    );
    assert_eq!(
        readback.completed_page_assignments,
        vec![(300, 1)],
        "expected page-table completion to preserve real slot ownership when explicit recycled-page truth disagrees with the current GPU page table"
    );
    assert_eq!(
        readback.completed_page_replacements,
        vec![(300, 400)],
        "expected replacement readback to keep matching the actual recycled resident page rather than echoing a stale request-side recycled-page id"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_preserves_frontier_recycle_preference_after_stale_requests_are_skipped(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        3,
        vec![
            page(100, true),
            page(200, false),
            page(300, false),
            page(400, true),
            page(500, false),
            page(800, true),
        ],
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 100,
                            slot: 0,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 800,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 200,
                            size_bytes: 2048,
                            generation: 71,
                            frontier_rank: 0,
                            assigned_slot: Some(2),
                            recycled_page_id: Some(900),
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 300,
                            size_bytes: 2048,
                            generation: 72,
                            frontier_rank: 1,
                            assigned_slot: Some(1),
                            recycled_page_id: Some(901),
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 500,
                            size_bytes: 2048,
                            generation: 73,
                            frontier_rank: 2,
                            assigned_slot: None,
                            recycled_page_id: Some(800),
                        },
                    ],
                    available_slots: Vec::new(),
                    evictable_pages: vec![
                        VirtualGeometryPreparePage {
                            page_id: 400,
                            slot: 1,
                            size_bytes: 2048,
                        },
                        VirtualGeometryPreparePage {
                            page_id: 800,
                            slot: 2,
                            size_bytes: 2048,
                        },
                    ],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(
        readback.page_table_entries,
        vec![(100, 0), (400, 1), (500, 2)],
        "expected uploader fallback submission to keep the late request on its frontier-preferred colder slot after earlier stale requests are skipped, instead of falling back to raw evictable-slot order and evicting the wrong lineage"
    );
    assert_eq!(
        readback.completed_page_ids,
        vec![500],
        "expected only the fallback request with a still-valid frontier recycle preference to complete after the earlier stale slot contracts are rejected"
    );
    assert_eq!(readback.completed_page_assignments, vec![(500, 2)]);
    assert_eq!(
        readback.completed_page_replacements,
        vec![(500, 800)],
        "expected replacement readback to preserve the preferred frontier recycle page chosen for the fallback request"
    );
}

#[test]
fn virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        world.to_render_snapshot(),
    );
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 0,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 0, Vec3::ZERO, 9.0),
            virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
        ],
        pages: vec![
            RenderVirtualGeometryPage {
                page_id: 200,
                resident: false,
                size_bytes: 4096,
            },
            RenderVirtualGeometryPage {
                page_id: 300,
                resident: true,
                size_bytes: 4096,
            },
        ],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("GpuReadbackVisBuffer64ContractMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    });
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: vec![mesh],
                    visible_clusters: vec![
                        VirtualGeometryPrepareCluster {
                            entity: mesh,
                            cluster_id: 20,
                            page_id: 200,
                            lod_level: 0,
                            resident_slot: None,
                            state: VirtualGeometryPrepareClusterState::Missing,
                        },
                        VirtualGeometryPrepareCluster {
                            entity: mesh,
                            cluster_id: 30,
                            page_id: 300,
                            lod_level: 0,
                            resident_slot: Some(0),
                            state: VirtualGeometryPrepareClusterState::Resident,
                        },
                    ],
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 300,
                        slot: 0,
                        size_bytes: 4096,
                    }],
                    pending_page_requests: Vec::new(),
                    available_slots: Vec::new(),
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let helper_visbuffer64 = renderer
        .read_last_virtual_geometry_gpu_readback_visbuffer64()
        .expect("expected non-consuming helper visbuffer64 readback");
    assert_eq!(
        helper_visbuffer64.0,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected the last-state helper to expose the same logical VisBuffer64 clear value without consuming the stored GPU readback"
    );
    assert_eq!(
        helper_visbuffer64.1,
        vec![RenderVirtualGeometryVisBuffer64Entry {
            entry_index: 0,
            packed_value: RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                30,
                300,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the non-consuming helper to mirror the same execution-backed logical VisBuffer64 entry stream as the stored GPU readback"
    );
    let readback = renderer
        .take_last_virtual_geometry_gpu_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(
        readback.visbuffer64_clear_value,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected GPU readback to expose the same stable logical VisBuffer64 clear value as the renderer-owned snapshot contract"
    );
    assert_eq!(
        readback.visbuffer64_entries,
        vec![RenderVirtualGeometryVisBuffer64Entry {
            entry_index: 0,
            packed_value: RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                30,
                300,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected GPU readback to publish the same execution-backed logical VisBuffer64 entry stream as the renderer-owned debug snapshot instead of only page-upload completion data"
    );
    let visbuffer64_words = renderer
        .read_last_virtual_geometry_visbuffer64_words()
        .expect("expected real VG visbuffer64 buffer words");
    assert_eq!(
        visbuffer64_words.0,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected the real VG visbuffer64 buffer helper to preserve the published clear value even after the CPU readback DTO has been consumed"
    );
    assert_eq!(
        visbuffer64_words.1,
        vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(0),
            30,
            300,
            0,
            RenderVirtualGeometryExecutionState::Resident,
        )],
        "expected the renderer to retain a real packed VisBuffer64 GPU buffer after readback consumption so later passes and inspection helpers do not depend on the temporary CPU readback object"
    );
}

#[test]
fn virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_cluster_selections(Some(vec![
                    VirtualGeometryClusterSelection {
                        submission_index: 0,
                        instance_index: Some(0),
                        entity: mesh,
                        cluster_id: 30,
                        cluster_ordinal: 1,
                        page_id: 300,
                        lod_level: 0,
                        submission_page_id: 300,
                        submission_lod_level: 0,
                        entity_cluster_start_ordinal: 1,
                        entity_cluster_span_count: 1,
                        entity_cluster_total_count: 2,
                        lineage_depth: 0,
                        frontier_rank: 0,
                        resident_slot: Some(0),
                        submission_slot: Some(0),
                        state: VirtualGeometryPrepareClusterState::Resident,
                    },
                ])),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        renderer.take_last_virtual_geometry_gpu_readback().is_none(),
        "expected the no-prepare path to skip VG uploader readback so the VisBuffer64 buffer cannot come from the CPU readback DTO"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_visbuffer64()
            .is_none(),
        "expected the no-prepare path to have no uploader readback helper output"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the renderer-owned VisBuffer64 buffer to report render-path execution ownership instead of an opaque late fallback source"
    );
    let visbuffer64_words = renderer
        .read_last_virtual_geometry_visbuffer64_words()
        .expect("expected execution-owned visbuffer64 buffer words");
    assert_eq!(
        visbuffer64_words.0,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected the execution-owned visbuffer64 buffer to preserve the published clear value even when there is no uploader readback or renderer-owned snapshot"
    );
    assert_eq!(
        visbuffer64_words.1,
        vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(0),
            30,
            300,
            0,
            RenderVirtualGeometryExecutionState::Resident,
        )],
        "expected frame-owned ClusterSelection truth to be sufficient for producing a real execution-owned VisBuffer64 buffer without depending on uploader readback or snapshot backfill"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the renderer-owned hardware-rasterization startup buffer to report explicit render-path execution ownership instead of relying on snapshot-only reconstruction"
    );
    let hardware_rasterization_records = renderer
        .read_last_virtual_geometry_hardware_rasterization_records()
        .expect("expected execution-owned hardware-rasterization records");
    assert_eq!(
        hardware_rasterization_records,
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            submission_index: 0,
            submission_page_id: 300,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: 1,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: Some(0),
            submission_slot: Some(0),
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected frame-owned ClusterSelection truth to be sufficient for producing a real execution-owned hardware-rasterization startup buffer without depending on snapshot reconstruction"
    );
    let selected_clusters = renderer
        .read_last_virtual_geometry_selected_clusters()
        .expect("expected execution-owned selected-cluster records");
    assert_eq!(
        selected_clusters,
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected frame-owned ClusterSelection truth to be retained as a real execution-owned selected-cluster buffer so later Nanite-style passes and inspection helpers do not need to reconstruct cluster identity from visbuffer or raster startup side channels"
    );
}

#[test]
fn virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 0, Vec::new());
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
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        renderer.take_last_virtual_geometry_gpu_readback().is_none(),
        "expected the no-prepare clear-only path to skip VG uploader readback"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no cluster writes to report an explicit clear-only VisBuffer64 render-path pass instead of collapsing that state into Unavailable"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no executed cluster selections to report an explicit clear-only hardware-rasterization pass instead of collapsing that state into Unavailable"
    );
    let visbuffer64_words = renderer
        .read_last_virtual_geometry_visbuffer64_words()
        .expect("expected VisBuffer64 clear-only readback to succeed");
    assert_eq!(
        visbuffer64_words.0,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected the clear-only path to preserve the same published VisBuffer64 clear value as frames that also emit cluster entries"
    );
    assert!(
        visbuffer64_words.1.is_empty(),
        "expected the clear-only path to keep the packed word stream empty when no cluster writes were emitted"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected hardware-rasterization clear-only readback to succeed")
            .is_empty(),
        "expected the clear-only path to keep hardware-rasterization startup records empty when no executed cluster selections were emitted"
    );
}

fn build_extract(
    viewport_size: UVec2,
    page_budget: u32,
    pages: Vec<RenderVirtualGeometryPage>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.directional_lights.clear();
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 0,
        page_budget,
        clusters: Vec::new(),
        pages,
        instances: Vec::new(),
        debug: Default::default(),
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

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}
