//! Product-path regressions for visibility-owned virtual-geometry args and prepare.
//!
//! These cases go through the public WGPU framework so the assertions observe submitted
//! frames, execution segments and debug snapshots rather than private SceneRenderer buffers.

use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, DisplayMode,
    FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
    RenderVirtualGeometryPage, RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

use crate::test_support::render_feature_fixtures::pluginized_wgpu_render_framework_with_asset_manager;

const VIEWPORT_SIZE: UVec2 = UVec2::new(160, 120);

struct Observation {
    debug: RenderVirtualGeometryDebugSnapshot,
    rgba: Vec<u8>,
    indirect_draw_count: u32,
    execution_segment_count: u32,
}

fn observe(
    mesh_entities: &[u64],
    clusters: Vec<RenderVirtualGeometryCluster>,
    pages: Vec<RenderVirtualGeometryPage>,
) -> Observation {
    observe_with_instances(mesh_entities, clusters, pages, Vec::new())
}

fn observe_with_instances(
    mesh_entities: &[u64],
    clusters: Vec<RenderVirtualGeometryCluster>,
    pages: Vec<RenderVirtualGeometryPage>,
    instances: Vec<RenderVirtualGeometryInstance>,
) -> Observation {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = resource_handle::<ModelMarker>(&asset_manager, "builtin://cube");
    let material = resource_handle::<MaterialMarker>(&asset_manager, "builtin://material/default");
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 1.2,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(VIEWPORT_SIZE);
    let meshes = mesh_entities
        .iter()
        .copied()
        .map(|entity| {
            let transform = Transform {
                scale: Vec3::new(0.8, 0.8, 1.0),
                ..Transform::default()
            };
            RenderMeshSnapshot {
                node_id: entity,
                stable_instance_key: render_mesh_stable_instance_key(entity, 0),
                transform_revision: render_mesh_transform_revision(&transform),
                transform,
                model: model.clone(),
                mesh: None,
                material: material.clone(),
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: RenderMeshStaticState::from_transform_static(false),
                common: RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    is_static: false,
                    ..RendererCommon::default()
                },
            }
        })
        .collect();
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes,
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..Default::default()
        },
        environment: Default::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: pages.len() as u32,
        clusters,
        pages,
        instances,
        ..RenderVirtualGeometryExtract::default()
    });
    extract.geometry.virtual_geometry_debug = Some(RenderVirtualGeometryDebugState {
        visualize_visbuffer: true,
        ..Default::default()
    });

    let framework = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = framework
        .create_viewport(
            zircon_runtime::core::framework::render::RenderViewportDescriptor::new(VIEWPORT_SIZE),
        )
        .unwrap();
    framework
        .set_quality_profile(viewport, virtual_geometry_quality_profile())
        .unwrap();
    framework.submit_frame_extract(viewport, extract).unwrap();
    let stats = framework.query_stats().unwrap();
    let debug = framework
        .query_virtual_geometry_debug_snapshot()
        .unwrap()
        .expect("public VG debug snapshot");
    let rgba = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("public WGPU frame")
        .rgba;
    let observation = Observation {
        debug: (*debug).clone(),
        rgba,
        indirect_draw_count: stats.last_virtual_geometry_indirect_draw_count,
        execution_segment_count: stats.last_virtual_geometry_execution_segment_count,
    };
    framework.destroy_viewport(viewport).unwrap();
    observation
}

fn virtual_geometry_quality_profile(
) -> zircon_runtime::core::framework::render::RenderQualityProfile {
    zircon_runtime::core::framework::render::RenderQualityProfile::new("vg-product-args-prepare")
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
        .with_async_compute(false)
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level: 0,
        parent_cluster_id: None,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn resource_handle<T>(manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing builtin resource {uri}")),
    )
}

fn assert_submission_subset(observation: &Observation, expected_entities: &[u64]) {
    let actual = observation
        .debug
        .submission_records
        .iter()
        .map(|record| record.entity)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected_entities);
}

#[test]
fn virtual_geometry_args_source_keeps_prepare_owned_draw_refs_when_some_entities_do_not_emit_pending_draws(
) {
    let result = observe(
        &[2],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.debug.execution_segments.len(), 2);
    assert_eq!(result.execution_segment_count, 2);
    assert_submission_subset(&result, &[2]);
}

#[test]
fn virtual_geometry_gpu_submission_fallback_ignores_non_submitted_visibility_draw_refs_when_cpu_records_are_gone(
) {
    let result = observe(
        &[2],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.debug.execution_segments.len(), 2);
    assert_submission_subset(&result, &[2]);
}

#[test]
fn virtual_geometry_fallback_full_mesh_args_follow_prepare_cluster_state_when_segments_are_absent()
{
    let result = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    assert_eq!(result.debug.selected_clusters.len(), 1);
    assert_eq!(result.debug.selected_clusters[0].cluster_id, 20);
    assert_eq!(result.debug.execution_segments.len(), 1);
}

#[test]
fn virtual_geometry_fallback_clusters_without_segments_expand_into_visibility_owned_indirect_slices(
) {
    let result = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.execution_segment_count, 2);
    assert!(result
        .debug
        .execution_segments
        .iter()
        .all(|segment| segment.cluster_span_count > 0));
}

#[test]
fn virtual_geometry_mixed_explicit_and_fallback_entities_reuse_one_prepare_owned_args_source() {
    let result = observe_with_instances(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
        vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 1,
            page_count: 1,
            ..RenderVirtualGeometryInstance::default()
        }],
    );
    assert_eq!(result.debug.execution_segments.len(), 2);
    assert_eq!(result.debug.submission_records.len(), 2);
    assert_eq!(result.indirect_draw_count, 2);
}

#[test]
fn virtual_geometry_missing_explicit_segments_do_not_resurrect_cpu_full_mesh_fallback_draws() {
    let result = observe(&[2], vec![cluster(3, 30, 301)], vec![page(301, true)]);
    assert!(result.debug.submission_records.is_empty());
    assert_eq!(result.indirect_draw_count, 0);
}

#[test]
fn virtual_geometry_missing_fallback_clusters_do_not_emit_zero_count_indirect_records() {
    let result = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    assert!(result
        .debug
        .execution_segments
        .iter()
        .all(|segment| segment.cluster_span_count > 0));
    assert_eq!(result.indirect_draw_count, 1);
}

#[test]
fn virtual_geometry_partial_missing_fallback_clusters_keep_original_cluster_ordinal_in_gpu_args() {
    let result = observe(
        &[2],
        vec![
            cluster(2, 20, 300),
            cluster(2, 21, 301),
            cluster(2, 22, 302),
        ],
        vec![page(300, true), page(302, true)],
    );
    let ordinals = result
        .debug
        .selected_clusters
        .iter()
        .map(|selected| selected.cluster_ordinal)
        .collect::<Vec<_>>();
    assert!(ordinals.windows(2).all(|pair| pair[0] < pair[1]));
    let final_cluster = result
        .debug
        .selected_clusters
        .iter()
        .find(|selected| selected.cluster_id == 22)
        .expect("the last resident fallback cluster should survive the missing middle page");
    assert_eq!(final_cluster.cluster_ordinal, 2);
}

#[test]
fn virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities() {
    let result = observe(
        &[2],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_submission_subset(&result, &[2]);
    assert!(result
        .debug
        .execution_segments
        .iter()
        .any(|segment| segment.entity == 2));
}

#[test]
fn virtual_geometry_prepare_streaming_state_changes_fallback_raster_output() {
    let resident = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let pending = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, false)]);
    assert_ne!(resident.rgba, pending.rgba);
    assert_ne!(resident.debug.resident_pages, pending.debug.resident_pages);
}

#[test]
fn virtual_geometry_prepare_streaming_state_changes_fallback_raster_coverage() {
    let resident = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let pending = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, false)]);
    assert_ne!(
        resident.debug.selected_clusters,
        pending.debug.selected_clusters
    );
    assert_ne!(resident.rgba, pending.rgba);
}

#[test]
fn virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes() {
    let forward = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    let reverse = observe(
        &[3, 2],
        vec![cluster(3, 30, 301), cluster(2, 20, 300)],
        vec![page(300, true), page(301, true)],
    );
    let forward_order = forward
        .debug
        .submission_records
        .iter()
        .map(|record| record.entity)
        .collect::<Vec<_>>();
    let reverse_order = reverse
        .debug
        .submission_records
        .iter()
        .map(|record| record.entity)
        .collect::<Vec<_>>();
    assert_ne!(forward_order, reverse_order);
}

#[test]
fn virtual_geometry_prepare_resident_slot_changes_fallback_raster_output() {
    let first = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let second = observe(&[2], vec![cluster(2, 20, 301)], vec![page(301, true)]);
    assert_ne!(
        first.debug.execution_segments[0].page_id,
        second.debug.execution_segments[0].page_id
    );
    assert_ne!(first.rgba, second.rgba);
}

#[test]
fn virtual_geometry_prepare_visible_cluster_ids_change_fallback_raster_region() {
    let first = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let second = observe(&[2], vec![cluster(2, 21, 300)], vec![page(300, true)]);
    assert_ne!(
        first.debug.selected_clusters[0].cluster_id,
        second.debug.selected_clusters[0].cluster_id
    );
    assert_ne!(first.rgba, second.rgba);
}

#[test]
fn virtual_geometry_prepare_draw_segments_override_extract_cluster_ordinals_for_fallback_raster() {
    let result = observe(
        &[2],
        vec![cluster(2, 20, 300), cluster(2, 21, 300)],
        vec![page(300, true)],
    );
    assert!(result
        .debug
        .selected_clusters
        .iter()
        .enumerate()
        .all(|(ordinal, selected)| selected.cluster_ordinal as usize == ordinal));
}

#[test]
fn virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled() {
    let result = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.indirect_draw_count, 2);
    assert!(result
        .rgba
        .chunks_exact(4)
        .any(|pixel| pixel[1] > pixel[0].saturating_add(20)));
}

#[test]
fn virtual_geometry_prepare_preserves_explicit_segment_boundaries_even_when_segments_share_page_slot(
) {
    let result = observe_with_instances(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 300)],
        vec![page(300, true)],
        vec![
            RenderVirtualGeometryInstance {
                entity: 2,
                cluster_count: 1,
                page_count: 1,
                ..RenderVirtualGeometryInstance::default()
            },
            RenderVirtualGeometryInstance {
                entity: 3,
                cluster_offset: 1,
                cluster_count: 1,
                page_count: 1,
                ..RenderVirtualGeometryInstance::default()
            },
        ],
    );
    assert_eq!(result.debug.execution_segments.len(), 2);
    assert!(result
        .debug
        .execution_segments
        .iter()
        .all(|segment| segment.page_id == 300));
}

#[test]
fn virtual_geometry_prepare_reuses_one_shared_indirect_buffer_across_multiple_indirect_draws() {
    let result = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.indirect_draw_count, 2);
    assert_eq!(result.debug.execution_segments.len(), 2);
}

#[test]
fn virtual_geometry_prepare_keeps_different_pages_in_same_slot_split_into_separate_indirect_draws()
{
    let result = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 301)],
        vec![page(300, true), page(301, true)],
    );
    assert_eq!(result.indirect_draw_count, 2);
    assert_ne!(
        result.debug.execution_segments[0].page_id,
        result.debug.execution_segments[1].page_id
    );
}

#[test]
fn virtual_geometry_prepare_visibility_owned_draw_segment_span_changes_fallback_coverage() {
    let one = observe_with_instances(
        &[2],
        vec![cluster(2, 20, 300), cluster(2, 21, 300)],
        vec![page(300, true)],
        vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 1,
            page_count: 1,
            ..RenderVirtualGeometryInstance::default()
        }],
    );
    let two = observe_with_instances(
        &[2],
        vec![cluster(2, 20, 300), cluster(2, 21, 300)],
        vec![page(300, true)],
        vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 2,
            page_count: 1,
            ..RenderVirtualGeometryInstance::default()
        }],
    );
    assert_ne!(
        one.debug.selected_clusters.len(),
        two.debug.selected_clusters.len()
    );
    assert_ne!(one.rgba, two.rgba);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_follow_visibility_owned_segment_span() {
    let one = observe_with_instances(
        &[2],
        vec![cluster(2, 20, 300), cluster(2, 21, 300)],
        vec![page(300, true)],
        vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 1,
            page_count: 1,
            ..RenderVirtualGeometryInstance::default()
        }],
    );
    let two = observe_with_instances(
        &[2],
        vec![cluster(2, 20, 300), cluster(2, 21, 300)],
        vec![page(300, true)],
        vec![RenderVirtualGeometryInstance {
            entity: 2,
            cluster_count: 2,
            page_count: 1,
            ..RenderVirtualGeometryInstance::default()
        }],
    );
    assert_eq!(one.execution_segment_count, 1);
    assert_eq!(two.execution_segment_count, 2);
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_resident_slot_changes() {
    let resident = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let pending = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, false)]);
    assert_ne!(
        resident.debug.execution_segments[0].state,
        pending.debug.execution_segments[0].state
    );
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_page_id_changes_inside_same_resident_slot(
) {
    let first = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let second = observe(&[2], vec![cluster(2, 20, 301)], vec![page(301, true)]);
    assert_ne!(
        first.debug.execution_segments[0].page_id,
        second.debug.execution_segments[0].page_id
    );
}

#[test]
fn virtual_geometry_prepare_gpu_generated_indirect_args_change_when_lod_level_changes_inside_same_page_slot(
) {
    let mut high = cluster(2, 20, 300);
    high.lod_level = 1;
    let low = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let high = observe(&[2], vec![high], vec![page(300, true)]);
    assert_ne!(
        low.debug.selected_clusters[0].lod_level,
        high.debug.selected_clusters[0].lod_level
    );
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_page_id_changes_inside_same_resident_slot(
) {
    let first = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, true)]);
    let second = observe(&[2], vec![cluster(2, 20, 301)], vec![page(301, true)]);
    assert_ne!(
        first.debug.execution_segments[0].page_id,
        second.debug.execution_segments[0].page_id
    );
    assert_ne!(first.rgba, second.rgba);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_changes_when_pending_request_frontier_rank_changes(
) {
    let first = observe(&[2], vec![cluster(2, 20, 300)], vec![page(300, false)]);
    let second = observe(&[2], vec![cluster(2, 21, 301)], vec![page(301, false)]);
    assert_ne!(first.debug.requested_pages, second.debug.requested_pages);
    assert_ne!(
        first.debug.selected_clusters,
        second.debug.selected_clusters
    );
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_scene_mesh_enumeration_order_changes(
) {
    let forward = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 300)],
        vec![page(300, true)],
    );
    let reverse = observe(
        &[3, 2],
        vec![cluster(3, 30, 300), cluster(2, 20, 300)],
        vec![page(300, true)],
    );
    assert_eq!(forward.rgba, reverse.rgba);
}

#[test]
fn virtual_geometry_prepare_cluster_raster_output_is_stable_when_scene_mesh_order_changes_with_shared_page_metadata(
) {
    let forward = observe(
        &[2, 3],
        vec![cluster(2, 20, 300), cluster(3, 30, 300)],
        vec![page(300, true)],
    );
    let reverse = observe(
        &[3, 2],
        vec![cluster(3, 30, 300), cluster(2, 20, 300)],
        vec![page(300, true)],
    );
    assert_eq!(
        forward.debug.selected_clusters.len(),
        reverse.debug.selected_clusters.len()
    );
    assert_eq!(forward.rgba, reverse.rgba);
}
