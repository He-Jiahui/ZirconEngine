use std::path::Path;

use zircon_runtime::graphics::ViewportFrame;

use super::{
    luma, LEGACY_EQUIRECT_SAMPLE_COLUMNS, LEGACY_EQUIRECT_SAMPLE_ROWS, LEGACY_GRID_OFFSET_SAMPLES,
    LEGACY_GRID_SKY_SAMPLE_Y_MAX, LEGACY_GRID_SKY_SAMPLE_Y_MIN, PBR_MATRIX_CELL_SAMPLE_SIZE,
    PBR_MATRIX_DIMENSION, PBR_MATRIX_ORTHO_SIZE, PBR_MATRIX_OUTPUT_SIZE,
    PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
};

pub(crate) fn assert_real_hdri_reflection_response(frame: &ViewportFrame) {
    let frame = FramePixels::from_viewport(frame);
    assert_real_hdri_reflection_response_from_pixels(&frame);
}

pub(crate) fn assert_saved_real_hdri_reflection_response(path: &Path) {
    let image = image::open(path)
        .unwrap_or_else(|error| panic!("read saved real HDRI PBR screenshot {path:?}: {error}"))
        .to_rgba8();
    assert_eq!(
        (image.width(), image.height()),
        (PBR_MATRIX_OUTPUT_SIZE.x, PBR_MATRIX_OUTPUT_SIZE.y),
        "saved HDRI PBR screenshot should keep the accepted matrix dimensions"
    );
    let frame = FramePixels::from_rgba_bytes(image.width(), image.height(), image.as_raw());
    assert_real_hdri_reflection_response_from_pixels(&frame);
}

pub(crate) fn assert_saved_pmrem_mip_diagnostic_blur_response(path: &Path) {
    let image = image::open(path)
        .unwrap_or_else(|error| {
            panic!("read saved PMREM mip diagnostic screenshot {path:?}: {error}")
        })
        .to_rgba8();
    assert_eq!(
        image.width() % PMREM_MIP_DIAGNOSTIC_TILE_SIZE,
        0,
        "PMREM mip diagnostic width should be a whole number of tiles"
    );
    assert_eq!(
        image.height(),
        PMREM_MIP_DIAGNOSTIC_TILE_SIZE * 12,
        "PMREM mip diagnostic should contain six source rows and six PMREM rows"
    );
    let frame = FramePixels::from_rgba_bytes(image.width(), image.height(), image.as_raw());
    assert_pmrem_mip_diagnostic_blur_response(&frame);
}

fn assert_real_hdri_reflection_response_from_pixels(frame: &FramePixels<'_>) {
    let upper_sky = average_region_rgb(frame, 40, 32, 96, 96);
    let lower_sky = average_region_rgb(frame, 40, frame.height.saturating_sub(128), 96, 96);
    let smooth_dielectric = pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, 0);
    let smooth_metal =
        pbr_matrix_cell_rgb(frame, PBR_MATRIX_DIMENSION - 1, PBR_MATRIX_DIMENSION - 1);
    let rough_metal = pbr_matrix_cell_rgb(frame, 0, PBR_MATRIX_DIMENSION - 1);

    assert!(
        color_distance(upper_sky, lower_sky) > 8.0,
        "real HDRI skybox should show directional scene variation, upper={upper_sky:?}, lower={lower_sky:?}"
    );
    assert!(
        color_distance(smooth_metal, smooth_dielectric) > 4.0,
        "smooth metallic cells should visibly differ from dielectric cells under real HDRI, metal={smooth_metal:?}, dielectric={smooth_dielectric:?}"
    );
    assert!(
        color_distance(smooth_metal, rough_metal) > 2.0,
        "smoothness should change real HDRI reflection response, smooth={smooth_metal:?}, rough={rough_metal:?}"
    );
    assert_pbr_matrix_quantitative_response(frame);
    assert_no_legacy_16x8_mosaic_grid(frame);
}

fn assert_pmrem_mip_diagnostic_blur_response(frame: &FramePixels<'_>) {
    let tile = PMREM_MIP_DIAGNOSTIC_TILE_SIZE;
    let mip_count = frame.width / tile;
    assert!(
        mip_count >= 4,
        "PMREM mip diagnostic should contain enough mip columns for blur validation, mip_count={mip_count}"
    );

    let mip0_source = average_mip_diagnostic_energy(frame, 0, 0);
    let mip0_pmrem = average_mip_diagnostic_energy(frame, 0, 6);
    assert!(
        (mip0_source - mip0_pmrem).abs() <= 0.001,
        "PMREM diagnostic mip0 should match the source cubemap before filtering, source={mip0_source}, pmrem={mip0_pmrem}"
    );

    let mut ratio_sum = 0.0_f32;
    let mut ratio_count = 0.0_f32;
    for mip in 1..mip_count.saturating_sub(1) {
        let source_energy = average_mip_diagnostic_energy(frame, mip, 0);
        let pmrem_energy = average_mip_diagnostic_energy(frame, mip, 6);
        if source_energy <= 0.5 {
            continue;
        }
        assert!(
            pmrem_energy <= source_energy * 0.92 + 0.05,
            "PMREM mip should be blurrier than the same-level source mip, mip={mip}, source={source_energy}, pmrem={pmrem_energy}"
        );
        ratio_sum += pmrem_energy / source_energy;
        ratio_count += 1.0;
    }
    assert!(
        ratio_count >= 3.0,
        "PMREM mip diagnostic should expose at least three non-flat rough mip comparisons"
    );
    assert!(
        ratio_sum / ratio_count <= 0.75,
        "PMREM rough mip chain should reduce high-frequency energy across the diagnostic, average_ratio={}",
        ratio_sum / ratio_count
    );

    let source_mip1 = average_mip_diagnostic_energy(frame, 1, 0);
    let pmrem_mip1 = average_mip_diagnostic_energy(frame, 1, 6);
    assert!(
        pmrem_mip1 <= source_mip1 * 0.55,
        "first PMREM rough mip should visibly blur the source cubemap diagnostic, source={source_mip1}, pmrem={pmrem_mip1}"
    );
}

fn average_mip_diagnostic_energy(frame: &FramePixels<'_>, mip: u32, first_face_row: u32) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for face in 0..6 {
        sum += mip_diagnostic_tile_high_frequency_energy(frame, mip, first_face_row + face);
        count += 1.0;
    }
    sum / count.max(1.0)
}

fn mip_diagnostic_tile_high_frequency_energy(
    frame: &FramePixels<'_>,
    tile_x: u32,
    tile_y: u32,
) -> f32 {
    let tile = PMREM_MIP_DIAGNOSTIC_TILE_SIZE;
    let x_min = tile_x * tile + 1;
    let x_max = ((tile_x + 1) * tile).saturating_sub(2);
    let y_min = tile_y * tile + 1;
    let y_max = ((tile_y + 1) * tile).saturating_sub(2);
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;

    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let gradient = (pixel_luma(frame, x + 1, y) - pixel_luma(frame, x - 1, y)).abs()
                + (pixel_luma(frame, x, y + 1) - pixel_luma(frame, x, y - 1)).abs();
            sum += gradient;
            count += 1.0;
        }
    }

    sum / count.max(1.0)
}

#[derive(Clone, Copy)]
struct FramePixels<'a> {
    width: u32,
    height: u32,
    rgba: &'a [u8],
}

impl<'a> FramePixels<'a> {
    fn from_viewport(frame: &'a ViewportFrame) -> Self {
        Self::from_rgba_bytes(frame.width, frame.height, &frame.rgba)
    }

    fn from_rgba_bytes(width: u32, height: u32, rgba: &'a [u8]) -> Self {
        assert_eq!(
            rgba.len(),
            width as usize * height as usize * 4,
            "RGBA byte length must match frame dimensions"
        );
        Self {
            width,
            height,
            rgba,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PbrMatrixCellMetrics {
    rgb: [f32; 3],
    luma: f32,
}

fn assert_pbr_matrix_quantitative_response(frame: &FramePixels<'_>) {
    let cells = pbr_matrix_metrics(frame);
    let mut min_luma = f32::MAX;
    let mut max_luma = f32::MIN;
    for row in &cells {
        for cell in row {
            min_luma = min_luma.min(cell.luma);
            max_luma = max_luma.max(cell.luma);
            assert!(
                cell.luma > 20.0,
                "every PBR matrix cell should remain visible under real HDRI, cell={cell:?}"
            );
        }
    }

    let smooth_row = PBR_MATRIX_DIMENSION - 1;
    let rough_rows_high_metal =
        average_cell_rgb((0..3).map(|row| cells[row][PBR_MATRIX_DIMENSION - 1]));
    let smooth_rows_high_metal = average_cell_rgb(
        ((PBR_MATRIX_DIMENSION - 3)..PBR_MATRIX_DIMENSION)
            .map(|row| cells[row][PBR_MATRIX_DIMENSION - 1]),
    );
    let smooth_low_metal = average_cell_rgb((0..3).map(|column| cells[smooth_row][column]));
    let smooth_high_metal =
        average_cell_rgb((5..PBR_MATRIX_DIMENSION).map(|column| cells[smooth_row][column]));
    let smooth_endpoint_delta = color_distance(
        cells[smooth_row][PBR_MATRIX_DIMENSION - 1].rgb,
        cells[smooth_row][0].rgb,
    );
    let row_spreads = pbr_matrix_row_spreads(&cells);
    let column_spreads = pbr_matrix_column_spreads(&cells);
    let responsive_rows = row_spreads.iter().filter(|spread| **spread > 40.0).count();
    let responsive_columns = column_spreads
        .iter()
        .filter(|spread| **spread > 18.0)
        .count();

    assert!(
        max_luma - min_luma > 72.0,
        "{}x{} PBR matrix should have broad real-HDRI response range, min={min_luma}, max={max_luma}",
        PBR_MATRIX_DIMENSION,
        PBR_MATRIX_DIMENSION
    );
    assert!(
        smooth_endpoint_delta > 24.0,
        "smooth metallic endpoint should visibly diverge from smooth dielectric endpoint, delta={smooth_endpoint_delta}, smooth_metal={:?}, smooth_dielectric={:?}",
        cells[smooth_row][PBR_MATRIX_DIMENSION - 1].rgb,
        cells[smooth_row][0].rgb
    );
    assert!(
        color_distance(smooth_low_metal, smooth_high_metal) > 12.0,
        "metallic ramp group average should shift the smooth row from dielectric to environment reflection, low={smooth_low_metal:?}, high={smooth_high_metal:?}"
    );
    let rough_high_metal_luma = luma(rough_rows_high_metal);
    let smooth_high_metal_luma = luma(smooth_rows_high_metal);
    let high_metal_luma_ratio = rough_high_metal_luma.max(smooth_high_metal_luma)
        / rough_high_metal_luma.min(smooth_high_metal_luma).max(1.0);
    assert!(
        high_metal_luma_ratio < 1.25,
        "PMREM roughness should redistribute reflection detail without large high-metal energy drift, rough={rough_rows_high_metal:?}, smooth={smooth_rows_high_metal:?}, ratio={high_metal_luma_ratio}"
    );
    assert!(
        responsive_rows >= PBR_MATRIX_DIMENSION / 2,
        "the smoother half of the PBR matrix rows should respond across the metallic sweep, responsive_rows={responsive_rows}, spreads={row_spreads:?}"
    );
    assert!(
        responsive_columns >= PBR_MATRIX_DIMENSION - 2,
        "most PBR matrix columns should exceed the saved-frame noise floor across the eight-step smoothness sweep, responsive_columns={responsive_columns}, spreads={column_spreads:?}"
    );
    assert_high_metal_smoothness_increases_reflection_detail(frame);
}

fn assert_high_metal_smoothness_increases_reflection_detail(frame: &FramePixels<'_>) {
    let high_metal_columns = [
        PBR_MATRIX_DIMENSION - 3,
        PBR_MATRIX_DIMENSION - 2,
        PBR_MATRIX_DIMENSION - 1,
    ];
    let rough_rows = [0_usize, 1, 2];
    let mid_smooth_rows = [
        PBR_MATRIX_DIMENSION / 2 - 1,
        PBR_MATRIX_DIMENSION / 2,
        PBR_MATRIX_DIMENSION / 2 + 1,
    ];
    let smooth_rows = [
        PBR_MATRIX_DIMENSION - 3,
        PBR_MATRIX_DIMENSION - 2,
        PBR_MATRIX_DIMENSION - 1,
    ];

    let rough_energy =
        average_high_frequency_energy_for_cells(frame, &rough_rows, &high_metal_columns);
    let mid_smooth_energy =
        average_high_frequency_energy_for_cells(frame, &mid_smooth_rows, &high_metal_columns);
    let smooth_energy =
        average_high_frequency_energy_for_cells(frame, &smooth_rows, &high_metal_columns);

    assert!(
        mid_smooth_energy > rough_energy * 1.05,
        "higher smoothness should restore HDRI reflection detail in high-metal cells, rough={rough_energy}, mid_smooth={mid_smooth_energy}, smooth={smooth_energy}"
    );
    assert!(
        smooth_energy > rough_energy * 1.10,
        "smooth high-metal cells should retain more high-frequency reflection detail than rough high-metal cells, rough={rough_energy}, smooth={smooth_energy}"
    );
}

fn average_high_frequency_energy_for_cells(
    frame: &FramePixels<'_>,
    rows: &[usize],
    columns: &[usize],
) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for row in rows {
        for column in columns {
            sum += pbr_matrix_cell_high_frequency_energy(frame, *row, *column);
            count += 1.0;
        }
    }
    sum / count.max(1.0)
}

fn pbr_matrix_cell_high_frequency_energy(
    frame: &FramePixels<'_>,
    row: usize,
    column: usize,
) -> f32 {
    let (center_x, center_y) = pbr_matrix_cell_center(frame, row, column);
    let radius = (PBR_MATRIX_CELL_SAMPLE_SIZE / 2).saturating_sub(4).max(4);
    let x_min = center_x.saturating_sub(radius).max(1);
    let x_max = center_x
        .saturating_add(radius)
        .min(frame.width.saturating_sub(2));
    let y_min = center_y.saturating_sub(radius).max(1);
    let y_max = center_y
        .saturating_add(radius)
        .min(frame.height.saturating_sub(2));
    let radius_f = radius as f32;
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;

    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let normalized_x = (x as f32 - center_x as f32) / radius_f;
            let normalized_y = (y as f32 - center_y as f32) / radius_f;
            let radius_sq = normalized_x * normalized_x + normalized_y * normalized_y;
            if !(0.05..=1.0).contains(&radius_sq) {
                continue;
            }
            let gradient = (pixel_luma(frame, x + 1, y) - pixel_luma(frame, x - 1, y)).abs()
                + (pixel_luma(frame, x, y + 1) - pixel_luma(frame, x, y - 1)).abs();
            sum += gradient;
            count += 1.0;
        }
    }

    sum / count.max(1.0)
}

fn assert_no_legacy_16x8_mosaic_grid(frame: &FramePixels<'_>) {
    let vertical_boundaries = legacy_grid_vertical_boundary_luma_deltas(frame);
    let vertical_offsets = legacy_grid_vertical_offset_luma_deltas(frame);
    let horizontal_boundaries = legacy_grid_horizontal_boundary_luma_deltas(frame);
    let horizontal_offsets = legacy_grid_horizontal_offset_luma_deltas(frame);
    let vertical_threshold = (mean(&vertical_offsets) * 4.0).max(1.0);
    let horizontal_threshold = (mean(&horizontal_offsets) * 4.0).max(1.0);
    let vertical_hits = vertical_boundaries
        .iter()
        .filter(|delta| **delta > vertical_threshold)
        .count();
    let horizontal_hits = horizontal_boundaries
        .iter()
        .filter(|delta| **delta > horizontal_threshold)
        .count();

    assert_eq!(
        vertical_hits, 0,
        "skybox should not expose legacy 16-column sample-table seams, threshold={vertical_threshold}, boundaries={vertical_boundaries:?}, offsets={vertical_offsets:?}"
    );
    assert_eq!(
        horizontal_hits, 0,
        "skybox should not expose legacy 8-row sample-table seams, threshold={horizontal_threshold}, boundaries={horizontal_boundaries:?}, offsets={horizontal_offsets:?}"
    );
}

fn pbr_matrix_metrics(
    frame: &FramePixels<'_>,
) -> [[PbrMatrixCellMetrics; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION] {
    let mut cells = [[PbrMatrixCellMetrics::default(); PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION];
    for (row_index, row) in cells.iter_mut().enumerate() {
        for (column_index, cell) in row.iter_mut().enumerate() {
            let rgb = pbr_matrix_cell_rgb_with_size(
                frame,
                row_index,
                column_index,
                PBR_MATRIX_CELL_SAMPLE_SIZE,
            );
            *cell = PbrMatrixCellMetrics {
                rgb,
                luma: luma(rgb),
            };
        }
    }
    cells
}

fn average_cell_rgb(cells: impl IntoIterator<Item = PbrMatrixCellMetrics>) -> [f32; 3] {
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for cell in cells {
        sum[0] += cell.rgb[0];
        sum[1] += cell.rgb[1];
        sum[2] += cell.rgb[2];
        count += 1.0;
    }
    if count <= 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [sum[0] / count, sum[1] / count, sum[2] / count]
    }
}

fn pbr_matrix_row_spreads(
    cells: &[[PbrMatrixCellMetrics; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION],
) -> [f32; PBR_MATRIX_DIMENSION] {
    let mut spreads = [0.0_f32; PBR_MATRIX_DIMENSION];
    for row in 0..PBR_MATRIX_DIMENSION {
        for first in 0..PBR_MATRIX_DIMENSION {
            for second in first + 1..PBR_MATRIX_DIMENSION {
                spreads[row] = spreads[row].max(color_distance(
                    cells[row][first].rgb,
                    cells[row][second].rgb,
                ));
            }
        }
    }
    spreads
}

fn pbr_matrix_column_spreads(
    cells: &[[PbrMatrixCellMetrics; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION],
) -> [f32; PBR_MATRIX_DIMENSION] {
    let mut spreads = [0.0_f32; PBR_MATRIX_DIMENSION];
    for column in 0..PBR_MATRIX_DIMENSION {
        for first in 0..PBR_MATRIX_DIMENSION {
            for second in first + 1..PBR_MATRIX_DIMENSION {
                spreads[column] = spreads[column].max(color_distance(
                    cells[first][column].rgb,
                    cells[second][column].rgb,
                ));
            }
        }
    }
    spreads
}

fn legacy_grid_vertical_boundary_luma_deltas(frame: &FramePixels<'_>) -> Vec<f32> {
    let step = frame.width / LEGACY_EQUIRECT_SAMPLE_COLUMNS;
    let y_start = LEGACY_GRID_SKY_SAMPLE_Y_MIN.min(frame.height);
    let y_end = LEGACY_GRID_SKY_SAMPLE_Y_MAX
        .min(frame.height)
        .max(y_start + 1);
    (step..frame.width)
        .step_by(step.max(1) as usize)
        .filter(|x| *x > 0 && *x < frame.width)
        .map(|x| average_luma_delta_x(frame, x, y_start, y_end))
        .collect()
}

fn legacy_grid_vertical_offset_luma_deltas(frame: &FramePixels<'_>) -> Vec<f32> {
    let step = frame.width / LEGACY_EQUIRECT_SAMPLE_COLUMNS;
    let y_start = LEGACY_GRID_SKY_SAMPLE_Y_MIN.min(frame.height);
    let y_end = LEGACY_GRID_SKY_SAMPLE_Y_MAX
        .min(frame.height)
        .max(y_start + 1);
    let mut deltas = Vec::new();
    for boundary in (step..frame.width).step_by(step.max(1) as usize) {
        for offset in LEGACY_GRID_OFFSET_SAMPLES {
            let x = (boundary as i32 + offset).clamp(1, frame.width.saturating_sub(1) as i32);
            deltas.push(average_luma_delta_x(frame, x as u32, y_start, y_end));
        }
    }
    deltas
}

fn legacy_grid_horizontal_boundary_luma_deltas(frame: &FramePixels<'_>) -> Vec<f32> {
    let step = frame.height / LEGACY_EQUIRECT_SAMPLE_ROWS;
    let x_start = 40.min(frame.width);
    let x_end = frame.width.saturating_sub(40).max(x_start + 1);
    (step..LEGACY_GRID_SKY_SAMPLE_Y_MAX.min(frame.height))
        .step_by(step.max(1) as usize)
        .filter(|y| *y > 0 && *y < frame.height)
        .map(|y| average_luma_delta_y(frame, y, x_start, x_end))
        .collect()
}

fn legacy_grid_horizontal_offset_luma_deltas(frame: &FramePixels<'_>) -> Vec<f32> {
    let step = frame.height / LEGACY_EQUIRECT_SAMPLE_ROWS;
    let x_start = 40.min(frame.width);
    let x_end = frame.width.saturating_sub(40).max(x_start + 1);
    let mut deltas = Vec::new();
    for boundary in
        (step..LEGACY_GRID_SKY_SAMPLE_Y_MAX.min(frame.height)).step_by(step.max(1) as usize)
    {
        for offset in LEGACY_GRID_OFFSET_SAMPLES {
            let y = (boundary as i32 + offset).clamp(1, frame.height.saturating_sub(1) as i32);
            deltas.push(average_luma_delta_y(frame, y as u32, x_start, x_end));
        }
    }
    deltas
}

fn average_luma_delta_x(frame: &FramePixels<'_>, x: u32, y_start: u32, y_end: u32) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for y in y_start..y_end {
        sum += (pixel_luma(frame, x, y) - pixel_luma(frame, x - 1, y)).abs();
        count += 1.0;
    }
    if count <= 0.0 {
        0.0
    } else {
        sum / count
    }
}

fn average_luma_delta_y(frame: &FramePixels<'_>, y: u32, x_start: u32, x_end: u32) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for x in x_start..x_end {
        sum += (pixel_luma(frame, x, y) - pixel_luma(frame, x, y - 1)).abs();
        count += 1.0;
    }
    if count <= 0.0 {
        0.0
    } else {
        sum / count
    }
}

fn pixel_luma(frame: &FramePixels<'_>, x: u32, y: u32) -> f32 {
    let index = (y as usize * frame.width as usize + x as usize) * 4;
    luma([
        frame.rgba[index] as f32,
        frame.rgba[index + 1] as f32,
        frame.rgba[index + 2] as f32,
    ])
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn pbr_matrix_cell_rgb(frame: &FramePixels<'_>, row: usize, column: usize) -> [f32; 3] {
    pbr_matrix_cell_rgb_with_size(frame, row, column, 40)
}

fn pbr_matrix_cell_rgb_with_size(
    frame: &FramePixels<'_>,
    row: usize,
    column: usize,
    sample_size: u32,
) -> [f32; 3] {
    let (center_x, center_y) = pbr_matrix_cell_center(frame, row, column);
    average_region_rgb(
        frame,
        center_x.saturating_sub(sample_size / 2),
        center_y.saturating_sub(sample_size / 2),
        sample_size,
        sample_size,
    )
}

fn pbr_matrix_cell_center(frame: &FramePixels<'_>, row: usize, column: usize) -> (u32, u32) {
    let aspect = frame.width as f32 / frame.height as f32;
    let half_height = PBR_MATRIX_ORTHO_SIZE;
    let half_width = half_height * aspect;
    let center_x = ((super::pbr_matrix_world_x(column) + half_width) / (half_width * 2.0)
        * frame.width as f32)
        .round()
        .clamp(0.0, frame.width.saturating_sub(1) as f32) as u32;
    let center_y = ((half_height - super::pbr_matrix_world_y(row)) / (half_height * 2.0)
        * frame.height as f32)
        .round()
        .clamp(0.0, frame.height.saturating_sub(1) as f32) as u32;
    (center_x, center_y)
}

fn average_region_rgb(
    frame: &FramePixels<'_>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> [f32; 3] {
    let x_end = x.saturating_add(width).min(frame.width);
    let y_end = y.saturating_add(height).min(frame.height);
    let frame_width = frame.width as usize;
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for py in y as usize..y_end as usize {
        for px in x as usize..x_end as usize {
            let index = (py * frame_width + px) * 4;
            sum[0] += frame.rgba[index] as f32;
            sum[1] += frame.rgba[index + 1] as f32;
            sum[2] += frame.rgba[index + 2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [sum[0] / count, sum[1] / count, sum[2] / count]
    }
}

fn color_distance(first: [f32; 3], second: [f32; 3]) -> f32 {
    let dr = first[0] - second[0];
    let dg = first[1] - second[1];
    let db = first[2] - second[2];
    (dr * dr + dg * dg + db * db).sqrt()
}
