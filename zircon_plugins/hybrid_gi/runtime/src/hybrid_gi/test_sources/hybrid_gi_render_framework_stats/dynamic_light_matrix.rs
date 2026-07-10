use std::fmt::Write as _;

use zircon_runtime::core::framework::render::{
    RenderPipelineHandle, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};

use super::*;

const MATRIX_COLUMNS: usize = 4;
const MATRIX_ROWS: usize = 2;
const MATRIX_SEPARATOR: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DynamicLightCase {
    Directional,
    Point,
    Spot,
    Emissive,
}

impl DynamicLightCase {
    const ALL: [Self; MATRIX_COLUMNS] =
        [Self::Directional, Self::Point, Self::Spot, Self::Emissive];

    fn name(self) -> &'static str {
        match self {
            Self::Directional => "directional",
            Self::Point => "point",
            Self::Spot => "spot",
            Self::Emissive => "emissive",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DynamicLightPanelStats {
    graph_pass_count: usize,
    cache_entry_count: usize,
    surface_cache_page_count: usize,
    trace_tile_count: usize,
    screen_probe_count: usize,
    voxel_clipmap_count: usize,
}

struct DynamicLightPanel {
    pipeline_name: &'static str,
    light_case: DynamicLightCase,
    frame: CapturedFrame,
    metrics: FrameMetrics,
    center_rgb: [f32; 3],
    stats: DynamicLightPanelStats,
}

#[test]
#[ignore]
fn export_hybrid_gi_scene_representation_only_forward_deferred_wgpu_png() {
    let assets = material_texture_capture_test_assets();
    let _cleanup = TempProjectCleanup(assets.root.clone());
    let asset_manager = assets.asset_manager.clone();
    let model = model_handle(&asset_manager);
    let white_material = assets.flat_normal;
    let emissive_material = assets.emissive_warm;
    let viewport_size = UVec2::new(192, 128);
    let framework = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let pipelines = [
        ("forward_plus", RenderPipelineHandle::new(1)),
        ("deferred", RenderPipelineHandle::new(2)),
    ];
    let mut panels = Vec::with_capacity(MATRIX_ROWS * MATRIX_COLUMNS);

    for (pipeline_name, pipeline) in pipelines {
        for light_case in DynamicLightCase::ALL {
            let extract = dynamic_light_extract(
                viewport_size,
                model.clone(),
                white_material.clone(),
                emissive_material.clone(),
                light_case,
            );
            let viewport = framework
                .create_viewport(RenderViewportDescriptor::new(viewport_size))
                .unwrap();
            framework.set_pipeline_asset(viewport, pipeline).unwrap();
            framework
                .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
                .unwrap();
            framework
                .submit_frame_extract(viewport, extract.clone())
                .unwrap();
            framework
                .submit_frame_extract(viewport, extract.clone())
                .unwrap();
            framework.submit_frame_extract(viewport, extract).unwrap();

            let render_stats = framework.query_stats().unwrap();
            let frame = framework
                .capture_frame(viewport)
                .unwrap()
                .expect("dynamic-light HGI Wgpu frame should be capturable");
            let metrics = frame_metrics(&frame);
            let center_rgb = [
                average_region_channel(&frame.rgba, viewport_size, 0, 0.2, 0.8, 0.2, 0.8),
                average_region_channel(&frame.rgba, viewport_size, 1, 0.2, 0.8, 0.2, 0.8),
                average_region_channel(&frame.rgba, viewport_size, 2, 0.2, 0.8, 0.2, 0.8),
            ];
            let stats = DynamicLightPanelStats {
                graph_pass_count: render_stats.last_hybrid_gi_graph_executed_pass_count,
                cache_entry_count: render_stats.last_hybrid_gi_cache_entry_count,
                surface_cache_page_count: render_stats
                    .last_hybrid_gi_surface_cache_resident_page_count,
                trace_tile_count: render_stats.last_hybrid_gi_probe_trace_tile_count,
                screen_probe_count: render_stats.last_hybrid_gi_scene_screen_probe_count,
                voxel_clipmap_count: render_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            };
            assert_dynamic_light_panel(light_case, metrics, center_rgb, stats);
            panels.push(DynamicLightPanel {
                pipeline_name,
                light_case,
                frame,
                metrics,
                center_rgb,
                stats,
            });
        }
    }

    for column in 0..MATRIX_COLUMNS {
        assert_pipeline_parity(&panels[column], &panels[MATRIX_COLUMNS + column]);
    }

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_dynamic_light_matrix_png(output_dir.join(DYNAMIC_LIGHT_MATRIX_WGPU_PNG), &panels);
    fs::write(
        output_dir.join(DYNAMIC_LIGHT_MATRIX_WGPU_REPORT),
        dynamic_light_matrix_report(&panels),
    )
    .unwrap();
}

fn dynamic_light_extract(
    viewport_size: UVec2,
    model: ResourceHandle<ModelMarker>,
    white_material: ResourceHandle<MaterialMarker>,
    emissive_material: ResourceHandle<MaterialMarker>,
    light_case: DynamicLightCase,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ortho_size: 6.0,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);
    let material = if light_case == DynamicLightCase::Emissive {
        emissive_material
    } else {
        white_material
    };
    let layer_mask = RenderLayerSet::from_scene_schema_v1_mask(u32::MAX);
    let directional_lights = if light_case == DynamicLightCase::Directional {
        vec![RenderDirectionalLightSnapshot {
            node_id: 901,
            light_id: 901,
            layer_mask: layer_mask.clone(),
            direction: Vec3::new(0.0, 0.0, -1.0),
            color: Vec3::new(1.0, 0.05, 0.02),
            intensity: 6.0,
            shadow: None,
        }]
    } else {
        Vec::new()
    };
    let point_lights = if light_case == DynamicLightCase::Point {
        vec![RenderPointLightSnapshot {
            node_id: 902,
            light_id: 902,
            layer_mask: layer_mask.clone(),
            position: Vec3::new(0.0, 0.0, 1.5),
            color: Vec3::new(0.02, 1.0, 0.06),
            intensity: 10.0,
            range: 5.0,
            shadow: None,
        }]
    } else {
        Vec::new()
    };
    let spot_lights = if light_case == DynamicLightCase::Spot {
        vec![RenderSpotLightSnapshot {
            node_id: 903,
            light_id: 903,
            layer_mask,
            position: Vec3::new(0.0, 0.0, 1.75),
            direction: Vec3::new(0.0, 0.0, -1.0),
            color: Vec3::new(0.02, 0.06, 1.0),
            intensity: 12.0,
            range: 5.0,
            inner_angle_radians: 0.2,
            outer_angle_radians: 0.7,
            shadow: None,
        }]
    } else {
        Vec::new()
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![mesh(11, model, material, Vec3::ZERO, 2.0)],
            directional_lights,
            point_lights,
            spot_lights,
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        environment: EnvironmentExtract::disabled(),
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
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: RenderHybridGiQuality::High,
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: RenderHybridGiDebugView::None,
    });
    extract
}

fn assert_dynamic_light_panel(
    light_case: DynamicLightCase,
    metrics: FrameMetrics,
    center_rgb: [f32; 3],
    stats: DynamicLightPanelStats,
) {
    assert!(metrics.visible_pixels > 0 && metrics.max_luma > 8.0);
    assert_eq!(stats.graph_pass_count, 4);
    assert!(stats.surface_cache_page_count >= 1);
    assert!(stats.trace_tile_count >= 1);
    assert!(stats.screen_probe_count >= 1);
    assert!(stats.voxel_clipmap_count >= 1);
    match light_case {
        DynamicLightCase::Directional | DynamicLightCase::Emissive => assert!(
            center_rgb[0] > center_rgb[1] + 0.1 && center_rgb[0] > center_rgb[2] + 0.1,
            "expected red-dominant {light_case:?} HGI radiance, got {center_rgb:?}"
        ),
        DynamicLightCase::Point => assert!(
            center_rgb[1] > center_rgb[0] + 0.1 && center_rgb[1] > center_rgb[2] + 0.1,
            "expected green-dominant point-light HGI radiance, got {center_rgb:?}"
        ),
        DynamicLightCase::Spot => assert!(
            center_rgb[2] > center_rgb[0] + 0.1 && center_rgb[2] > center_rgb[1] + 0.1,
            "expected blue-dominant spot-light HGI radiance, got {center_rgb:?}"
        ),
    }
}

fn assert_pipeline_parity(forward: &DynamicLightPanel, deferred: &DynamicLightPanel) {
    assert_eq!(forward.light_case, deferred.light_case);
    let average_rgb_delta = forward
        .center_rgb
        .into_iter()
        .zip(deferred.center_rgb)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / 3.0;
    assert!(
        average_rgb_delta <= 8.0,
        "Forward+/Deferred HGI parity exceeded for {:?}: forward={:?}, deferred={:?}, average_delta={average_rgb_delta:.2}",
        forward.light_case,
        forward.center_rgb,
        deferred.center_rgb,
    );
}

fn dynamic_light_matrix_report(panels: &[DynamicLightPanel]) -> String {
    let mut report = format!(
        "png={DYNAMIC_LIGHT_MATRIX_WGPU_PNG}\nlayout=forward_plus_directional,point,spot,emissive_then_deferred_directional,point,spot,emissive\nwidth={}\nheight={}\nproduct_debug_view=none\npreview_direct_lighting=disabled\npublic_hybrid_gi_extract_contract=settings_quality_trace_card_voxel_debug_only\nauthored_probe_trace_surface=hard_cut\nruntime_scene_source=HybridGiSceneRepresentation_to_RenderHybridGiPreparedFrame\nhgi_input_contract=shared_HybridGiInputSet_and_shared_composite\ntrace_radiance_contract=authoritative_surface_cache_and_voxel_radiance_without_scene_wide_relighting\nlumen_reference=LumenSceneCardCapture_direct_lighting_to_SurfaceCache_then_ScreenProbeGather_composite\n",
        panels[0].frame.width * MATRIX_COLUMNS as u32 + (MATRIX_COLUMNS as u32 - 1),
        panels[0].frame.height * MATRIX_ROWS as u32 + (MATRIX_ROWS as u32 - 1),
    );
    for panel in panels {
        writeln!(
            report,
            "{}_{}_generation={}\n{}_{}_visible_pixels={}\n{}_{}_max_luma={:.2}\n{}_{}_center_rgb={:.2},{:.2},{:.2}\n{}_{}_graph_pass_count={}\n{}_{}_cache_entry_count={}\n{}_{}_surface_cache_page_count={}\n{}_{}_trace_tile_count={}\n{}_{}_screen_probe_count={}\n{}_{}_voxel_clipmap_count={}",
            panel.pipeline_name,
            panel.light_case.name(),
            panel.frame.generation,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.metrics.visible_pixels,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.metrics.max_luma,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.center_rgb[0],
            panel.center_rgb[1],
            panel.center_rgb[2],
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.graph_pass_count,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.cache_entry_count,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.surface_cache_page_count,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.trace_tile_count,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.screen_probe_count,
            panel.pipeline_name,
            panel.light_case.name(),
            panel.stats.voxel_clipmap_count,
        )
        .unwrap();
    }
    report
}

fn write_dynamic_light_matrix_png(path: PathBuf, panels: &[DynamicLightPanel]) {
    assert_eq!(panels.len(), MATRIX_ROWS * MATRIX_COLUMNS);
    let panel_width = panels[0].frame.width;
    let panel_height = panels[0].frame.height;
    let output_width = panel_width * MATRIX_COLUMNS as u32 + MATRIX_SEPARATOR * 3;
    let output_height = panel_height * MATRIX_ROWS as u32 + MATRIX_SEPARATOR;
    let mut rgba = vec![255_u8; (output_width * output_height * 4) as usize];

    for (index, panel) in panels.iter().enumerate() {
        assert_eq!(panel.frame.width, panel_width);
        assert_eq!(panel.frame.height, panel_height);
        let column = index % MATRIX_COLUMNS;
        let row = index / MATRIX_COLUMNS;
        let destination_x = column as u32 * (panel_width + MATRIX_SEPARATOR);
        let destination_y = row as u32 * (panel_height + MATRIX_SEPARATOR);
        for y in 0..panel_height as usize {
            let source_start = y * panel_width as usize * 4;
            let destination_start =
                ((destination_y as usize + y) * output_width as usize + destination_x as usize) * 4;
            let row_len = panel_width as usize * 4;
            rgba[destination_start..destination_start + row_len]
                .copy_from_slice(&panel.frame.rgba[source_start..source_start + row_len]);
        }
    }

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, output_height, rgba)
        .expect("dynamic-light matrix rgba payload should match its dimensions");
    image.save_with_format(path, ImageFormat::Png).unwrap();
}
