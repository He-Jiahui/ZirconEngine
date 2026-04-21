use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderWorldSnapshotHandle,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::world::World;

#[test]
fn render_framework_visualize_visbuffer_draws_only_execution_subset_in_captured_frame() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(160, 120);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-visbuffer-overlay-contract")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let without_overlay = build_extract(RenderVirtualGeometryDebugState::default(), viewport_size);
    server
        .submit_frame_extract(viewport, without_overlay)
        .expect("baseline frame submission should succeed");
    let without_overlay = server
        .capture_frame(viewport)
        .expect("baseline capture should succeed")
        .expect("baseline frame should exist");

    let with_overlay = build_extract(
        RenderVirtualGeometryDebugState {
            visualize_visbuffer: true,
            ..RenderVirtualGeometryDebugState::default()
        },
        viewport_size,
    );
    server
        .submit_frame_extract(viewport, with_overlay)
        .expect("overlay frame submission should succeed");
    let with_overlay = server
        .capture_frame(viewport)
        .expect("overlay capture should succeed")
        .expect("overlay frame should exist");
    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");

    assert_eq!(
        snapshot
            .execution_segments
            .iter()
            .map(|segment| (
                segment.cluster_start_ordinal,
                segment.cluster_span_count,
                segment.page_id,
                segment.state,
            ))
            .collect::<Vec<_>>(),
        vec![(1, 1, 300, RenderVirtualGeometryExecutionState::Resident)],
        "expected the real execution subset to keep only the resident right-hand cluster before overlay rendering runs"
    );
    assert_eq!(
        snapshot
            .visbuffer_debug_marks
            .iter()
            .map(|mark| (mark.cluster_id, mark.page_id, mark.state))
            .collect::<Vec<_>>(),
        vec![(30, 300, RenderVirtualGeometryExecutionState::Resident)],
        "expected the stored renderer-owned snapshot to keep only the execution-backed visbuffer mark"
    );

    let left_differing_pixels = differing_pixels_in_half(
        &without_overlay.rgba,
        &with_overlay.rgba,
        viewport_size,
        Half::Left,
    );
    assert_eq!(
        left_differing_pixels, 0,
        "expected the missing left-hand cluster to stay out of the captured visbuffer overlay once same-frame overlay generation follows the execution subset"
    );
}

fn build_extract(
    debug: RenderVirtualGeometryDebugState,
    viewport_size: UVec2,
) -> RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut snapshot = world.to_render_snapshot();
    snapshot.scene.directional_lights.clear();
    snapshot.scene.point_lights.clear();
    snapshot.scene.spot_lights.clear();
    snapshot.preview.lighting_enabled = false;
    snapshot.preview.skybox_enabled = false;
    snapshot.preview.clear_color = Vec4::ZERO;

    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 0,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 0, Vec3::new(-0.75, 0.6, 0.0)),
            virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::new(0.75, 0.6, 0.0)),
        ],
        pages: vec![
            virtual_geometry_page(200, false),
            virtual_geometry_page(300, true),
        ],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("VisBufferOverlayContractMesh".to_string()),
            source_hint: Some("integration-test".to_string()),
        }],
        debug,
    });
    extract
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    bounds_center: Vec3,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.18,
        screen_space_error: 1.0,
    }
}

fn virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

#[derive(Clone, Copy)]
enum Half {
    Left,
}

fn differing_pixels_in_half(
    before: &[u8],
    after: &[u8],
    viewport_size: UVec2,
    half: Half,
) -> usize {
    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let x_range = match half {
        Half::Left => 0..(width / 2).max(1),
    };

    let mut differing_pixels = 0usize;
    for y in 0..height {
        for x in x_range.clone() {
            let index = (y * width + x) * 4;
            if before[index..index + 4] != after[index..index + 4] {
                differing_pixels += 1;
            }
        }
    }

    differing_pixels
}
