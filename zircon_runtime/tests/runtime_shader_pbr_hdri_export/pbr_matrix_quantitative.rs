//! Quantitative acceptance gates for the 8x8 metallic and smoothness product render.

use std::fmt::Write;

use zircon_runtime::core::framework::render::{
    build_environment_brdf_lut, cubemap_direction_from_scaled_uv,
    cubemap_face_scaled_uv_from_direction, cubemap_texel_solid_angle,
    source_cubemap_evaluate_irradiance_sh9, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, source_cubemap_pmrem_mip_from_roughness, CubemapFace,
    SourceCubemapEnvironment, ENVIRONMENT_BRDF_LUT_SIZE,
};
use zircon_runtime::graphics::ViewportFrame;

use super::{
    luma, pbr_matrix_axis_value, pbr_matrix_world_x, pbr_matrix_world_y, PBR_MATRIX_DIMENSION,
    PBR_MATRIX_ORTHO_SIZE, PBR_MATRIX_OUTPUT_SIZE, PBR_MATRIX_SPHERE_SCALE,
};

#[path = "pbr_matrix_quantitative/math.rs"]
mod math;

use math::*;

const BASE_COLOR: [f32; 3] = [0.78, 0.74, 0.66];
const MIRROR_MIN_SSIM: f32 = 0.95;
const REAL_HDRI_PMREM_MIN_SSIM: f32 = 0.90;
const ROUGHNESS_NOISE_FLOOR: f32 = 1.0e-6;
const DIELECTRIC_MAX_DELTA_E: f32 = 12.0;
const DIELECTRIC_CENTER_F0_MIN: f32 = 0.015;
const DIELECTRIC_CENTER_F0_MAX: f32 = 0.12;
const DIELECTRIC_GRAZING_RESPONSE_DELTA: f32 = 0.03;

#[derive(Debug)]
pub(super) struct PbrMatrixQuantitativeReport {
    grazing_mean_relative_delta: f32,
    grazing_max_per_radius_relative_delta: f32,
    mirror_ssim: f32,
    minimum_lakes_pmrem_ssim: f32,
    roughness_laplacian_variance: [[f32; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION],
    minimum_roughness_adjacent_delta: f32,
    dielectric_delta_e: f32,
    dielectric_center_f0_response: f32,
    dielectric_grazing_response: f32,
    rough_metal_luma: f32,
    rough_metal_lower_bound: f32,
    rough_metal_upper_bound: f32,
}

impl PbrMatrixQuantitativeReport {
    pub(super) fn to_text(&self) -> String {
        let mut output = String::new();
        writeln!(output, "Shader 06 PBR matrix quantitative report").unwrap();
        writeln!(
            output,
            "grazing_mean_relative_delta={:.6}",
            self.grazing_mean_relative_delta
        )
        .unwrap();
        writeln!(
            output,
            "grazing_max_per_radius_relative_delta={:.6}",
            self.grazing_max_per_radius_relative_delta
        )
        .unwrap();
        writeln!(output, "mirror_ssim={:.6}", self.mirror_ssim).unwrap();
        writeln!(
            output,
            "minimum_lakes_pmrem_ssim={:.6}",
            self.minimum_lakes_pmrem_ssim
        )
        .unwrap();
        writeln!(
            output,
            "minimum_roughness_adjacent_delta={:.8}",
            self.minimum_roughness_adjacent_delta
        )
        .unwrap();
        for column in 0..PBR_MATRIX_DIMENSION {
            write!(output, "metallic_column_{column}_laplacian_variance=[").unwrap();
            for (index, value) in self.roughness_laplacian_variance[column].iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write!(output, "{value:.8}").unwrap();
            }
            output.push_str("]\n");
        }
        writeln!(output, "dielectric_delta_e={:.6}", self.dielectric_delta_e).unwrap();
        writeln!(
            output,
            "dielectric_center_f0_response={:.6}",
            self.dielectric_center_f0_response
        )
        .unwrap();
        writeln!(
            output,
            "dielectric_grazing_response={:.6}",
            self.dielectric_grazing_response
        )
        .unwrap();
        writeln!(output, "rough_metal_luma={:.6}", self.rough_metal_luma).unwrap();
        writeln!(
            output,
            "rough_metal_bounds=[{:.6}, {:.6}]",
            self.rough_metal_lower_bound, self.rough_metal_upper_bound
        )
        .unwrap();
        output
    }
}

pub(super) fn assert_plan06_quantitative_gates(
    frame: &ViewportFrame,
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
    environment: &SourceCubemapEnvironment,
    frequency_hdr: &[[f32; 4]],
    frequency_diffuse_hdr: &[[f32; 4]],
    frequency_environment: &SourceCubemapEnvironment,
    grazing_mean_relative_delta: f32,
    grazing_max_per_radius_relative_delta: f32,
) -> PbrMatrixQuantitativeReport {
    assert_eq!(
        (frame.width, frame.height),
        (PBR_MATRIX_OUTPUT_SIZE.x, PBR_MATRIX_OUTPUT_SIZE.y)
    );
    assert_eq!(hdr.len(), frame.width as usize * frame.height as usize);
    assert_eq!(diffuse_hdr.len(), hdr.len());
    assert_eq!(frequency_hdr.len(), hdr.len());
    assert_eq!(frequency_diffuse_hdr.len(), hdr.len());

    let brdf_lut = build_environment_brdf_lut(ENVIRONMENT_BRDF_LUT_SIZE, 1024);
    let cells = build_cell_samples(
        frame.width,
        frame.height,
        hdr,
        diffuse_hdr,
        environment,
        &brdf_lut,
    );
    let frequency_cells = build_cell_samples(
        frame.width,
        frame.height,
        frequency_hdr,
        frequency_diffuse_hdr,
        frequency_environment,
        &brdf_lut,
    );
    let mirror_ssim =
        mirror_source_ssim(&cells[PBR_MATRIX_DIMENSION - 1][PBR_MATRIX_DIMENSION - 1]);
    eprintln!("plan06 mirror_ssim={mirror_ssim:.6}");
    assert!(
        mirror_ssim >= MIRROR_MIN_SSIM,
        "smooth metallic matrix sphere must match direct source-cubemap reflection, ssim={mirror_ssim}, threshold={MIRROR_MIN_SSIM}"
    );

    let minimum_lakes_pmrem_ssim = assert_real_hdri_pmrem_lod_response(&cells);
    let (roughness_laplacian_variance, minimum_roughness_adjacent_delta) =
        assert_roughness_monotonicity("controlled", &frequency_cells);

    let rough_dielectric = &cells[0][0];
    let dielectric_delta_e = average_dielectric_body_delta_e(rough_dielectric);
    assert!(
        dielectric_delta_e < DIELECTRIC_MAX_DELTA_E,
        "rough dielectric body must match SH9 irradiance times albedo, delta_e={dielectric_delta_e}, threshold={DIELECTRIC_MAX_DELTA_E}"
    );

    let smooth_dielectric = &cells[PBR_MATRIX_DIMENSION - 1][0];
    let dielectric_center_f0_response = average_specular_response(smooth_dielectric, 0.72, 1.0);
    let dielectric_grazing_response = average_specular_response(smooth_dielectric, 0.12, 0.38);
    assert!(
        (DIELECTRIC_CENTER_F0_MIN..=DIELECTRIC_CENTER_F0_MAX)
            .contains(&dielectric_center_f0_response),
        "dielectric normal-incidence response must retain the 4% F0 lobe, response={dielectric_center_f0_response}"
    );
    assert!(
        dielectric_grazing_response
            > dielectric_center_f0_response + DIELECTRIC_GRAZING_RESPONSE_DELTA,
        "dielectric grazing Fresnel must visibly exceed center response, center={dielectric_center_f0_response}, grazing={dielectric_grazing_response}"
    );

    let rough_metal = &cells[0][PBR_MATRIX_DIMENSION - 1];
    let rough_metal_luma = mean_sample_luma(rough_metal, |sample| sample.no_v >= 0.18);
    let sh_mean_luma = luma([
        environment.irradiance_sh9[0][0] * 0.282_094_8,
        environment.irradiance_sh9[0][1] * 0.282_094_8,
        environment.irradiance_sh9[0][2] * 0.282_094_8,
    ]) * environment.intensity;
    let source_mean_luma = source_cubemap_solid_angle_mean_luma(environment);
    let rough_metal_lower_bound = sh_mean_luma * BASE_COLOR[2] * 0.20;
    let rough_metal_upper_bound = source_mean_luma * 1.05;
    assert!(
        (rough_metal_lower_bound..=rough_metal_upper_bound).contains(&rough_metal_luma),
        "rough metallic energy must remain inside the SH/source bounds, luma={rough_metal_luma}, lower={rough_metal_lower_bound}, upper={rough_metal_upper_bound}"
    );

    PbrMatrixQuantitativeReport {
        grazing_mean_relative_delta,
        grazing_max_per_radius_relative_delta,
        mirror_ssim,
        minimum_lakes_pmrem_ssim,
        roughness_laplacian_variance,
        minimum_roughness_adjacent_delta,
        dielectric_delta_e,
        dielectric_center_f0_response,
        dielectric_grazing_response,
        rough_metal_luma,
        rough_metal_lower_bound,
        rough_metal_upper_bound,
    }
}

fn assert_real_hdri_pmrem_lod_response(
    cells: &[[MatrixCellSamples; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION],
) -> f32 {
    let mut minimum_ssim = f32::MAX;
    for (row, row_cells) in cells.iter().enumerate() {
        for (column, samples) in row_cells.iter().enumerate() {
            let selected = samples.iter().filter(|sample| sample.no_v >= 0.55);
            let (rendered, expected): (Vec<_>, Vec<_>) = selected
                .map(|sample| {
                    (
                        luma(sample.reflection_estimate),
                        luma(sample.pmrem_reflection),
                    )
                })
                .unzip();
            let rendered = normalize_percentile(&rendered, 0.02, 0.98);
            let expected = normalize_percentile(&expected, 0.02, 0.98);
            let ssim = global_ssim(&rendered, &expected);
            minimum_ssim = minimum_ssim.min(ssim);
            assert!(
                ssim >= REAL_HDRI_PMREM_MIN_SSIM,
                "real Lakes reflection must match the PMREM mip selected from material roughness, row={row}, column={column}, ssim={ssim}, threshold={REAL_HDRI_PMREM_MIN_SSIM}"
            );
        }
    }
    eprintln!("plan06 minimum_lakes_pmrem_ssim={minimum_ssim:.6}");
    minimum_ssim
}

fn assert_roughness_monotonicity(
    label: &str,
    cells: &[[MatrixCellSamples; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION],
) -> ([[f32; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION], f32) {
    let mut variances = [[0.0_f32; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION];
    let mut minimum_delta = f32::MAX;
    for column in 0..PBR_MATRIX_DIMENSION {
        for row in 0..PBR_MATRIX_DIMENSION {
            variances[column][row] = laplacian_variance(&cells[row][column]);
        }
    }
    eprintln!("plan06 {label}_specular_laplacian_variance={variances:?}");
    for column in 0..PBR_MATRIX_DIMENSION {
        for row in 1..PBR_MATRIX_DIMENSION {
            let rougher = variances[column][row - 1];
            let smoother = variances[column][row];
            let delta = smoother - rougher;
            minimum_delta = minimum_delta.min(delta);
            let required_delta = ROUGHNESS_NOISE_FLOOR.max(rougher.abs() * 0.005);
            assert!(
                delta > required_delta,
                "{label} reflection Laplacian variance must strictly increase with smoothness above noise, metallic_column={column}, rough_row={}, smooth_row={row}, rough={rougher}, smooth={smoother}, delta={delta}, required={required_delta}, column_values={:?}",
                row - 1,
                variances[column]
            );
        }
    }
    (variances, minimum_delta)
}

fn blurred_log_laplacian_variance(samples: &[MatrixSample], radius: u32) -> f32 {
    let mut source = std::collections::HashMap::with_capacity(samples.len());
    for sample in samples.iter().filter(|sample| sample.no_v >= 0.45) {
        source.insert(
            (sample.x, sample.y),
            luma(sample.reflection_estimate).max(0.0).ln_1p(),
        );
    }
    let mut blurred = std::collections::HashMap::with_capacity(source.len());
    for &(x, y) in source.keys() {
        let mut sum = 0.0;
        let mut count = 0_u32;
        for offset_y in -(radius as i32)..=radius as i32 {
            for offset_x in -(radius as i32)..=radius as i32 {
                let Some(sample_x) = x.checked_add_signed(offset_x) else {
                    continue;
                };
                let Some(sample_y) = y.checked_add_signed(offset_y) else {
                    continue;
                };
                if let Some(value) = source.get(&(sample_x, sample_y)) {
                    sum += value;
                    count += 1;
                }
            }
        }
        blurred.insert((x, y), sum / count.max(1) as f32);
    }
    laplacian_variance_from_lookup(&blurred)
}

#[derive(Clone, Copy, Debug)]
struct MatrixSample {
    x: u32,
    y: u32,
    no_v: f32,
    rendered: [f32; 3],
    measured_diffuse: [f32; 3],
    measured_specular: [f32; 3],
    expected_diffuse: [f32; 3],
    reflection_estimate: [f32; 3],
    source_reflection: [f32; 3],
    pmrem_reflection: [f32; 3],
}

type MatrixCellSamples = Vec<MatrixSample>;

fn build_cell_samples(
    width: u32,
    height: u32,
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
    environment: &SourceCubemapEnvironment,
    brdf_lut: &[[f32; 2]],
) -> [[MatrixCellSamples; PBR_MATRIX_DIMENSION]; PBR_MATRIX_DIMENSION] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            build_cell_sample_set(
                width,
                height,
                hdr,
                diffuse_hdr,
                environment,
                brdf_lut,
                row,
                column,
            )
        })
    })
}

fn build_cell_sample_set(
    width: u32,
    height: u32,
    hdr: &[[f32; 4]],
    diffuse_hdr: &[[f32; 4]],
    environment: &SourceCubemapEnvironment,
    brdf_lut: &[[f32; 2]],
    row: usize,
    column: usize,
) -> MatrixCellSamples {
    let center = [pbr_matrix_world_x(column), pbr_matrix_world_y(row)];
    let radius_pixels =
        (PBR_MATRIX_SPHERE_SCALE / (PBR_MATRIX_ORTHO_SIZE * 2.0) * height as f32).ceil() as i32;
    let center_pixel = world_to_pixel(width, height, center);
    let metallic = pbr_matrix_axis_value(column);
    let roughness = (1.0 - pbr_matrix_axis_value(row)).clamp(0.001, 1.0);
    let f0 = mix3([0.04; 3], BASE_COLOR, metallic);
    let mut samples = Vec::new();

    for y in (center_pixel[1] as i32 - radius_pixels)..=(center_pixel[1] as i32 + radius_pixels) {
        for x in (center_pixel[0] as i32 - radius_pixels)..=(center_pixel[0] as i32 + radius_pixels)
        {
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }
            let world = pixel_to_world(width, height, x as u32, y as u32);
            let local_x = (world[0] - center[0]) / PBR_MATRIX_SPHERE_SCALE;
            let local_y = (world[1] - center[1]) / PBR_MATRIX_SPHERE_SCALE;
            let radial_sq = local_x * local_x + local_y * local_y;
            if radial_sq >= 0.985 {
                continue;
            }
            let normal = [local_x, local_y, (1.0 - radial_sq).max(0.0).sqrt()];
            let no_v = normal[2];
            let reflection_dir = normalize3([
                2.0 * normal[0] * normal[2],
                2.0 * normal[1] * normal[2],
                2.0 * normal[2] * normal[2] - 1.0,
            ]);
            let rotated_normal = rotate_y(normal, environment.rotation_radians);
            let rotated_reflection = rotate_y(reflection_dir, environment.rotation_radians);
            let irradiance =
                source_cubemap_evaluate_irradiance_sh9(&environment.irradiance_sh9, rotated_normal);
            let expected_diffuse = mul3_components(
                mul3_scalar(irradiance, environment.intensity),
                mul3_scalar(BASE_COLOR, 1.0 - metallic),
            );
            let source_reflection = mul3_scalar(
                sample_cubemap_lod(environment, rotated_reflection, 0.0, true),
                environment.intensity,
            );
            let pmrem_lod = source_cubemap_pmrem_mip_from_roughness(
                roughness,
                environment.mip_chain.pmrem_mip_count(),
            );
            let pmrem_reflection = mul3_scalar(
                sample_cubemap_lod(environment, rotated_reflection, pmrem_lod, false),
                environment.intensity,
            );
            let brdf = sample_brdf_response(brdf_lut, no_v, roughness, f0);
            let rendered_rgba = hdr[y as usize * width as usize + x as usize];
            let rendered = [rendered_rgba[0], rendered_rgba[1], rendered_rgba[2]];
            let diffuse_rgba = diffuse_hdr[y as usize * width as usize + x as usize];
            let measured_diffuse = [diffuse_rgba[0], diffuse_rgba[1], diffuse_rgba[2]];
            let measured_specular = max3(sub3(rendered, measured_diffuse), 0.0);
            let reflection_estimate = div3_components(measured_specular, max3(brdf, 0.001));
            samples.push(MatrixSample {
                x: x as u32,
                y: y as u32,
                no_v,
                rendered,
                measured_diffuse,
                measured_specular,
                expected_diffuse,
                reflection_estimate,
                source_reflection,
                pmrem_reflection,
            });
        }
    }
    assert!(
        samples.len() >= 800,
        "matrix cell must contain enough HDR samples"
    );
    samples
}

fn mirror_source_ssim(samples: &[MatrixSample]) -> f32 {
    let selected = samples.iter().filter(|sample| sample.no_v >= 0.55);
    let (rendered, reference): (Vec<_>, Vec<_>) = selected
        .map(|sample| {
            (
                luma(sample.reflection_estimate),
                luma(sample.source_reflection),
            )
        })
        .unzip();
    let rendered = normalize_percentile(&rendered, 0.02, 0.98);
    let reference = normalize_percentile(&reference, 0.02, 0.98);
    global_ssim(&rendered, &reference)
}

fn laplacian_variance(samples: &[MatrixSample]) -> f32 {
    blurred_log_laplacian_variance(samples, 4)
}

fn laplacian_variance_from_lookup(lookup: &std::collections::HashMap<(u32, u32), f32>) -> f32 {
    let mut laplacians = Vec::new();
    for (&(x, y), &center) in lookup {
        let Some(left) = x.checked_sub(1).and_then(|px| lookup.get(&(px, y))) else {
            continue;
        };
        let Some(up) = y.checked_sub(1).and_then(|py| lookup.get(&(x, py))) else {
            continue;
        };
        let Some(right) = lookup.get(&(x + 1, y)) else {
            continue;
        };
        let Some(down) = lookup.get(&(x, y + 1)) else {
            continue;
        };
        laplacians.push(4.0 * center - left - right - up - down);
    }
    variance(&laplacians)
}

fn average_dielectric_body_delta_e(samples: &[MatrixSample]) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for sample in samples.iter().filter(|sample| sample.no_v >= 0.72) {
        sum += delta_e_76(
            linear_rgb_to_lab(sample.measured_diffuse),
            linear_rgb_to_lab(sample.expected_diffuse),
        );
        count += 1.0;
    }
    sum / count.max(1.0)
}

fn average_specular_response(samples: &[MatrixSample], no_v_min: f32, no_v_max: f32) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    for sample in samples
        .iter()
        .filter(|sample| (no_v_min..=no_v_max).contains(&sample.no_v))
    {
        let measured = luma(sample.measured_specular);
        let incident = luma(sample.source_reflection).max(0.001);
        sum += measured / incident;
        count += 1.0;
    }
    sum / count.max(1.0)
}

fn mean_sample_luma(samples: &[MatrixSample], include: impl Fn(&MatrixSample) -> bool) -> f32 {
    let values = samples
        .iter()
        .filter(|sample| include(sample))
        .map(|sample| luma(sample.rendered))
        .collect::<Vec<_>>();
    mean(&values)
}

fn source_cubemap_solid_angle_mean_luma(environment: &SourceCubemapEnvironment) -> f32 {
    let face_size = environment.mip_chain.source_face_size();
    let mip_count = environment.mip_chain.source_mip_count();
    let mut weighted_sum = 0.0_f32;
    let mut weight_sum = 0.0_f32;
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        for y in 0..face_size {
            for x in 0..face_size {
                let weight = cubemap_texel_solid_angle(x, y, face_size);
                let texel = environment.mip_chain.source_texels()
                    [offset + y as usize * face_size as usize + x as usize];
                weighted_sum += luma([texel[0], texel[1], texel[2]]) * weight;
                weight_sum += weight;
            }
        }
    }
    weighted_sum / weight_sum.max(f32::EPSILON) * environment.intensity
}

fn sample_brdf_response(lut: &[[f32; 2]], no_v: f32, roughness: f32, f0: [f32; 3]) -> [f32; 3] {
    let size = ENVIRONMENT_BRDF_LUT_SIZE;
    let x = no_v.clamp(0.0, 1.0) * size as f32 - 0.5;
    let y = roughness.clamp(0.0, 1.0) * size as f32 - 0.5;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let sample = |sx: i32, sy: i32| {
        let sx = sx.clamp(0, size as i32 - 1) as usize;
        let sy = sy.clamp(0, size as i32 - 1) as usize;
        lut[sy * size as usize + sx]
    };
    let ab = lerp2(
        lerp2(sample(x0, y0), sample(x0 + 1, y0), tx),
        lerp2(sample(x0, y0 + 1), sample(x0 + 1, y0 + 1), tx),
        ty,
    );
    let f90 = (50.0 * f0[1]).clamp(0.0, 1.0);
    [
        (f0[0] * ab[0] + f90 * ab[1]).clamp(0.0, 1.0),
        (f0[1] * ab[0] + f90 * ab[1]).clamp(0.0, 1.0),
        (f0[2] * ab[0] + f90 * ab[1]).clamp(0.0, 1.0),
    ]
}

fn sample_cubemap_lod(
    environment: &SourceCubemapEnvironment,
    direction: [f32; 3],
    lod: f32,
    source: bool,
) -> [f32; 3] {
    let max_mip = if source {
        environment.mip_chain.source_mip_count()
    } else {
        environment.mip_chain.pmrem_mip_count()
    }
    .saturating_sub(1);
    let lod = lod.clamp(0.0, max_mip as f32);
    let mip0 = lod.floor() as u32;
    let mip1 = (mip0 + 1).min(max_mip);
    let blend = lod - mip0 as f32;
    lerp3(
        sample_cubemap_mip(environment, direction, mip0, source),
        sample_cubemap_mip(environment, direction, mip1, source),
        blend,
    )
}

fn sample_cubemap_mip(
    environment: &SourceCubemapEnvironment,
    direction: [f32; 3],
    mip: u32,
    source: bool,
) -> [f32; 3] {
    let chain = &environment.mip_chain;
    let (face, uv) = cubemap_face_scaled_uv_from_direction(direction);
    let face_size = if source {
        chain.source_face_size()
    } else {
        chain.pmrem_face_size()
    };
    let size = source_cubemap_mip_size(face_size, mip);
    let x = (uv[0] * 0.5 + 0.5) * size as f32 - 0.5;
    let y = (uv[1] * 0.5 + 0.5) * size as f32 - 0.5;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let sample = |sx, sy| sample_cubemap_texel(environment, face, mip, sx, sy, source);
    let color = lerp4(
        lerp4(sample(x0, y0), sample(x0 + 1, y0), tx),
        lerp4(sample(x0, y0 + 1), sample(x0 + 1, y0 + 1), tx),
        ty,
    );
    [color[0], color[1], color[2]]
}

fn sample_cubemap_texel(
    environment: &SourceCubemapEnvironment,
    face: CubemapFace,
    mip: u32,
    x: i32,
    y: i32,
    source: bool,
) -> [f32; 4] {
    let chain = &environment.mip_chain;
    let (face_size, mip_count, texels) = if source {
        (
            chain.source_face_size(),
            chain.source_mip_count(),
            chain.source_texels(),
        )
    } else {
        (
            chain.pmrem_face_size(),
            chain.pmrem_mip_count(),
            chain.pmrem_texels(),
        )
    };
    let size = source_cubemap_mip_size(face_size, mip);
    let (face, x, y) = if x >= 0 && y >= 0 && x < size as i32 && y < size as i32 {
        (face, x as u32, y as u32)
    } else {
        let uv = [
            ((x as f32 + 0.5) / size as f32) * 2.0 - 1.0,
            ((y as f32 + 0.5) / size as f32) * 2.0 - 1.0,
        ];
        let direction = cubemap_direction_from_scaled_uv(face, uv);
        let (next_face, next_uv) = cubemap_face_scaled_uv_from_direction(direction);
        let next_x = (((next_uv[0] * 0.5 + 0.5) * size as f32 - 0.5).round() as i32)
            .clamp(0, size as i32 - 1) as u32;
        let next_y = (((next_uv[1] * 0.5 + 0.5) * size as f32 - 0.5).round() as i32)
            .clamp(0, size as i32 - 1) as u32;
        (next_face, next_x, next_y)
    };
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
    texels[offset + y as usize * size as usize + x as usize]
}

fn world_to_pixel(width: u32, height: u32, world: [f32; 2]) -> [u32; 2] {
    let half_width = PBR_MATRIX_ORTHO_SIZE * width as f32 / height as f32;
    [
        (((world[0] + half_width) / (2.0 * half_width)) * width as f32).round() as u32,
        (((PBR_MATRIX_ORTHO_SIZE - world[1]) / (2.0 * PBR_MATRIX_ORTHO_SIZE)) * height as f32)
            .round() as u32,
    ]
}

fn pixel_to_world(width: u32, height: u32, x: u32, y: u32) -> [f32; 2] {
    let half_width = PBR_MATRIX_ORTHO_SIZE * width as f32 / height as f32;
    [
        (((x as f32 + 0.5) / width as f32) * 2.0 - 1.0) * half_width,
        (1.0 - ((y as f32 + 0.5) / height as f32) * 2.0) * PBR_MATRIX_ORTHO_SIZE,
    ]
}
