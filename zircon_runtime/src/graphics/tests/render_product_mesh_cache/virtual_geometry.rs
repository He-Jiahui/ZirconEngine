use std::{fs, path::PathBuf, sync::Arc};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    cook_virtual_geometry_from_mesh, AssetUri, MaterialAsset, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, VirtualGeometryCookConfig,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, CapturedFrame, DisplayMode, GeometryExtract, ProjectionMode,
    RenderCameraClear, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMaterialLightingModel, RenderMeshSnapshot, RenderMeshStaticState, RenderQualityProfile,
    RenderStats, RenderViewportDescriptor, RenderVirtualGeometryCluster,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryPage, RenderVirtualGeometryPayloadSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};

use super::super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework_with_advanced_providers,
    pluginized_wgpu_render_framework_with_advanced_providers_and_asset_manager,
};
use super::super::render_product_submit::{
    material_with_import_note, snapshot_with_projection_for_mesh_cache_tests,
};
use super::register_material_asset_revision;

const READBACK_STATUS: &str =
    "render_plan08_virtual_geometry_product_draw_source_readback_passed_targeted_cargo";
const PAGE_CLUSTER_PRODUCT_STATUS: &str =
    "render_plan08_virtual_geometry_page_cluster_product_execution_wgpu_passed_renderdoc_deferred";
const PAGE_CLUSTER_PRODUCT_PNG_STATUS: &str =
    "render_plan08_virtual_geometry_page_cluster_product_readback_png_passed_renderdoc_deferred";

#[test]
fn render_product_virtual_geometry_model_asset_uses_automatic_draw_source() {
    assert!(!READBACK_STATUS.is_empty());

    let (frame, stats) =
        capture_automatic_virtual_geometry_product_frame("product-virtual-geometry-automatic", 697);

    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::AutomaticFallback,
        "product VG should come from the model asset/provider path, not a hand-authored extract",
    );
    assert_eq!(
        stats.last_virtual_geometry_instance_count, 1,
        "automatic VG should preserve the submitted mesh instance as the draw source",
    );
    assert!(
        stats.last_virtual_geometry_input_cluster_count >= 1,
        "automatic VG should expose cooked model clusters to visibility",
    );
    assert!(
        stats.last_virtual_geometry_input_page_count >= 1,
        "automatic VG should expose cooked model pages to streaming",
    );
    assert!(
        stats.last_virtual_geometry_indirect_draw_count >= 1,
        "automatic VG should still produce GPU-driven execution draws",
    );
    assert!(
        stats.last_virtual_geometry_indirect_segment_count >= 1,
        "automatic VG should produce executable indirect segments",
    );
    assert_virtual_geometry_execution_stats_visible(&stats);
    assert_virtual_geometry_capture_visible(&frame);
}

#[test]
fn render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame() {
    assert!(!PAGE_CLUSTER_PRODUCT_STATUS.is_empty());

    let (frame, stats) = capture_automatic_virtual_geometry_product_frame(
        "product-virtual-geometry-page-cluster",
        698,
    );

    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::AutomaticFallback,
        "page/cluster product proof must stay on the automatic ModelAsset VG path",
    );
    assert_virtual_geometry_execution_stats_visible(&stats);
    assert_virtual_geometry_page_cluster_product_bindings_executed(&stats);
    assert_virtual_geometry_capture_visible(&frame);
}

#[test]
#[ignore = "manual product PNG export for Plan 08 VirtualGeometry page/cluster evidence"]
fn export_virtual_geometry_page_cluster_product_png() {
    assert!(!PAGE_CLUSTER_PRODUCT_PNG_STATUS.is_empty());

    let (frame, stats) = capture_automatic_virtual_geometry_product_frame(
        "product-virtual-geometry-page-cluster-png",
        699,
    );
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::AutomaticFallback,
        "page/cluster PNG export should stay on the automatic ModelAsset VG path",
    );
    assert_virtual_geometry_execution_stats_visible(&stats);
    assert_virtual_geometry_page_cluster_product_bindings_executed(&stats);
    assert_virtual_geometry_capture_visible(&frame);

    let output_path = render_test_output_dir()
        .join("runtime_render_plan08_virtual_geometry_page_cluster_product_20260703.png");
    save_virtual_geometry_product_frame(&frame, &output_path);
}

fn capture_automatic_virtual_geometry_product_frame(
    quality_label: &'static str,
    entity: u64,
) -> (CapturedFrame, RenderStats) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model_uri = AssetUri::parse("res://models/product-vg.model.toml").unwrap();
    let model_id = ResourceId::from_locator(&model_uri);
    register_virtual_geometry_model_revision(
        &asset_manager,
        model_id,
        model_uri,
        "product-vg-model-v1",
    );
    let material_uri = AssetUri::parse("res://materials/product-vg-visible.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_asset_revision(
        &asset_manager,
        material_id,
        material_uri,
        "product-vg-visible-material-v1",
        unlit_virtual_geometry_material(),
    );

    let framework =
        pluginized_wgpu_render_framework_with_advanced_providers_and_asset_manager(asset_manager);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new(quality_label)
                .with_virtual_geometry(true)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            automatic_virtual_geometry_model_extract(model_id, material_id, entity),
        )
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("automatic VG product frame should be capturable");
    let stats = framework.query_stats().unwrap();

    (frame, stats)
}

#[test]
fn render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache() {
    let framework = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-virtual-geometry-residual")
                .with_virtual_geometry(true)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_virtual_geometry_extract(597))
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert_eq!(
        first.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert!(
        first.last_virtual_geometry_indirect_draw_count >= 1,
        "authored virtual geometry should still produce GPU-driven execution draws",
    );
    assert!(
        first.last_virtual_geometry_indirect_buffer_count >= 1,
        "authored virtual geometry should create mesh-level WGPU indirect buffers",
    );
    assert!(
        first.last_virtual_geometry_indirect_args_count >= 1,
        "authored virtual geometry should populate indexed indirect args",
    );
    assert!(
        first.last_virtual_geometry_indirect_segment_count >= 1,
        "authored virtual geometry should record executable indirect segments",
    );
    assert_virtual_geometry_execution_stats_visible(&first);
    assert_eq!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "virtual-geometry visibility carrier meshes must not be advertised as static mesh command-cache candidates",
    );
    assert_eq!(
        first.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);
    assert!(
        first.last_mesh_dynamic_command_count
            >= first.last_virtual_geometry_execution_segment_count,
        "virtual-geometry execution commands should remain on the dynamic indirect replay path",
    );

    framework
        .submit_frame_extract(viewport, static_cache_virtual_geometry_extract(598))
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert_eq!(
        second.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert!(
        second.last_virtual_geometry_indirect_draw_count >= 1,
        "virtual geometry remains GPU-driven across frames instead of being absorbed by MeshDraw cache",
    );
    assert!(
        second.last_virtual_geometry_indirect_buffer_count >= 1,
        "virtual geometry keeps mesh-level WGPU indirect buffers across frames",
    );
    assert!(
        second.last_virtual_geometry_indirect_args_count >= 1,
        "virtual geometry keeps indexed indirect args across frames",
    );
    assert!(
        second.last_virtual_geometry_indirect_segment_count >= 1,
        "virtual geometry keeps executable indirect segments across frames",
    );
    assert_virtual_geometry_execution_stats_visible(&second);
    assert_eq!(
        second.last_mesh_pending_static_command_cache_draw_candidate_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count,
        0,
    );
    assert_eq!(second.last_mesh_cached_command_hit_count, 0);
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert!(
        second.last_mesh_dynamic_command_count
            >= second.last_virtual_geometry_execution_segment_count,
        "virtual-geometry execution commands should remain on the dynamic indirect replay path",
    );
    assert!(
        second.last_mesh_command_rebuild_count >= second.last_mesh_dynamic_command_count,
        "dynamic virtual-geometry indirect commands should be reflected in command rebuild stats",
    );
}

fn static_cache_virtual_geometry_extract(world: u64) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![static_cache_virtual_geometry_visibility_mesh()],
    );
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            static_cache_virtual_geometry_cluster(803, 15, 150, 1, Vec3::new(100.0, 0.0, 0.0), 9.0),
            static_cache_virtual_geometry_cluster(803, 30, 300, 0, Vec3::ZERO, 8.0),
            static_cache_virtual_geometry_cluster(803, 20, 200, 1, Vec3::new(0.1, 0.0, 0.0), 5.0),
            static_cache_virtual_geometry_cluster(803, 10, 100, 2, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            static_cache_virtual_geometry_page(100, false),
            static_cache_virtual_geometry_page(150, false),
            static_cache_virtual_geometry_page(200, true),
            static_cache_virtual_geometry_page(300, false),
            static_cache_virtual_geometry_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract
}

fn static_cache_virtual_geometry_visibility_mesh() -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 803,
        stable_instance_key: 803 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "builtin://material/default",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}

fn assert_virtual_geometry_execution_stats_visible(stats: &RenderStats) {
    assert!(
        stats.last_virtual_geometry_execution_segment_count >= 1,
        "product stats should expose executable virtual-geometry segments",
    );
    assert!(
        stats.last_virtual_geometry_execution_page_count >= 1,
        "product stats should retain the resident/requested page set used by VG execution",
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_missing_segment_count, 0,
        "the product fixture keeps executable segments on resident or requested pages",
    );
    assert!(
        stats.last_virtual_geometry_execution_resident_segment_count
            + stats.last_virtual_geometry_execution_pending_segment_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "resident and pending execution buckets should cover executable VG segments",
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_source,
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
    );
    assert_eq!(
        stats.last_virtual_geometry_visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
    );
    assert!(
        stats.last_virtual_geometry_selected_cluster_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "selected cluster stats should cover every executable VG segment",
    );
    assert!(
        stats.last_virtual_geometry_visbuffer64_entry_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "visbuffer64 stats should cover every executable VG segment",
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_record_count,
        stats.last_virtual_geometry_execution_segment_count,
    );
}

fn assert_virtual_geometry_page_cluster_product_bindings_executed(stats: &RenderStats) {
    assert!(
        stats.last_virtual_geometry_input_page_count
            >= stats.last_virtual_geometry_execution_page_count,
        "VG product execution should keep extracted pages for every execution page; input_pages={}, execution_pages={}",
        stats.last_virtual_geometry_input_page_count,
        stats.last_virtual_geometry_execution_page_count,
    );
    assert!(
        stats.last_virtual_geometry_visible_cluster_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "visible VG clusters should feed every execution segment; visible_clusters={}, execution_segments={}",
        stats.last_virtual_geometry_visible_cluster_count,
        stats.last_virtual_geometry_execution_segment_count,
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_resident_segment_count
            + stats.last_virtual_geometry_execution_pending_segment_count,
        stats.last_virtual_geometry_execution_segment_count,
        "resident plus pending VG execution buckets should exactly cover executable segments",
    );
    assert!(
        stats.last_virtual_geometry_indirect_buffer_count >= 1,
        "VG page/cluster product path should allocate an indirect buffer",
    );
    assert!(
        stats.last_virtual_geometry_indirect_args_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "VG indirect args should cover the execution segment set",
    );
    assert!(
        stats.last_virtual_geometry_selected_cluster_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "VG selected clusters should cover every execution segment",
    );
    assert!(
        stats.last_virtual_geometry_visbuffer64_entry_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "VG visbuffer64 entries should cover every execution segment",
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_record_count,
        stats.last_virtual_geometry_execution_segment_count,
        "hardware rasterization records should stay one-to-one with VG execution segments",
    );
}

fn static_cache_virtual_geometry_cluster(
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

fn static_cache_virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

fn register_virtual_geometry_model_revision(
    asset_manager: &ProjectAssetManager,
    model_id: ResourceId,
    model_uri: AssetUri,
    source_hash: &str,
) {
    asset_manager
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(model_id, ResourceKind::Model, model_uri.clone())
                .with_source_hash(source_hash),
            automatic_virtual_geometry_model_asset(model_uri),
        )
        .expect("virtual geometry model insert");
}

fn automatic_virtual_geometry_model_extract(
    model_id: ResourceId,
    material_id: ResourceId,
    world: u64,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Perspective,
        transform: Transform::from_translation(Vec3::new(0.45, 0.05, 4.0)),
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(UVec2::new(320, 240));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(camera.projection_mode),
    );
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(697), camera);
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    extract.view.select_camera_descriptor(descriptor);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![automatic_virtual_geometry_model_mesh(model_id, material_id)],
    );
    extract
}

fn automatic_virtual_geometry_model_mesh(
    model_id: ResourceId,
    material_id: ResourceId,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 903,
        stable_instance_key: 903 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(model_id),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}

fn automatic_virtual_geometry_model_asset(uri: AssetUri) -> ModelAsset {
    let (vertices, indices) = product_virtual_geometry_mesh();
    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        &indices,
        VirtualGeometryCookConfig {
            cluster_triangle_count: 1,
            page_cluster_count: 2,
            mesh_name: Some("ProductAutomaticVG".to_string()),
            source_hint: Some("render-product".to_string()),
        },
    )
    .expect("product VG mesh should cook");
    let mut primitive = ModelPrimitiveAsset {
        vertices,
        indices,
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: Some(virtual_geometry),
    };
    primitive.assign_virtual_geometry_vertex_ordinals();

    ModelAsset {
        uri,
        primitives: vec![primitive],
    }
}

fn unlit_virtual_geometry_material() -> MaterialAsset {
    let mut material = material_with_import_note();
    material.base_color = [0.12, 0.82, 0.94, 1.0];
    material.emissive = [0.0, 0.0, 0.0];
    material.validation_diagnostics.clear();
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String(RenderMaterialLightingModel::Unlit.to_string()),
    );
    material
}

fn assert_virtual_geometry_capture_visible(frame: &CapturedFrame) {
    let visible_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count();
    assert!(
        visible_pixels >= 24,
        "automatic VG product path should leave visible pixels in the captured product frame; visible_pixels={visible_pixels}; frame={}x{}",
        frame.width,
        frame.height
    );
}

fn render_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below repository root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render");
    fs::create_dir_all(&output_dir).expect("render product test output dir should be writable");
    output_dir
}

fn save_virtual_geometry_product_frame(frame: &CapturedFrame, output_path: &PathBuf) {
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone())
        .expect("captured VG frame dimensions should match RGBA byte length");
    image
        .save_with_format(output_path, ImageFormat::Png)
        .expect("VG product PNG should be writable");
}

fn product_virtual_geometry_mesh() -> (Vec<MeshVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for triangle_index in 0..5_u32 {
        let x = triangle_index as f32 * 0.2;
        let base = vertices.len() as u32;
        vertices.push(MeshVertex::new(Vec3::new(x, 0.0, 0.0), Vec3::Y, Vec2::ZERO));
        vertices.push(MeshVertex::new(
            Vec3::new(x + 0.1, 0.0, 0.0),
            Vec3::Y,
            Vec2::X,
        ));
        vertices.push(MeshVertex::new(Vec3::new(x, 0.1, 0.0), Vec3::Y, Vec2::Y));
        indices.extend([base, base + 1, base + 2]);
    }

    (vertices, indices)
}
