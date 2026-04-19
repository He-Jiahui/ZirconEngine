use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::scene::world::World;

use crate::{
    types::{
        EditorOrRuntimeFrame, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
