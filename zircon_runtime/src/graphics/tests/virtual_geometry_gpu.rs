use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryCluster,
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPage,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderWorldSnapshotHandle,
};
use crate::core::math::{view_matrix, Mat4, Transform, UVec2, Vec3};
use crate::graphics::tests::plugin_render_feature_fixtures::virtual_geometry_render_feature_descriptor;
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use crate::scene::world::World;

use crate::{
    types::{
        ViewportRenderFrame, VirtualGeometryClusterSelection, VirtualGeometryPrepareCluster,
        VirtualGeometryPrepareClusterState, VirtualGeometryPrepareFrame,
        VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
    },
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn compile_virtual_geometry_pipeline(extract: &RenderFrameExtract) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([virtual_geometry_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
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
        .unwrap()
}

fn virtual_geometry_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        2,
        vec![page(200, true), page(500, true), page(300, false)],
    );

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.page_table_entries(), vec![(200, 0), (300, 1)]);
    assert_eq!(readback.completed_page_ids(), vec![300]);
    assert_eq!(readback.completed_page_assignments(), vec![(300, 1)]);
}

#[test]
fn virtual_geometry_gpu_uploader_readback_merges_gpu_completed_assignments_into_page_table_snapshot(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 2, vec![page(200, true), page(300, false)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.completed_page_ids(), vec![300]);
    assert_eq!(readback.completed_page_assignments(), vec![(300, 5)]);
    assert_eq!(
        readback.page_table_entries(),
        vec![(200, 0), (300, 5)],
        "expected GPU uploader page-table snapshot to include newly completed page-slot assignments in the same readback"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_reports_actual_recycled_page_for_implicit_evictable_slot_reuse(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, false), page(400, true)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.page_table_entries(), vec![(200, 1)]);
    assert_eq!(readback.completed_page_ids(), vec![200]);
    assert_eq!(readback.completed_page_assignments(), vec![(200, 1)]);
    assert_eq!(
        readback.completed_page_replacements(),
        vec![(200, 400)],
        "expected GPU uploader to report the actual resident page recycled through an implicit evictable-slot reuse so runtime completion does not have to infer replacement truth only from page-table aliasing"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_respects_budget_without_evictable_pages() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, true), page(300, false)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.page_table_entries(), vec![(200, 0)]);
    assert_eq!(readback.completed_page_ids(), Vec::<u32>::new());
    assert_eq!(
        readback.completed_page_assignments(),
        Vec::<(u32, u32)>::new()
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, true)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.page_table_entries(), vec![(200, 0)]);
    assert_eq!(
        readback.completed_page_ids(),
        Vec::<u32>::new(),
        "expected uploader to reject oversized page requests when streaming bytes are insufficient"
    );
    assert_eq!(
        readback.completed_page_assignments(),
        Vec::<(u32, u32)>::new()
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(400, false), page(500, false)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.page_table_entries(), vec![(500, 0)]);
    assert_eq!(
        readback.completed_page_ids(),
        vec![500],
        "expected uploader to skip oversized requests and complete later requests that fit the streaming budget"
    );
    assert_eq!(readback.completed_page_assignments(), vec![(500, 0)]);
}

#[test]
fn virtual_geometry_gpu_uploader_readback_assigns_free_slots_before_recycling_evictable_slots() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 3, vec![page(200, true), page(500, true)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
    assert_eq!(readback.completed_page_ids(), vec![300, 600]);
    assert_eq!(
        readback.completed_page_assignments(),
        vec![(300, 2), (600, 7)],
        "expected uploader to consume explicit available slots before recycling evictable resident slots"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_prioritizes_explicit_frontier_rank_over_input_order() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(300, false), page(600, false)]);

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
        readback.completed_page_ids(),
        vec![300],
        "expected GPU uploader completion to follow explicit frontier truth rather than raw pending request input order when only one slot is available"
    );
    assert_eq!(
        readback.completed_page_assignments(),
        vec![(300, 0)],
        "expected page-table completion to keep the earliest frontier page resident even when a later-rank request appears first in the pending input buffer"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_uses_explicit_request_assigned_slot_over_evictable_input_order(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
        readback.page_table_entries(),
        vec![(100, 0), (400, 1), (200, 2)],
        "expected page-table completion to honor the request's explicit frontier-aware recycle-slot choice instead of consuming evictable slots in raw input order"
    );
    assert_eq!(readback.completed_page_ids(), vec![200]);
    assert_eq!(
        readback.completed_page_assignments(),
        vec![(200, 2)],
        "expected GPU uploader to keep the nearer descendant slot hot and recycle the explicit farther-frontier slot chosen by the request contract"
    );
    assert_eq!(
        readback.completed_page_replacements(),
        vec![(200, 800)],
        "expected GPU readback to preserve the explicit frontier-aware recycled page truth instead of forcing runtime/page-table consumers to infer replacement only from slot aliasing"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_rejects_stale_explicit_recycle_slot_contract() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
        readback.page_table_entries(),
        vec![(100, 0), (300, 1), (900, 2)],
        "expected GPU uploader to reject a stale explicit recycle-slot contract once the claimed recycled page no longer owns that slot, instead of silently evicting the current slot occupant"
    );
    assert_eq!(
        readback.completed_page_ids(),
        vec![300],
        "expected uploader to skip the stale explicit recycle-slot request and continue with the next valid pending request under the same page budget"
    );
    assert_eq!(
        readback.completed_page_assignments(),
        vec![(300, 1)],
        "expected page-table completion to preserve real slot ownership when explicit recycled-page truth disagrees with the current GPU page table"
    );
    assert_eq!(
        readback.completed_page_replacements(),
        vec![(300, 400)],
        "expected replacement readback to keep matching the actual recycled resident page rather than echoing a stale request-side recycled-page id"
    );
}

#[test]
fn virtual_geometry_gpu_uploader_readback_preserves_frontier_recycle_preference_after_stale_requests_are_skipped(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
        readback.page_table_entries(),
        vec![(100, 0), (400, 1), (500, 2)],
        "expected uploader fallback submission to keep the late request on its frontier-preferred colder slot after earlier stale requests are skipped, instead of falling back to raw evictable-slot order and evicting the wrong lineage"
    );
    assert_eq!(
        readback.completed_page_ids(),
        vec![500],
        "expected only the fallback request with a still-valid frontier recycle preference to complete after the earlier stale slot contracts are rejected"
    );
    assert_eq!(readback.completed_page_assignments(), vec![(500, 2)]);
    assert_eq!(
        readback.completed_page_replacements(),
        vec![(500, 800)],
        "expected replacement readback to preserve the preferred frontier recycle page chosen for the fallback request"
    );
}

#[test]
fn virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
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

    let compiled = compile_virtual_geometry_pipeline(&extract);

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

    assert_eq!(
        renderer.read_last_virtual_geometry_cluster_selection_input_source(),
        RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        "expected a direct prepare-backed frame without stored cluster selections to record on-demand prepare provenance so later Nanite debugging can distinguish this path from mirrored runtime-frame ownership"
    );
    let expected_selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
        instance_index: Some(0),
        entity: mesh,
        cluster_id: 30,
        cluster_ordinal: 1,
        page_id: 300,
        lod_level: 0,
        state: RenderVirtualGeometryExecutionState::Resident,
    }];
    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected prepare-owned ClusterSelection truth to drive the renderer-owned selected-cluster buffer directly instead of leaving the render path stuck on a clear-only fallback"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_selected_clusters()
            .expect("expected renderer-owned selected-cluster readback"),
        expected_selected_clusters,
        "expected prepare-owned ClusterSelection truth to populate the renderer-owned selected-cluster buffer without requiring a frame-owned override"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected prepare-owned ClusterSelection truth to drive the renderer-owned hardware-rasterization startup buffer directly instead of leaving the render path stuck on a clear-only fallback"
    );
    assert_eq!(
        renderer.last_virtual_geometry_hardware_rasterization_record_count(),
        1,
        "expected prepare-owned ClusterSelection truth to produce one hardware-rasterization startup record on the renderer-owned pass path"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_hardware_rasterization_records()
            .expect("expected renderer-owned hardware-rasterization readback"),
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
        "expected prepare-owned ClusterSelection truth to populate the renderer-owned hardware-rasterization startup records without waiting for CPU readback reconstruction"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected prepare-owned ClusterSelection truth to drive the renderer-owned VisBuffer64 pass directly instead of leaving the render path stuck on a clear-only fallback"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_visbuffer64_words()
            .expect("expected renderer-owned VisBuffer64 readback"),
        (
            RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                30,
                300,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            )]
        ),
        "expected prepare-owned ClusterSelection truth to populate the renderer-owned VisBuffer64 words without requiring a frame-owned override"
    );
    let helper_visbuffer64 = renderer
        .read_last_virtual_geometry_gpu_readback_visbuffer64()
        .expect("expected non-consuming helper visbuffer64 readback");
    let helper_visbuffer64_source = renderer
        .read_last_virtual_geometry_gpu_readback_visbuffer64_source()
        .expect("expected non-consuming visbuffer64 source readback");
    let helper_visbuffer64_entry_count = renderer
        .read_last_virtual_geometry_gpu_readback_visbuffer64_entry_count()
        .expect("expected non-consuming visbuffer64 entry-count readback");
    let helper_hardware_rasterization_source = renderer
        .read_last_virtual_geometry_gpu_readback_hardware_rasterization_source()
        .expect("expected non-consuming hardware-rasterization source readback");
    let helper_hardware_rasterization_record_count = renderer
        .read_last_virtual_geometry_gpu_readback_hardware_rasterization_record_count()
        .expect("expected non-consuming hardware-rasterization record-count readback");
    let helper_selected_clusters = renderer
        .read_last_virtual_geometry_gpu_readback_selected_clusters()
        .expect("expected non-consuming selected-cluster readback");
    let helper_selected_cluster_source = renderer
        .read_last_virtual_geometry_gpu_readback_selected_cluster_source()
        .expect("expected non-consuming selected-cluster source readback");
    let helper_selected_cluster_count = renderer
        .read_last_virtual_geometry_gpu_readback_selected_cluster_count()
        .expect("expected non-consuming selected-cluster count readback");
    assert_eq!(
        helper_hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the uploader readback helper to preserve the actual hardware-rasterization render-path provenance once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        helper_hardware_rasterization_record_count, 1,
        "expected the uploader readback helper to preserve the actual hardware-rasterization startup record count once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        helper_selected_cluster_source,
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the uploader readback helper to preserve the actual selected-cluster render-path provenance once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        helper_selected_cluster_count, 1,
        "expected the uploader readback helper to preserve the render-path selected-cluster count once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        helper_selected_clusters,
        expected_selected_clusters,
        "expected the non-consuming GPU readback helper to mirror the same execution-backed selected-cluster subset as the stored uploader readback"
    );
    assert_eq!(
        helper_visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the uploader readback helper to preserve the actual VisBuffer64 render-path provenance once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        helper_visbuffer64_entry_count, 1,
        "expected the uploader readback helper to preserve the render-path VisBuffer64 entry count once prepare-owned ClusterSelection truth reaches the render path directly"
    );
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
        renderer.read_last_virtual_geometry_hardware_rasterization_source(),
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the renderer-owned hardware-rasterization source snapshot to keep the same execution-owned provenance after the uploader DTO is consumed"
    );
    assert_eq!(
        renderer.last_virtual_geometry_hardware_rasterization_record_count(), 1,
        "expected the renderer-owned hardware-rasterization record count to preserve the same execution-owned startup record count after the uploader DTO is consumed"
    );
    assert_eq!(
        readback.visbuffer64_source(),
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the stored uploader DTO to preserve the real VisBuffer64 render-path provenance once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        readback.visbuffer64_entry_count(), 1,
        "expected the stored uploader DTO to preserve the real render-path VisBuffer64 entry count once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        readback.visbuffer64_clear_value(),
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected GPU readback to expose the same stable logical VisBuffer64 clear value as the renderer-owned snapshot contract"
    );
    assert_eq!(
        readback.visbuffer64_entries(),
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
    assert_eq!(
        readback.selected_clusters(),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected GPU readback to publish the same execution-backed selected-cluster subset as the renderer-owned snapshot instead of exposing only uploader completion data"
    );
    assert_eq!(
        readback.selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the stored uploader DTO to preserve the real selected-cluster render-path provenance once prepare-owned ClusterSelection truth reaches the render path directly"
    );
    assert_eq!(
        readback.selected_cluster_count(), 1,
        "expected the stored uploader DTO to preserve the real render-path selected-cluster count once prepare-owned ClusterSelection truth reaches the render path directly"
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
fn virtual_geometry_cull_input_buffer_exists_without_snapshot_or_gpu_readback() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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
    let expected_camera_transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.view.camera.transform = expected_camera_transform;
    let mut render_clusters = vec![
        RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(71),
            ..virtual_geometry_cluster(mesh, 20, 200, 1, Vec3::ZERO, 8.0)
        },
        RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(72),
            ..virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 6.0)
        },
    ];
    render_clusters.resize(91, RenderVirtualGeometryCluster::default());
    for cluster_array_index in [70, 71, 72, 90] {
        render_clusters[cluster_array_index] = RenderVirtualGeometryCluster {
            entity: mesh,
            cluster_id: u32::try_from(cluster_array_index).unwrap_or(u32::MAX),
            page_id: 200,
            lod_level: 0,
            bounds_center: Vec3::new(3.0, 4.0, 0.0),
            bounds_radius: 0.5,
            ..RenderVirtualGeometryCluster::default()
        };
    }
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: render_clusters,
        hierarchy_nodes: vec![
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 72,
                child_base: 0,
                child_count: 2,
                cluster_start: 1,
                cluster_count: 1,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 7,
                child_base: 0,
                child_count: 0,
                cluster_start: 70,
                cluster_count: 3,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 42,
                child_base: 0,
                child_count: 0,
                cluster_start: 90,
                cluster_count: 1,
            },
        ],
        hierarchy_child_ids: vec![7, 42],
        pages: vec![page(200, true), page(300, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("CullInputBufferUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(0),
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });

    let compiled = compile_virtual_geometry_pipeline(&extract);

    let expected_fov_y = extract.view.camera.fov_y_radians;
    let expected_z_near = extract.view.camera.z_near;
    let expected_z_far = extract.view.camera.z_far;
    let expected_camera_translation = expected_camera_transform.translation.to_array();
    let expected_view_proj = Mat4::perspective_rh(
        expected_fov_y,
        viewport_size.x as f32 / viewport_size.y as f32,
        expected_z_near,
        expected_z_far,
    )
    .mul_mat4(&view_matrix(expected_camera_transform))
    .to_cols_array_2d();

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
                        resident_slot: None,
                        submission_slot: Some(0),
                        state: VirtualGeometryPrepareClusterState::PendingUpload,
                    },
                ])),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.read_last_virtual_geometry_cluster_selection_input_source(),
        RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
        "expected the renderer-owned authority seam to preserve explicit frame-owned ClusterSelection provenance for the cull-input bridge as well"
    );
    assert!(
        renderer.take_last_virtual_geometry_gpu_readback().is_none(),
        "expected the no-prepare path to skip VG uploader readback so the cull-input buffer cannot be sourced from uploader DTO state"
    );

    let cull_input = renderer
        .read_last_virtual_geometry_cull_input_snapshot()
        .expect("expected cull-input buffer readback to succeed")
        .expect("expected a real renderer-owned cull-input buffer");
    assert_eq!(
        cull_input,
        RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 1,
            instance_count: 1,
            cluster_count: 91,
            page_count: 2,
            visible_entity_count: 1,
            visible_cluster_count: 91,
            resident_page_count: 0,
            pending_page_request_count: 0,
            available_page_slot_count: 0,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(0),
                freeze_cull: true,
                visualize_bvh: true,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
        },
        "expected the new renderer-owned cull-input buffer to survive without snapshot/readback backfill and decode to the same authored VG budgets/debug/provenance surface that later NodeAndClusterCull/NaniteGlobalStateBuffer work will consume"
    );
    assert_eq!(
        renderer.read_last_virtual_geometry_node_and_cluster_cull_source(),
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected the first explicit NodeAndClusterCull startup bridge to report render-path cull-input ownership instead of leaving the new cull-input buffer without a consumer-side pass seam"
    );
    assert_eq!(
        renderer.last_virtual_geometry_node_and_cluster_cull_record_count(),
        1,
        "expected the first explicit NodeAndClusterCull startup bridge to publish one startup record when a VG cull-input snapshot exists"
    );
    let node_and_cluster_cull_input = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_input_snapshot()
        .expect("expected node-and-cluster-cull startup buffer readback to succeed")
        .expect("expected a real renderer-owned node-and-cluster-cull startup buffer");
    assert_eq!(
        node_and_cluster_cull_input,
        cull_input,
        "expected the first NodeAndClusterCull startup seam to preserve the exact packed cull-input DTO so a later NaniteGlobalStateBuffer / GPU hierarchy traversal can swap producers without changing host-side inspection contracts"
    );
    let node_and_cluster_cull_global_state = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_global_state_snapshot()
        .expect("expected node-and-cluster-cull global-state readback to succeed")
        .expect("expected a real renderer-owned node-and-cluster-cull global-state buffer");
    assert_eq!(
        node_and_cluster_cull_global_state,
        RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
            cull_input,
            viewport_size: [viewport_size.x, viewport_size.y],
            camera_translation: expected_camera_translation,
            child_split_screen_space_error_threshold: 1.0,
            child_frustum_culling_enabled: true,
            view_proj: expected_view_proj,
            previous_camera_translation: expected_camera_translation,
            previous_view_proj: expected_view_proj,
        },
        "expected the upgraded NodeAndClusterCull startup seam to preserve viewport size, camera origin, and the same view-projection matrix shape the renderer already uses for scene-uniform setup"
    );
    let node_and_cluster_cull_instance_seeds = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_instance_seeds()
        .expect("expected node-and-cluster-cull instance-seed readback to succeed");
    assert_eq!(
        node_and_cluster_cull_instance_seeds,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 0,
            entity: mesh,
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
        }],
        "expected the next NodeAndClusterCull bridge step to consume the typed startup/global-state seam into one per-instance root seed worklist row so later GPU BVH traversal can swap in real VisitNode traversal without changing the renderer-owned seed contract"
    );
    let node_and_cluster_cull_dispatch_setup = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_dispatch_setup_snapshot()
        .expect("expected node-and-cluster-cull dispatch-setup readback to succeed")
        .expect("expected a real renderer-owned node-and-cluster-cull dispatch-setup buffer");
    assert_eq!(
        node_and_cluster_cull_dispatch_setup,
        RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
            instance_seed_count: 1,
            cluster_budget: 1,
            page_budget: 1,
            workgroup_size: 64,
            dispatch_group_count: [1, 1, 1],
        },
        "expected the first explicit NodeAndClusterCull dispatch/setup seam to consume the typed global-state budget inputs and derived root-seed count into a dedicated renderer-owned startup record before any real compute traversal exists"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_launch_worklist_snapshot()
            .expect("expected node-and-cluster-cull launch-worklist readback to succeed")
            .expect("expected a real renderer-owned node-and-cluster-cull launch-worklist buffer"),
        RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
            global_state: node_and_cluster_cull_global_state,
            dispatch_setup: node_and_cluster_cull_dispatch_setup,
            instance_seeds: node_and_cluster_cull_instance_seeds,
        },
        "expected the renderer-owned NodeAndClusterCull launch-worklist buffer to preserve the combined global-state, dispatch-setup, and root-seed contract so later baseline compute or real traversal can bind one authoritative startup package"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_instance_work_items()
            .expect("expected node-and-cluster-cull instance-work-item readback to succeed"),
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
            instance_index: 0,
            entity: mesh,
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            cluster_budget: 1,
            page_budget: 1,
            forced_mip: Some(0),
        }],
        "expected the renderer-owned NodeAndClusterCull compute-stub buffer to preserve typed per-instance work items so later baseline execution and GPU traversal can share the same authoritative seam"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_cluster_work_items()
            .expect("expected node-and-cluster-cull cluster-work-item readback to succeed"),
        vec![
            VirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 0,
                hierarchy_node_id: Some(71),
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 1,
                hierarchy_node_id: Some(72),
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
        ],
        "expected the renderer-owned NodeAndClusterCull seam below instance work items to preserve typed per-cluster queue rows so the next VisitNode or baseline traversal stage can bind a real cluster worklist instead of rescanning broad instance slices"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_hierarchy_child_ids()
            .expect("expected node-and-cluster-cull hierarchy-child-id readback to succeed"),
        vec![7, 42],
        "expected the renderer-owned NodeAndClusterCull child-id table to preserve non-contiguous authored child nodes beside traversal records instead of forcing later consumers to reinterpret child_base/count as node ids"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_child_work_items()
            .expect("expected node-and-cluster-cull child-work-item readback to succeed"),
        vec![
            VirtualGeometryNodeAndClusterCullChildWorkItem {
                instance_index: 0,
                entity: mesh,
                parent_cluster_array_index: 1,
                parent_hierarchy_node_id: Some(72),
                child_node_id: 7,
                child_table_index: 0,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullChildWorkItem {
                instance_index: 0,
                entity: mesh,
                parent_cluster_array_index: 1,
                parent_hierarchy_node_id: Some(72),
                child_node_id: 42,
                child_table_index: 1,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
        ],
        "expected the renderer-owned NodeAndClusterCull child worklist to expand authored child-id table ranges into persistent child-node rows instead of leaving EnqueueChild as an opaque traversal marker"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_traversal_records()
            .expect("expected node-and-cluster-cull traversal-record readback to succeed"),
        vec![
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 0,
                hierarchy_node_id: Some(71),
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 0,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 0,
                hierarchy_node_id: Some(71),
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 1,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 1,
                hierarchy_node_id: Some(72),
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 2,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                child_source:
                    VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 1,
                hierarchy_node_id: Some(72),
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 2,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 1,
                hierarchy_node_id: Some(7),
                node_cluster_start: 70,
                node_cluster_count: 3,
                child_base: 0,
                child_count: 0,
                traversal_index: 4,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 1,
                hierarchy_node_id: Some(42),
                node_cluster_start: 90,
                node_cluster_count: 1,
                child_base: 0,
                child_count: 0,
                traversal_index: 5,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 70,
                hierarchy_node_id: Some(7),
                node_cluster_start: 70,
                node_cluster_count: 3,
                child_base: 0,
                child_count: 0,
                traversal_index: 6,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: mesh,
                cluster_array_index: 90,
                hierarchy_node_id: Some(42),
                node_cluster_start: 90,
                node_cluster_count: 1,
                child_base: 0,
                child_count: 0,
                traversal_index: 7,
                cluster_budget: 1,
                page_budget: 1,
                forced_mip: Some(0),
            },
        ],
        "expected the renderer-owned NodeAndClusterCull traversal seam to publish parent VisitNode/StoreCluster/EnqueueChild rows plus budget-limited child StoreCluster rows before the real hierarchy cull kernel lands"
    );
}

#[test]
fn virtual_geometry_node_and_cluster_cull_reuses_previous_renderer_owned_global_state_without_debug_snapshot(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let first_camera_transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    let second_camera_transform = Transform::from_translation(Vec3::new(-2.0, 6.0, 8.0));
    let first_extract =
        build_node_and_cluster_cull_history_extract(viewport_size, first_camera_transform);
    let second_extract =
        build_node_and_cluster_cull_history_extract(viewport_size, second_camera_transform);
    let compiled = compile_virtual_geometry_pipeline(&first_extract);
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(first_extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let first_global_state = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_global_state_snapshot()
        .expect("expected first global-state readback to succeed")
        .expect("expected first global-state buffer");

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(second_extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let second_global_state = renderer
        .read_last_virtual_geometry_node_and_cluster_cull_global_state_snapshot()
        .expect("expected second global-state readback to succeed")
        .expect("expected second global-state buffer");

    assert_eq!(
        second_global_state.previous_camera_translation,
        first_global_state.camera_translation,
        "expected the second no-debug-snapshot render path to reuse the renderer-owned NodeAndClusterCull global-state history instead of falling back to the second frame camera"
    );
    assert_eq!(
        second_global_state.previous_view_proj,
        first_global_state.view_proj,
        "expected renderer-owned NodeAndClusterCull history to preserve the previous frame view-projection when no frame debug snapshot was stored"
    );
    assert_ne!(
        second_global_state.previous_camera_translation,
        second_global_state.camera_translation
    );
}

#[test]
fn virtual_geometry_node_and_cluster_cull_page_requests_are_readable_from_last_state() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    let mut render_clusters = vec![
        RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(71),
            ..virtual_geometry_cluster(mesh, 20, 200, 0, Vec3::ZERO, 8.0)
        },
        RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(72),
            ..virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 6.0)
        },
    ];
    render_clusters.resize(72, RenderVirtualGeometryCluster::default());
    render_clusters[70] = RenderVirtualGeometryCluster {
        entity: mesh,
        cluster_id: 70,
        page_id: 200,
        lod_level: 0,
        bounds_center: Vec3::new(3.0, 4.0, 0.0),
        bounds_radius: 0.5,
        ..RenderVirtualGeometryCluster::default()
    };
    render_clusters[71] = RenderVirtualGeometryCluster {
        entity: mesh,
        cluster_id: 71,
        page_id: 300,
        lod_level: 0,
        bounds_center: Vec3::new(3.0, 4.0, 0.0),
        bounds_radius: 0.5,
        ..RenderVirtualGeometryCluster::default()
    };
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: render_clusters,
        hierarchy_nodes: vec![
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 72,
                child_base: 0,
                child_count: 1,
                cluster_start: 1,
                cluster_count: 1,
            },
            RenderVirtualGeometryHierarchyNode {
                instance_index: 0,
                node_id: 7,
                child_base: 0,
                child_count: 0,
                cluster_start: 70,
                cluster_count: 2,
            },
        ],
        hierarchy_child_ids: vec![7],
        pages: vec![page(200, true), page(300, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("NodeAndClusterCullPageRequestUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(0),
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
    });

    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size),
            &compiled,
            None,
        )
        .unwrap();

    assert_eq!(
        renderer.last_virtual_geometry_node_and_cluster_cull_page_request_count(),
        1,
        "expected renderer last-state to preserve the NodeAndClusterCull page request count for runtime upload feedback"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_page_requests()
            .expect("expected node-and-cluster-cull page-request readback to succeed"),
        vec![300],
        "expected renderer-owned NodeAndClusterCull page-request buffer to publish the nonresident child page id for the next runtime upload-authority integration"
    );
}

#[test]
fn virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
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
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));

    let compiled = compile_virtual_geometry_pipeline(&extract);

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

    assert_eq!(
        renderer.read_last_virtual_geometry_cluster_selection_input_source(),
        RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned,
        "expected a frame that injects cluster selections directly to preserve explicit override provenance on the renderer-owned last-state surface"
    );
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
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_visbuffer64_source()
            .is_none(),
        "expected the no-prepare path to have no uploader VisBuffer64 source helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_visbuffer64_entry_count()
            .is_none(),
        "expected the no-prepare path to have no uploader VisBuffer64 entry-count helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_hardware_rasterization_source()
            .is_none(),
        "expected the no-prepare path to have no uploader hardware-rasterization source helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_hardware_rasterization_record_count()
            .is_none(),
        "expected the no-prepare path to have no uploader hardware-rasterization record-count helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_selected_clusters()
            .is_none(),
        "expected the no-prepare path to have no uploader selected-cluster readback helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_selected_cluster_source()
            .is_none(),
        "expected the no-prepare path to have no uploader selected-cluster source helper output"
    );
    assert!(
        renderer
            .read_last_virtual_geometry_gpu_readback_selected_cluster_count()
            .is_none(),
        "expected the no-prepare path to have no uploader selected-cluster count helper output"
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
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the renderer-owned selected-cluster buffer to report explicit render-path execution ownership instead of relying on snapshot-only reconstruction"
    );
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
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 0, Vec::new());

    let compiled = compile_virtual_geometry_pipeline(&extract);

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
        renderer.read_last_virtual_geometry_node_and_cluster_cull_source(),
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected the NodeAndClusterCull startup seam to keep publishing one global-state cull-input record whenever a VG extract exists, even if downstream selection/raster passes stay clear-only"
    );
    assert_eq!(
        renderer.last_virtual_geometry_node_and_cluster_cull_record_count(),
        1,
        "expected the NodeAndClusterCull startup seam to keep one published global-state record even when downstream selection/raster passes emit no clusters"
    );
    assert_eq!(
        renderer
            .read_last_virtual_geometry_node_and_cluster_cull_input_snapshot()
            .expect("expected node-and-cluster-cull startup readback to succeed")
            .expect("expected the node-and-cluster-cull startup seam to publish a real buffer"),
        RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 0,
            page_budget: 0,
            instance_count: 0,
            cluster_count: 0,
            page_count: 0,
            visible_entity_count: 0,
            visible_cluster_count: 0,
            resident_page_count: 0,
            pending_page_request_count: 0,
            available_page_slot_count: 0,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState::default(),
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        },
        "expected the NodeAndClusterCull startup seam to preserve the zero-work global-state layout so a later NaniteGlobalStateBuffer consumer can still bind deterministic camera/budget/debug input on empty VG frames"
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
    assert_eq!(
        renderer.read_last_virtual_geometry_selected_cluster_source(),
        RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no executed cluster selections to report an explicit clear-only selected-cluster pass instead of collapsing that state into Unavailable"
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

fn build_node_and_cluster_cull_history_extract(
    viewport_size: UVec2,
    camera_transform: Transform,
) -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = camera_transform;
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![virtual_geometry_cluster(mesh, 20, 200, 0, Vec3::ZERO, 1.0)],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
            mesh_name: Some("NodeAndClusterCullHistoryUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    });
    extract
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
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: pages.clone(),
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
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}
