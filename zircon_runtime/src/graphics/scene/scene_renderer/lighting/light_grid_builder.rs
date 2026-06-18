use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    GpuLightData, GpuLightType, ProjectionMode, ViewportCameraSnapshot,
};
use crate::core::math::{view_matrix, Mat4, UVec2, Vec3};

pub(crate) const LIGHT_GRID_INITIAL_TILE_SIZE_PX: u32 = 8;
pub(crate) const LIGHT_GRID_MAX_ZBIN_WORDS: u32 = 4096;
pub(crate) const LIGHT_GRID_MAX_TILE_WORDS: u32 = 8192;
pub(crate) const LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES: usize = 128;
const ZBIN_HEADER_WORDS: u32 = 2;
pub(crate) const LIGHT_GRID_EMPTY_ZBIN_HEADER: u32 = 0x0000_FFFF;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct LightGridParams {
    pub(crate) world_to_view: [[f32; 4]; 4],
    pub(crate) zbin_scale: f32,
    pub(crate) zbin_offset: f32,
    pub(crate) bin_count: u32,
    pub(crate) words_per_tile: u32,
    pub(crate) tile_resolution: [u32; 2],
    pub(crate) tile_size_px: u32,
    pub(crate) light_count: u32,
    pub(crate) projection_mode: u32,
    // WGSL uniform layout aligns the trailing vec3 padding member to 16 bytes and rounds the
    // complete struct size to 128 bytes.
    pub(crate) _uniform_padding: [u32; 7],
}

impl LightGridParams {
    pub(crate) fn disabled() -> Self {
        Self {
            world_to_view: Mat4::IDENTITY.to_cols_array_2d(),
            bin_count: 1,
            words_per_tile: 1,
            tile_resolution: [1, 1],
            tile_size_px: 1,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LightGridProjection {
    Perspective,
    Orthographic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LightGridViewInfo {
    pub(crate) viewport_size: UVec2,
    pub(crate) world_to_view: Mat4,
    pub(crate) view_to_clip: Mat4,
    pub(crate) projection: LightGridProjection,
    pub(crate) z_near: f32,
    pub(crate) z_far: f32,
}

impl LightGridViewInfo {
    pub(crate) fn from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Self {
        let viewport_size = UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1));
        let aspect_ratio = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
        let z_near = camera.z_near.max(0.001);
        let z_far = camera.z_far.max(z_near + 0.001);
        let projection = match camera.projection_mode {
            ProjectionMode::Perspective => LightGridProjection::Perspective,
            ProjectionMode::Orthographic => LightGridProjection::Orthographic,
        };
        let view_to_clip = match projection {
            LightGridProjection::Perspective => {
                Mat4::perspective_rh(camera.fov_y_radians, aspect_ratio.max(0.001), z_near, z_far)
            }
            LightGridProjection::Orthographic => {
                let half_height = (camera.ortho_size * 0.5).max(0.001);
                let half_width = half_height * aspect_ratio.max(0.001);
                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    z_near,
                    z_far,
                )
            }
        };

        Self {
            viewport_size,
            world_to_view: view_matrix(camera.transform),
            view_to_clip,
            projection,
            z_near,
            z_far,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LightGridStats {
    pub(crate) light_count: u32,
    pub(crate) tile_count: u32,
    pub(crate) zbin_count: u32,
    pub(crate) non_empty_tile_count: u32,
    pub(crate) non_empty_zbin_count: u32,
    pub(crate) non_empty_cluster_count: u32,
    pub(crate) peak_lights_per_cluster: u32,
    pub(crate) average_lights_per_cluster: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LightGridCpuOutput {
    pub(crate) zbins: Vec<u32>,
    pub(crate) tile_masks: Vec<u32>,
    pub(crate) params: LightGridParams,
    pub(crate) stats: LightGridStats,
}

pub(crate) fn build_light_grid(
    lights: &[GpuLightData],
    view: &LightGridViewInfo,
) -> LightGridCpuOutput {
    let light_count = lights.len().min(u16::MAX as usize) as u32;
    let words_per_tile = light_count.div_ceil(32).max(1);
    let tile_size_px = tile_size_for_budget(view.viewport_size, words_per_tile);
    let tile_resolution = tile_resolution_for_size(view.viewport_size, tile_size_px);
    let bin_stride = ZBIN_HEADER_WORDS + words_per_tile;
    let bin_count = (LIGHT_GRID_MAX_ZBIN_WORDS / bin_stride).max(1);
    let zbin_scale = zbin_scale(view, bin_count);
    let zbin_offset = zbin_offset(view, zbin_scale);
    let params = LightGridParams {
        world_to_view: view.world_to_view.to_cols_array_2d(),
        zbin_scale,
        zbin_offset,
        bin_count,
        words_per_tile,
        tile_resolution: [tile_resolution.x, tile_resolution.y],
        tile_size_px,
        light_count,
        projection_mode: match view.projection {
            LightGridProjection::Perspective => 0,
            LightGridProjection::Orthographic => 1,
        },
        _uniform_padding: [0; 7],
    };
    let mut zbins = vec![0; bin_count as usize * bin_stride as usize];
    for bin in 0..bin_count {
        zbins[bin_base(bin, bin_stride)] = LIGHT_GRID_EMPTY_ZBIN_HEADER;
    }
    let tile_count = tile_resolution.x as usize * tile_resolution.y as usize;
    let mut tile_masks = vec![0; tile_count * words_per_tile as usize];
    let mut zbin_min_max: Vec<Option<(u32, u32)>> = vec![None; bin_count as usize];

    for (light_index, light) in lights.iter().take(light_count as usize).enumerate() {
        let Some(influence) = light_influence(light, view, &params) else {
            continue;
        };
        let light_index = light_index as u32;
        let word_index = light_index / 32;
        let bit = 1_u32 << (light_index % 32);
        for tile_y in influence.tile_min[1]..influence.tile_max_exclusive[1] {
            for tile_x in influence.tile_min[0]..influence.tile_max_exclusive[0] {
                let tile_word =
                    tile_word_index(tile_x, tile_y, tile_resolution, words_per_tile, word_index);
                tile_masks[tile_word] |= bit;
            }
        }
        for bin in influence.bin_min..=influence.bin_max {
            let word = bin_base(bin, bin_stride) + ZBIN_HEADER_WORDS as usize + word_index as usize;
            zbins[word] |= bit;
            let min_max = &mut zbin_min_max[bin as usize];
            *min_max = Some(match *min_max {
                Some((min_index, max_index)) => {
                    (min_index.min(light_index), max_index.max(light_index))
                }
                None => (light_index, light_index),
            });
        }
    }

    for (bin, min_max) in zbin_min_max.into_iter().enumerate() {
        if let Some((min_index, max_index)) = min_max {
            zbins[bin_base(bin as u32, bin_stride)] = encode_zbin_header(min_index, max_index);
        }
    }

    let stats = light_grid_stats(&zbins, &tile_masks, &params);

    LightGridCpuOutput {
        zbins,
        tile_masks,
        params,
        stats,
    }
}

fn tile_size_for_budget(viewport_size: UVec2, words_per_tile: u32) -> u32 {
    let mut tile_size = LIGHT_GRID_INITIAL_TILE_SIZE_PX;
    while tile_word_count(viewport_size, tile_size, words_per_tile) > LIGHT_GRID_MAX_TILE_WORDS {
        tile_size = tile_size.saturating_mul(2).max(tile_size + 1);
    }
    tile_size
}

fn tile_word_count(viewport_size: UVec2, tile_size: u32, words_per_tile: u32) -> u32 {
    let resolution = tile_resolution_for_size(viewport_size, tile_size);
    resolution
        .x
        .saturating_mul(resolution.y)
        .saturating_mul(words_per_tile)
}

fn tile_resolution_for_size(viewport_size: UVec2, tile_size: u32) -> UVec2 {
    UVec2::new(
        viewport_size.x.max(1).div_ceil(tile_size.max(1)),
        viewport_size.y.max(1).div_ceil(tile_size.max(1)),
    )
}

fn zbin_scale(view: &LightGridViewInfo, bin_count: u32) -> f32 {
    match view.projection {
        LightGridProjection::Perspective => {
            let log_range = view.z_far.log2() - view.z_near.log2();
            bin_count as f32 / log_range.max(0.001)
        }
        LightGridProjection::Orthographic => {
            bin_count as f32 / (view.z_far - view.z_near).max(0.001)
        }
    }
}

fn zbin_offset(view: &LightGridViewInfo, zbin_scale: f32) -> f32 {
    match view.projection {
        LightGridProjection::Perspective => -view.z_near.log2() * zbin_scale,
        LightGridProjection::Orthographic => -view.z_near * zbin_scale,
    }
}

fn zbin_index(view_z: f32, view: &LightGridViewInfo, params: &LightGridParams) -> u32 {
    let clamped_z = view_z.clamp(view.z_near, view.z_far);
    let raw = match view.projection {
        LightGridProjection::Perspective => {
            clamped_z.log2() * params.zbin_scale + params.zbin_offset
        }
        LightGridProjection::Orthographic => clamped_z * params.zbin_scale + params.zbin_offset,
    };
    raw.floor().max(0.0).min((params.bin_count - 1) as f32) as u32
}

fn light_influence(
    light: &GpuLightData,
    view: &LightGridViewInfo,
    params: &LightGridParams,
) -> Option<LightInfluence> {
    match light_type(light) {
        Some(GpuLightType::Directional) => Some(LightInfluence {
            tile_min: [0, 0],
            tile_max_exclusive: params.tile_resolution,
            bin_min: 0,
            bin_max: params.bin_count - 1,
        }),
        Some(GpuLightType::Point | GpuLightType::Spot | GpuLightType::Rect) => {
            let world_position = Vec3::new(
                light.position_range[0],
                light.position_range[1],
                light.position_range[2],
            );
            let radius = light.position_range[3].max(0.0);
            if radius <= 0.0 {
                return None;
            }
            sphere_influence(world_position, radius, view, params)
        }
        None => None,
    }
}

fn light_type(light: &GpuLightData) -> Option<GpuLightType> {
    match light.direction_type[3].to_bits() {
        0 => Some(GpuLightType::Directional),
        1 => Some(GpuLightType::Point),
        2 => Some(GpuLightType::Spot),
        3 => Some(GpuLightType::Rect),
        _ => None,
    }
}

fn sphere_influence(
    world_position: Vec3,
    radius: f32,
    view: &LightGridViewInfo,
    params: &LightGridParams,
) -> Option<LightInfluence> {
    let view_position = view.world_to_view.transform_point3(world_position);
    let view_z = -view_position.z;
    if !view_z.is_finite() || view_z + radius < view.z_near || view_z - radius > view.z_far {
        return None;
    }
    let tile_rect = sphere_tile_rect(view_position, radius, view, params)?;
    let bin_min = zbin_index((view_z - radius).max(view.z_near), view, params);
    let bin_max = zbin_index((view_z + radius).min(view.z_far), view, params).max(bin_min);
    Some(LightInfluence {
        tile_min: tile_rect.tile_min,
        tile_max_exclusive: tile_rect.tile_max_exclusive,
        bin_min,
        bin_max,
    })
}

fn sphere_tile_rect(
    view_position: Vec3,
    radius: f32,
    view: &LightGridViewInfo,
    params: &LightGridParams,
) -> Option<TileRect> {
    let clip = view.view_to_clip * view_position.extend(1.0);
    if !clip.w.is_finite() || clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    let viewport = view.viewport_size;
    let center_px = [
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (0.5 - ndc.y * 0.5) * viewport.y as f32,
    ];
    let radius_px = projected_radius_px(view_position, radius, view);
    let min_px = [center_px[0] - radius_px[0], center_px[1] - radius_px[1]];
    let max_px = [center_px[0] + radius_px[0], center_px[1] + radius_px[1]];
    if max_px[0] <= 0.0
        || max_px[1] <= 0.0
        || min_px[0] >= viewport.x as f32
        || min_px[1] >= viewport.y as f32
    {
        return None;
    }

    let min_x = min_px[0].floor().max(0.0) as u32;
    let min_y = min_px[1].floor().max(0.0) as u32;
    let max_x = max_px[0].ceil().min(viewport.x as f32).max(1.0) as u32;
    let max_y = max_px[1].ceil().min(viewport.y as f32).max(1.0) as u32;
    let tile_size = params.tile_size_px.max(1);
    Some(TileRect {
        tile_min: [min_x / tile_size, min_y / tile_size],
        tile_max_exclusive: [
            (max_x.saturating_sub(1) / tile_size + 1).min(params.tile_resolution[0]),
            (max_y.saturating_sub(1) / tile_size + 1).min(params.tile_resolution[1]),
        ],
    })
}

fn projected_radius_px(view_position: Vec3, radius: f32, view: &LightGridViewInfo) -> [f32; 2] {
    let viewport = view.viewport_size;
    match view.projection {
        LightGridProjection::Perspective => {
            let z_for_radius = (-view_position.z - radius).max(view.z_near).max(0.001);
            let cols = view.view_to_clip.to_cols_array();
            [
                radius * cols[0].abs() / z_for_radius * viewport.x as f32 * 0.5,
                radius * cols[5].abs() / z_for_radius * viewport.y as f32 * 0.5,
            ]
        }
        LightGridProjection::Orthographic => {
            let cols = view.view_to_clip.to_cols_array();
            [
                radius * cols[0].abs() * viewport.x as f32 * 0.5,
                radius * cols[5].abs() * viewport.y as f32 * 0.5,
            ]
        }
    }
}

fn light_grid_stats(zbins: &[u32], tile_masks: &[u32], params: &LightGridParams) -> LightGridStats {
    let words_per_tile = params.words_per_tile as usize;
    let bin_stride = (ZBIN_HEADER_WORDS + params.words_per_tile) as usize;
    let tile_count = params.tile_resolution[0] as usize * params.tile_resolution[1] as usize;
    let mut non_empty_tile_count = 0;
    for tile in 0..tile_count {
        let base = tile * words_per_tile;
        if tile_masks[base..base + words_per_tile]
            .iter()
            .any(|word| *word != 0)
        {
            non_empty_tile_count += 1;
        }
    }

    let mut non_empty_zbin_count = 0;
    let mut non_empty_cluster_count = 0;
    let mut peak_lights_per_cluster = 0;
    let mut total_cluster_lights = 0_u64;
    for bin in 0..params.bin_count as usize {
        let zbin_base = bin * bin_stride + ZBIN_HEADER_WORDS as usize;
        let zbin_words = &zbins[zbin_base..zbin_base + words_per_tile];
        if zbin_words.iter().any(|word| *word != 0) {
            non_empty_zbin_count += 1;
        }
        for tile in 0..tile_count {
            let tile_base = tile * words_per_tile;
            let mut cluster_lights = 0;
            for word in 0..words_per_tile {
                cluster_lights += (tile_masks[tile_base + word] & zbin_words[word]).count_ones();
            }
            if cluster_lights > 0 {
                non_empty_cluster_count += 1;
            }
            peak_lights_per_cluster = peak_lights_per_cluster.max(cluster_lights);
            total_cluster_lights += u64::from(cluster_lights);
        }
    }

    let cluster_count = tile_count.max(1) as f32 * params.bin_count.max(1) as f32;
    LightGridStats {
        light_count: params.light_count,
        tile_count: tile_count as u32,
        zbin_count: params.bin_count,
        non_empty_tile_count,
        non_empty_zbin_count,
        non_empty_cluster_count,
        peak_lights_per_cluster,
        average_lights_per_cluster: total_cluster_lights as f32 / cluster_count,
    }
}

fn encode_zbin_header(min_index: u32, max_index: u32) -> u32 {
    min_index.min(u16::MAX as u32) | (max_index.min(u16::MAX as u32) << 16)
}

#[cfg(test)]
fn decode_zbin_header(header: u32) -> Option<(u32, u32)> {
    if header == LIGHT_GRID_EMPTY_ZBIN_HEADER {
        return None;
    }
    Some((header & 0xFFFF, header >> 16))
}

fn bin_base(bin: u32, bin_stride: u32) -> usize {
    bin as usize * bin_stride as usize
}

fn tile_word_index(
    tile_x: u32,
    tile_y: u32,
    tile_resolution: UVec2,
    words_per_tile: u32,
    word_index: u32,
) -> usize {
    ((tile_y * tile_resolution.x + tile_x) * words_per_tile + word_index) as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TileRect {
    tile_min: [u32; 2],
    tile_max_exclusive: [u32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LightInfluence {
    tile_min: [u32; 2],
    tile_max_exclusive: [u32; 2],
    bin_min: u32,
    bin_max: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES,
    };
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
        naga::front::wgsl::parse_str(include_str!("shaders/zr_light_grid.wgsl"))
            .expect("light grid WGSL include should parse");
    }

    fn test_view(viewport_size: UVec2) -> LightGridViewInfo {
        let camera = ViewportCameraSnapshot {
            transform: Transform::from_translation(Vec3::ZERO).with_rotation(Quat::IDENTITY),
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
        };
        LightGridViewInfo::from_camera(&camera, viewport_size)
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
}
