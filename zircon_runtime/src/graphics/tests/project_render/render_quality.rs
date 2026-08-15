use std::fs;

use crate::asset::pipeline::manager::AssetManager;
use crate::asset::project::ProjectManager;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderDirectionalLightSnapshot, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderPipelineHandle, RenderQualityProfile, RenderStats,
    RenderViewportDescriptor,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker};
use crate::graphics::WgpuRenderFramework;
use crate::scene::components::{default_render_layer_mask, Mobility};

use super::super::plugin_render_feature_fixtures::default_rendering_feature_descriptors;
use super::{
    average_channel, average_channel_in_region, average_luma, build_snapshot,
    dominant_green_pixels, fullscreen_quad_transform, offset_quad_transform,
    project_asset_manager_with_first_wave_plugin_importers, resource_handle, submit_snapshot,
    unique_temp_project_root, write_checker_png, write_flat_color_wgsl, write_material,
    write_material_with_base_color_and_texture, write_quad_obj, write_scene, write_solid_png,
};
#[test]
fn temporal_history_rotates_history_when_scene_material_changes() {
    let root = unique_temp_project_root("graphics_temporal_history");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsTemporalHistory",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.02, 0.92, 0.1],
    );
    write_flat_color_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_black.wgsl"),
        [0.0, 0.0, 0.0],
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_quad_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("flat_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("flat_black.zmaterial"),
        "res://shaders/flat_black.wgsl",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let green_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_green.zmaterial");
    let black_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_black.zmaterial");
    let viewport_size = UVec2::new(160, 120);

    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let history_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            history_viewport,
            RenderQualityProfile::new("history-only")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(true),
        )
        .unwrap();

    submit_snapshot(
        &server,
        history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: fullscreen_quad_transform(),
                model,
                mesh: None,
                material: green_material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    ..Default::default()
                },
            }],
            Vec::new(),
            viewport_size,
        ),
    );
    let first_history = server.query_stats().unwrap().last_frame_history;
    let _ = server.capture_frame(history_viewport).unwrap();
    let history_frame = submit_snapshot(
        &server,
        history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: fullscreen_quad_transform(),
                model,
                mesh: None,
                material: black_material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    ..Default::default()
                },
            }],
            Vec::new(),
            viewport_size,
        ),
    );
    let second_history = server.query_stats().unwrap().last_frame_history;

    let no_history_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            no_history_viewport,
            RenderQualityProfile::new("no-history")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let no_history_frame = submit_snapshot(
        &server,
        no_history_viewport,
        build_snapshot(
            vec![RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: fullscreen_quad_transform(),
                model,
                mesh: None,
                material: black_material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    ..Default::default()
                },
            }],
            Vec::new(),
            viewport_size,
        ),
    );

    let history_green_pixels = dominant_green_pixels(&history_frame.rgba);
    let no_history_green_pixels = dominant_green_pixels(&no_history_frame.rgba);
    assert_ne!(first_history, second_history);
    assert!(
        history_green_pixels <= no_history_green_pixels + 64,
        "history resolve should rotate on material change instead of preserving prior color; green pixels with history={history_green_pixels}, without history={no_history_green_pixels}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn ssao_quality_profile_darkens_scene_when_enabled() {
    let root = unique_temp_project_root("graphics_ssao");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsSsao",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_gray.wgsl"),
        [0.72, 0.72, 0.72],
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_quad_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("flat_gray.zmaterial"),
        "res://shaders/flat_gray.wgsl",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_gray.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let snapshot = build_snapshot(
        vec![
            RenderMeshSnapshot {
                node_id: 1,
                stable_instance_key: 1 << 16,
                transform_revision: 0,
                transform: fullscreen_quad_transform(),
                model,
                mesh: None,
                material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    ..Default::default()
                },
            },
            RenderMeshSnapshot {
                node_id: 2,
                stable_instance_key: 2 << 16,
                transform_revision: 0,
                transform: offset_quad_transform(),
                model,
                mesh: None,
                material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        default_render_layer_mask(),
                    ),
                    ..Default::default()
                },
            },
        ],
        Vec::new(),
        viewport_size,
    );

    let server = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        asset_manager,
        default_rendering_feature_descriptors(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    let ao_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            ao_viewport,
            RenderQualityProfile::new("ao-on")
                .with_clustered_lighting(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let ao_frame = submit_snapshot(&server, ao_viewport, snapshot.clone());
    let ao_stats = server.query_stats().unwrap();
    assert_ssao_shared_hzb_product_path(&ao_stats);

    let no_ao_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            no_ao_viewport,
            RenderQualityProfile::new("ao-off")
                .with_clustered_lighting(false)
                .with_temporal_history(false)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();
    let no_ao_frame = submit_snapshot(&server, no_ao_viewport, snapshot);

    let ao_luma = average_luma(&ao_frame.rgba);
    let no_ao_luma = average_luma(&no_ao_frame.rgba);
    assert!(
        ao_luma + 5.0 < no_ao_luma,
        "expected SSAO-enabled output to be darker; ao luma={ao_luma:.2}, no-ao luma={no_ao_luma:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

fn assert_ssao_shared_hzb_product_path(stats: &RenderStats) {
    assert_graph_pass_executed(stats, "hzb-build");
    assert_graph_pass_executed(stats, "ssao-evaluate");
    assert_graph_executor_executed(stats, "visibility.hzb-build");
    assert_graph_executor_executed(stats, "compute.generic");
    assert!(
        stats.last_hzb_mip_count > 1,
        "SSAO product path should build a shared HZB mip chain; stats={stats:?}"
    );
    assert!(
        stats.last_hzb_graph_executed_pass_count > 0,
        "SSAO product path should execute HZB graph work; stats={stats:?}"
    );
    assert_texture_alias_recorded(stats, PostProcessGraphResourceNames::HZB_FURTHEST);
    let materialization = stats.last_graph_materialization_report;
    assert!(
        materialization.missing_texture_count == 0
            && materialization.missing_buffer_count == 0
            && materialization.missing_required_external_count == 0
            && materialization.stale_binding_count() == 0,
        "SSAO shared HZB graph should bind all required resources; report={materialization:?}"
    );
}

fn assert_graph_pass_executed(stats: &RenderStats, pass_name: &str) {
    assert!(
        stats
            .last_graph_executed_passes
            .iter()
            .any(|pass| pass == pass_name),
        "expected graph pass `{pass_name}` to execute; passes={:?}",
        stats.last_graph_executed_passes
    );
}

fn assert_graph_executor_executed(stats: &RenderStats, executor_id: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "expected graph executor `{executor_id}` to execute; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_texture_alias_recorded(stats: &RenderStats, logical_name: &str) {
    assert!(
        stats
            .last_graph_execution_alias_report
            .texture_aliases
            .iter()
            .any(|record| record.logical_name == logical_name),
        "expected texture alias report to include `{logical_name}`; aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

#[test]
fn clustered_lighting_quality_profile_schedules_cluster_pass_without_tile_tint() {
    let root = unique_temp_project_root("graphics_clustered_lighting");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsClusteredLighting",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_white.wgsl"),
        [0.55, 0.55, 0.55],
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_quad_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("flat_white.zmaterial"),
        "res://shaders/flat_white.wgsl",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/flat_white.zmaterial");
    let viewport_size = UVec2::new(160, 120);
    let lights = vec![RenderDirectionalLightSnapshot {
        node_id: 7,
        light_id: 7,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
        direction: Vec3::new(-0.65, -0.35, -1.0).normalize_or_zero(),
        color: Vec3::new(1.0, 0.48, 0.2),
        intensity: 3.5,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: None,
    }];
    let snapshot = build_snapshot(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model,
            mesh: None,
            material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
                ..Default::default()
            },
        }],
        lights,
        viewport_size,
    );

    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let clustered_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            clustered_viewport,
            RenderQualityProfile::new("clustered-on")
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let clustered_frame = submit_snapshot(&server, clustered_viewport, snapshot.clone());
    let clustered_stats = server.query_stats().unwrap();

    let flat_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            flat_viewport,
            RenderQualityProfile::new("clustered-off")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let flat_frame = submit_snapshot(&server, flat_viewport, snapshot);
    let flat_stats = server.query_stats().unwrap();

    let has_clustered_feature = |features: &[String]| {
        features
            .iter()
            .any(|feature| feature == "clustered_lighting")
    };
    let has_clustered_executor = |executor_ids: &[String]| {
        executor_ids
            .iter()
            .any(|executor_id| executor_id == "lighting.light-grid")
    };

    assert!(
        has_clustered_feature(&clustered_stats.last_effective_features),
        "clustered profile should keep clustered lighting in the compiled feature set"
    );
    assert!(
        has_clustered_executor(&clustered_stats.last_graph_executed_executor_ids),
        "clustered profile should execute the clustered light-list pass"
    );
    assert!(
        !has_clustered_feature(&flat_stats.last_effective_features),
        "flat profile should remove clustered lighting from the compiled feature set"
    );
    assert!(
        !has_clustered_executor(&flat_stats.last_graph_executed_executor_ids),
        "flat profile should not execute the clustered light-list pass"
    );

    let clustered_red = average_channel(&clustered_frame.rgba, 0);
    let flat_red = average_channel(&flat_frame.rgba, 0);
    let red_delta = (clustered_red - flat_red).abs();
    assert!(
        red_delta <= 1.0,
        "clustered light-list buffer should not directly tint final color; clustered red={clustered_red:.2}, flat red={flat_red:.2}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path() {
    let root = unique_temp_project_root("graphics_deferred_runtime");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GraphicsDeferredRuntime",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_flat_color_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("flat_green.wgsl"),
        [0.0, 1.0, 0.0],
    );
    write_solid_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("white.png"),
        [255, 255, 255, 255],
    );
    write_quad_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("quad.obj"),
    );
    write_material_with_base_color_and_texture(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("forward_green.zmaterial"),
        "res://shaders/flat_green.wgsl",
        [1.0, 0.08, 0.08, 1.0],
        "res://textures/white.png",
    );
    write_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
        "res://materials/forward_green.zmaterial",
    );

    let asset_manager = project_asset_manager_with_first_wave_plugin_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
    let material = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/forward_green.zmaterial",
    );
    let viewport_size = UVec2::new(160, 120);
    let snapshot = build_snapshot(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model,
            mesh: None,
            material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
                ..Default::default()
            },
        }],
        Vec::new(),
        viewport_size,
    );

    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let forward_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(forward_viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            forward_viewport,
            RenderQualityProfile::new("forward-clean")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let forward_frame = submit_snapshot(&server, forward_viewport, snapshot.clone());

    let deferred_viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(deferred_viewport, RenderPipelineHandle::new(2))
        .unwrap();
    server
        .set_quality_profile(
            deferred_viewport,
            RenderQualityProfile::new("deferred-clean")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .unwrap();
    let deferred_frame = submit_snapshot(&server, deferred_viewport, snapshot);
    let deferred_stats = server.query_stats().unwrap();
    for executor_id in [
        "deferred.depth-prepass",
        "deferred.gbuffer",
        "lighting.deferred",
        "mesh.transparent",
    ] {
        assert!(
            deferred_stats
                .last_graph_executed_executor_ids
                .contains(&executor_id.to_string()),
            "deferred submit should execute graph executor `{executor_id}`; executed={:?}",
            deferred_stats.last_graph_executed_executor_ids
        );
    }

    let sample_origin = UVec2::new(viewport_size.x / 4, viewport_size.y / 4);
    let sample_size = UVec2::new(viewport_size.x / 2, viewport_size.y / 2);
    let forward_red = average_channel_in_region(&forward_frame, sample_origin, sample_size, 0);
    let forward_green = average_channel_in_region(&forward_frame, sample_origin, sample_size, 1);
    let deferred_red = average_channel_in_region(&deferred_frame, sample_origin, sample_size, 0);
    let deferred_green = average_channel_in_region(&deferred_frame, sample_origin, sample_size, 1);

    assert!(
        forward_green > forward_red + 25.0,
        "forward baseline should remain project-shader green; red={forward_red:.2}, green={forward_green:.2}"
    );
    assert!(
        deferred_red > deferred_green + 20.0,
        "deferred runtime should shade through GBuffer material decode instead of the project shader; red={deferred_red:.2}, green={deferred_green:.2}"
    );

    let _ = fs::remove_dir_all(root);
}
