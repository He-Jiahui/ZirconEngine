use super::*;
use crate::core::framework::render::{DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES};
use crate::core::math::{Quat, Transform};

#[test]
fn light_grid_params_cpu_layout_matches_wgsl_uniform_size() {
    assert_eq!(
        std::mem::size_of::<LightGridParams>(),
        LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES
    );
}

#[test]
fn light_grid_builder_marks_directional_light_across_all_tiles_and_bins() {
    let view = test_view(UVec2::new(16, 16));
    let output = build_light_grid(&[directional_light()], &view);

    assert_eq!(output.params.tile_resolution, [2, 2]);
    assert_eq!(output.params.words_per_tile, 1);
    assert!(output.tile_masks.iter().all(|word| *word == 1));
    for bin in 0..output.params.bin_count {
        let base = bin_base(bin, ZBIN_HEADER_WORDS + output.params.words_per_tile);
        assert_eq!(decode_zbin_header(output.zbins[base]), Some((0, 0)));
        assert_eq!(output.zbins[base + ZBIN_HEADER_WORDS as usize], 1);
    }
    assert_eq!(output.stats.non_empty_tile_count, 4);
    assert_eq!(output.stats.non_empty_zbin_count, output.params.bin_count);
    assert_eq!(output.stats.peak_lights_per_cluster, 1);
}

#[test]
fn light_grid_builder_culls_point_light_to_screen_and_depth_ranges() {
    let view = test_view(UVec2::new(64, 64));
    let lights = [
        point_light(Vec3::new(-1.25, 0.0, -4.0), 0.4),
        point_light(Vec3::new(100.0, 0.0, -4.0), 0.4),
    ];
    let output = build_light_grid(&lights, &view);

    let lit_tiles = lit_tile_indices(&output);
    assert!(!lit_tiles.is_empty());
    assert!(lit_tiles.len() < output.stats.tile_count as usize);
    assert!(output.tile_masks.iter().any(|word| *word == 1));
    assert!(output.tile_masks.iter().all(|word| *word & 0b10 == 0));
    assert!(output.stats.non_empty_zbin_count > 0);
    assert!(output.stats.non_empty_zbin_count < output.params.bin_count);
    assert_eq!(output.stats.peak_lights_per_cluster, 1);
}

#[test]
fn light_grid_view_uses_canonical_orthographic_half_height() {
    let viewport_size = UVec2::new(100, 100);
    let mut camera = test_camera(viewport_size);
    camera.projection_mode = ProjectionMode::Orthographic;
    camera.ortho_size = 10.0;

    let view = LightGridViewInfo::from_camera(&camera, viewport_size);
    let projection = view.view_to_clip.to_cols_array();

    assert!((projection[5] - 0.1).abs() <= f32::EPSILON);
}

#[test]
fn light_grid_view_reuses_shared_camera_projection_owner() {
    let viewport_size = UVec2::new(100, 100);
    let camera = test_camera(viewport_size);
    let view = LightGridViewInfo::from_camera(&camera, viewport_size);

    assert_eq!(
        view.view_to_clip,
        ViewProjectionMatrixPair::projection_from_camera(&camera, viewport_size)
    );
}

#[test]
fn light_grid_builder_keeps_camera_inside_point_light_conservative() {
    let view = test_view(UVec2::new(64, 64));
    let output = build_light_grid(&[point_light(Vec3::new(0.0, 0.0, 0.05), 1.0)], &view);

    assert!(output.tile_masks.iter().all(|word| *word == 1));
    assert!(output.stats.non_empty_zbin_count > 0);
}

#[test]
fn light_grid_builder_keeps_near_crossing_sphere_with_behind_camera_center() {
    let view = test_view(UVec2::new(64, 64));
    let output = build_light_grid(&[point_light(Vec3::new(1.0, 0.0, 1.0), 1.2)], &view);

    assert!(output.tile_masks.iter().all(|word| *word == 1));
    assert!(output.stats.non_empty_zbin_count > 0);
}

#[test]
fn light_grid_builder_rejects_sphere_fully_behind_camera() {
    let view = test_view(UVec2::new(64, 64));
    let output = build_light_grid(&[point_light(Vec3::new(0.0, 0.0, 1.0), 0.5)], &view);

    assert!(output.tile_masks.iter().all(|word| *word == 0));
    assert_eq!(output.stats.non_empty_zbin_count, 0);
}

#[test]
fn light_grid_builder_increases_tile_size_to_fit_mask_budget() {
    let view = test_view(UVec2::new(4096, 4096));
    let lights = vec![directional_light(); 1024];
    let output = build_light_grid(&lights, &view);

    assert!(output.params.tile_size_px > LIGHT_GRID_INITIAL_TILE_SIZE_PX);
    assert!(
        output.params.tile_resolution[0]
            * output.params.tile_resolution[1]
            * output.params.words_per_tile
            <= LIGHT_GRID_MAX_TILE_WORDS
    );
    assert_eq!(output.params.words_per_tile, 32);
}

#[test]
fn light_grid_builder_zbin_header_tracks_min_and_max_light_indices() {
    let view = test_view(UVec2::new(32, 32));
    let lights = [
        point_light(Vec3::new(0.0, 0.0, -2.0), 0.2),
        point_light(Vec3::new(0.0, 0.0, -8.0), 0.2),
    ];
    let output = build_light_grid(&lights, &view);
    let bin_stride = ZBIN_HEADER_WORDS + output.params.words_per_tile;
    let non_empty_headers = (0..output.params.bin_count)
        .filter_map(|bin| {
            let header = output.zbins[bin_base(bin, bin_stride)];
            decode_zbin_header(header)
        })
        .collect::<Vec<_>>();

    assert!(non_empty_headers.iter().any(|header| *header == (0, 0)));
    assert!(non_empty_headers.iter().any(|header| *header == (1, 1)));
    assert!(!non_empty_headers.iter().any(|header| *header == (0, 1)));
}

#[test]
fn light_grid_shader_include_is_valid_wgsl() {
    naga::front::wgsl::parse_str(include_str!("../shaders/zr_light_grid.wgsl"))
        .expect("light grid WGSL include should parse");
}

fn test_view(viewport_size: UVec2) -> LightGridViewInfo {
    LightGridViewInfo::from_camera(&test_camera(viewport_size), viewport_size)
}

fn test_camera(viewport_size: UVec2) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::ZERO).with_rotation(Quat::IDENTITY),
        core_pipeline: Default::default(),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: 10.0,
        z_near: 0.1,
        z_far: 32.0,
        aspect_ratio: viewport_size.x as f32 / viewport_size.y.max(1) as f32,
        is_active: true,
        hdr: false,
        exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
        msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
        dynamic_resolution: Default::default(),
        temporal_jitter: Default::default(),
        projection_override: None,
    }
}

fn directional_light() -> GpuLightData {
    GpuLightData {
        direction_type: [0.0, -1.0, 0.0, GpuLightType::Directional.as_f32_bits()],
        ..GpuLightData::default()
    }
}

fn point_light(position: Vec3, range: f32) -> GpuLightData {
    GpuLightData {
        position_range: [position.x, position.y, position.z, range],
        direction_type: [0.0, 0.0, 0.0, GpuLightType::Point.as_f32_bits()],
        ..GpuLightData::default()
    }
}

fn lit_tile_indices(output: &LightGridCpuOutput) -> Vec<usize> {
    output
        .tile_masks
        .chunks(output.params.words_per_tile as usize)
        .enumerate()
        .filter_map(|(index, words)| words.iter().any(|word| *word != 0).then_some(index))
        .collect()
}
