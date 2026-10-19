use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderSceneSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle,
};
use zircon_runtime::core::framework::render::{
    RenderFramework, RenderFrameworkError, RenderQualityProfile, RenderViewportDescriptor,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::scene::world::World;

use crate::test_support::render_feature_fixtures::{
    pluginized_wgpu_render_framework_with_uploader_collector, PluginizedWgpuRenderFrameworkFixture,
};
use crate::virtual_geometry::{
    VirtualGeometryGpuReadback, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    VirtualGeometryPrepareRequest, VirtualGeometryUploaderTestCollector,
};

fn compile_virtual_geometry_pipeline(_extract: &RenderFrameExtract) {}

fn virtual_geometry_scene_renderer(
    asset_manager: Arc<ProjectAssetManager>,
) -> PluginizedWgpuRenderFrameworkFixture {
    pluginized_wgpu_render_framework_with_uploader_collector(
        asset_manager,
        Arc::new(VirtualGeometryUploaderTestCollector::default()),
    )
}

struct ViewportRenderFrame {
    extract: RenderFrameExtract,
    viewport_size: UVec2,
    prepare: Option<VirtualGeometryPrepareFrame>,
}

impl ViewportRenderFrame {
    fn from_extract(extract: RenderFrameExtract, viewport_size: UVec2) -> Self {
        Self {
            extract,
            viewport_size,
            prepare: None,
        }
    }

    fn with_virtual_geometry_prepare(
        mut self,
        prepare: Option<VirtualGeometryPrepareFrame>,
    ) -> Self {
        self.prepare = prepare;
        self
    }
}

trait UploaderFrameTestExt {
    fn render_frame_with_pipeline(
        &mut self,
        frame: ViewportRenderFrame,
        _pipeline: &(),
        _capture: Option<()>,
    ) -> Result<(), RenderFrameworkError>;
    fn take_completed_uploader_readback(&self) -> Option<VirtualGeometryGpuReadback>;
}

impl UploaderFrameTestExt for PluginizedWgpuRenderFrameworkFixture {
    fn render_frame_with_pipeline(
        &mut self,
        mut frame: ViewportRenderFrame,
        _pipeline: &(),
        _capture: Option<()>,
    ) -> Result<(), RenderFrameworkError> {
        if let Some(prepare) = frame.prepare.take() {
            self.uploader_collector().set_prepare(prepare);
        }
        let viewport = self.create_viewport(RenderViewportDescriptor::new(frame.viewport_size))?;
        self.set_quality_profile(
            viewport,
            RenderQualityProfile::new("virtual-geometry-uploader-contract")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )?;
        self.test_support_submit_virtual_geometry_frame(viewport, frame.extract)
            .map(|_| ())
    }

    fn take_completed_uploader_readback(&self) -> Option<VirtualGeometryGpuReadback> {
        Some(
            self.uploader_collector()
                .take_readback()
                .expect("actual GPU readback should complete"),
        )
    }
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
        pages,
        page_dependencies: Vec::new(),
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

#[test]
fn virtual_geometry_gpu_uploader_deduplicates_against_three_word_completion_records() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 3, vec![page(400, false), page(500, false)]);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: Vec::new(),
                    pending_page_requests: vec![
                        VirtualGeometryPrepareRequest {
                            page_id: 400,
                            size_bytes: 4096,
                            generation: 1,
                            frontier_rank: 0,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 500,
                            size_bytes: 4096,
                            generation: 2,
                            frontier_rank: 1,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                        VirtualGeometryPrepareRequest {
                            page_id: 500,
                            size_bytes: 4096,
                            generation: 3,
                            frontier_rank: 2,
                            assigned_slot: None,
                            recycled_page_id: None,
                        },
                    ],
                    available_slots: vec![0, 1, 2],
                    evictable_pages: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_completed_uploader_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.completed_page_ids(), vec![400, 500]);
}

#[test]
fn virtual_geometry_gpu_uploader_saturates_streaming_plus_reclaimable_byte_budget() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = virtual_geometry_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size, 1, vec![page(200, true), page(300, false)]);
    let compiled = compile_virtual_geometry_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_virtual_geometry_prepare(Some(VirtualGeometryPrepareFrame {
                    visible_entities: Vec::new(),
                    visible_clusters: Vec::new(),
                    cluster_draw_segments: Vec::new(),
                    resident_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: 1,
                    }],
                    pending_page_requests: vec![VirtualGeometryPrepareRequest {
                        page_id: 300,
                        size_bytes: 4096,
                        generation: 1,
                        frontier_rank: 0,
                        assigned_slot: None,
                        recycled_page_id: None,
                    }],
                    available_slots: Vec::new(),
                    evictable_pages: vec![VirtualGeometryPreparePage {
                        page_id: 200,
                        slot: 0,
                        size_bytes: u64::MAX,
                    }],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_completed_uploader_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.completed_page_ids(), vec![300]);
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
            ViewportRenderFrame::from_extract(extract, viewport_size)
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
        .take_completed_uploader_readback()
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
            ViewportRenderFrame::from_extract(extract, viewport_size)
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
        .take_completed_uploader_readback()
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
            ViewportRenderFrame::from_extract(extract, viewport_size)
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
        .take_completed_uploader_readback()
        .expect("expected virtual geometry GPU readback");
    assert_eq!(readback.page_table_entries(), vec![(200, 0)]);
    assert_eq!(readback.completed_page_ids(), Vec::<u32>::new());
    assert_eq!(
        readback.completed_page_assignments(),
        Vec::<(u32, u32)>::new()
    );
}
