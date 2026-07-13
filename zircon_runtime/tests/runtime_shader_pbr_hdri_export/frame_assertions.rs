use zircon_runtime::core::framework::render::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    source_cubemap_face_mip_offset, source_cubemap_mip_size, CubemapFace, ProjectionMode,
    SourceCubemapEnvironment,
};
use zircon_runtime::graphics::ViewportFrame;

use super::scene_fixtures::SinglePbrSphereCameraView;

const MIRROR_SPHERE_CENTER: [f32; 3] = [0.0, -0.12, 0.0];
const MIRROR_SPHERE_RADIUS: f32 = 1.35;
const MIRROR_ORTHOGRAPHIC_CAMERA_Z: f32 = 7.0;
const MIRROR_PERSPECTIVE_CAMERA_Z: f32 = 4.2;
const MIRROR_CAMERA_ORTHO_HALF_HEIGHT: f32 = 3.4;
const MIRROR_CAMERA_FOV_Y_RADIANS: f32 = 60.0_f32.to_radians();

pub(super) fn assert_single_sphere_reflects_environment(frame: &ViewportFrame, label: &str) {
    let upper_sky = average_frame_region_rgb(frame, 40, 32, 128, 128);
    let lower_sky = average_frame_region_rgb(frame, 40, frame.height.saturating_sub(160), 128, 128);
    let center =
        average_frame_region_rgb(frame, frame.width / 2 - 48, frame.height / 2 - 48, 96, 96);
    let (min_luma, max_luma) = frame_luma_range(frame);

    assert!(
        rgb_distance(upper_sky, lower_sky) > 8.0,
        "{label} should render inside a directional real HDRI skybox, upper={upper_sky:?}, lower={lower_sky:?}"
    );
    assert!(
        super::luma(center) > 18.0,
        "{label} center sphere sample should remain visible, center={center:?}"
    );
    assert!(
        max_luma - min_luma > 48.0,
        "{label} should expose real HDRI and material contrast, min_luma={min_luma}, max_luma={max_luma}"
    );
}

pub(super) fn assert_mirror_sphere_reflection_orientation(frame: &ViewportFrame) {
    let background_upper = average_frame_region_rgb(frame, 40, 32, 160, 160);
    let background_lower =
        average_frame_region_rgb(frame, 40, frame.height.saturating_sub(200), 160, 160);
    let sphere_upper =
        average_frame_region_rgb(frame, frame.width / 2 - 96, frame.height / 2 - 230, 192, 72);
    let sphere_upper_inner =
        average_frame_region_rgb(frame, frame.width / 2 - 96, frame.height / 2 - 150, 192, 72);
    let sphere_center =
        average_frame_region_rgb(frame, frame.width / 2 - 72, frame.height / 2 - 36, 144, 72);
    let sphere_lower =
        average_frame_region_rgb(frame, frame.width / 2 - 96, frame.height / 2 + 150, 192, 72);

    let upper_to_lower_background = rgb_distance(sphere_upper, background_lower);
    let upper_to_upper_background = rgb_distance(sphere_upper, background_upper);
    let upper_inner_to_lower_background = rgb_distance(sphere_upper_inner, background_lower);
    let upper_inner_to_upper_background = rgb_distance(sphere_upper_inner, background_upper);
    let upper_inner_blue_excess = sphere_upper_inner[2] - sphere_upper_inner[0];
    let lower_blue_excess = sphere_lower[2] - sphere_lower[0];
    let upper_lower_reflection_distance = rgb_distance(sphere_upper, sphere_lower);
    let center_reflection_distance =
        rgb_distance(sphere_center, sphere_lower).max(rgb_distance(sphere_center, sphere_upper));

    assert!(
        upper_to_upper_background < upper_to_lower_background,
        "mirror sphere upper cap should reflect the upper environment, sphere_upper={sphere_upper:?}, upper_sky={background_upper:?}, lower_sky={background_lower:?}"
    );
    assert!(
        upper_inner_to_upper_background < upper_inner_to_lower_background,
        "mirror sphere upper interior should reflect the upper sky instead of the lower ground, sphere_upper_inner={sphere_upper_inner:?}, upper_sky={background_upper:?}, lower_sky={background_lower:?}"
    );
    assert!(
        upper_inner_blue_excess > lower_blue_excess + 24.0,
        "mirror sphere should keep blue sky above the ground/road band, sphere_upper_inner={sphere_upper_inner:?}, sphere_lower={sphere_lower:?}, upper_blue_excess={upper_inner_blue_excess}, lower_blue_excess={lower_blue_excess}"
    );
    assert!(
        upper_lower_reflection_distance > 12.0,
        "mirror sphere should keep distinct upper/lower reflected HDRI regions, sphere_upper={sphere_upper:?}, sphere_lower={sphere_lower:?}"
    );
    assert!(
        center_reflection_distance > 8.0,
        "mirror sphere should not collapse to a flat clipped reflection, sphere_center={sphere_center:?}, sphere_upper={sphere_upper:?}, sphere_lower={sphere_lower:?}"
    );

    let stats = mirror_sphere_reflection_stats(frame);
    assert!(
        stats.luma_stddev > 28.0,
        "mirror sphere should show sharp HDRI reflection detail instead of a flat white/gray fill, stats={stats:?}"
    );
    assert!(
        stats.mean_saturation > 0.055,
        "mirror sphere should preserve reflected HDRI color variation, stats={stats:?}"
    );
    assert!(
        stats.clipped_ratio < 0.35,
        "mirror sphere should leave the HDRI highlight localized instead of clipping most of the sphere, stats={stats:?}"
    );

    let grazing = mirror_sphere_grazing_balance_stats(frame);
    assert!(
        grazing.right_luma <= grazing.left_luma * 1.35 + 16.0,
        "mirror sphere should not over-emphasize right-side grazing reflection, grazing={grazing:?}"
    );
    assert!(
        grazing.left_luma <= grazing.right_luma * 1.35 + 16.0,
        "mirror sphere should not over-emphasize left-side grazing reflection, grazing={grazing:?}"
    );
    assert!(
        grazing.right_luma_stddev <= grazing.left_luma_stddev * 1.75 + 16.0,
        "mirror sphere right-side grazing detail should stay comparable to the left side, grazing={grazing:?}"
    );
}

pub(super) fn assert_mirror_sphere_matches_source_reference(
    frame: &ViewportFrame,
    projection_mode: ProjectionMode,
    environment: &SourceCubemapEnvironment,
    label: &str,
) {
    let samples = mirror_sphere_source_reference_samples(frame, environment, |ndc, aspect| {
        legacy_front_mirror_sphere_hit(ndc, aspect, projection_mode)
    });
    assert_mirror_sphere_source_reference_samples(&samples, label);
}

pub(super) fn assert_mirror_sphere_matches_source_reference_with_camera_view(
    frame: &ViewportFrame,
    camera_view: SinglePbrSphereCameraView,
    environment: &SourceCubemapEnvironment,
    label: &str,
) {
    let samples = mirror_sphere_source_reference_samples(frame, environment, |ndc, aspect| {
        mirror_sphere_hit(ndc, aspect, camera_view)
    });
    assert_mirror_sphere_source_reference_samples(&samples, label);
}

fn assert_mirror_sphere_source_reference_samples(
    samples: &MirrorSphereSourceReferenceSamples,
    label: &str,
) {
    assert!(
        samples.render_luma.len() > 20_000,
        "{label} source-reference check should sample enough sphere pixels, count={}",
        samples.render_luma.len()
    );

    let render_range = percentile_range(&samples.render_luma, 0.02, 0.98);
    let reference_range = percentile_range(&samples.reference_luma, 0.02, 0.98);
    let render_normalized = normalize_values(&samples.render_luma, render_range);
    let reference_normalized = normalize_values(&samples.reference_luma, reference_range);
    let correlation = pearson_correlation(&render_normalized, &reference_normalized);
    let ssim = global_ssim(&render_normalized, &reference_normalized);

    assert!(
        correlation >= 0.86,
        "{label} mirror sphere should follow the source HDRI reflection field, correlation={correlation}, render_range={render_range:?}, reference_range={reference_range:?}"
    );
    assert!(
        ssim >= 0.68,
        "{label} mirror sphere should preserve source HDRI reflection structure, ssim={ssim}, render_range={render_range:?}, reference_range={reference_range:?}"
    );

    let grazing = mirror_sphere_source_reference_grazing_stats(&samples);
    assert!(
        grazing.sample_count >= 4_000,
        "{label} source-reference grazing check should sample enough rim pixels, grazing={grazing:?}"
    );
    assert!(
        grazing.correlation >= 0.72,
        "{label} grazing reflection should stay correlated with the source HDRI, grazing={grazing:?}"
    );
    assert!(
        (grazing.render_right_left_ratio - grazing.reference_right_left_ratio).abs() <= 0.24,
        "{label} left/right grazing balance should match the source HDRI instead of adding a renderer-side rim, grazing={grazing:?}"
    );
}

pub(super) fn assert_textured_material_has_surface_variation(frame: &ViewportFrame) {
    let x_start = frame.width / 2 - 180;
    let y_start = frame.height / 2 - 180;
    let mut sum = 0.0_f32;
    let mut sum_sq = 0.0_f32;
    let mut count = 0.0_f32;
    for y in (y_start..(y_start + 360).min(frame.height)).step_by(4) {
        for x in (x_start..(x_start + 360).min(frame.width)).step_by(4) {
            let rgb = frame_pixel_rgb(frame, x, y);
            let value = super::luma(rgb);
            sum += value;
            sum_sq += value * value;
            count += 1.0;
        }
    }

    let mean = sum / count.max(1.0);
    let variance = (sum_sq / count.max(1.0) - mean * mean).max(0.0);
    assert!(
        variance.sqrt() > 6.0,
        "textured PBR sphere should show map/reflection surface variation, stddev={}",
        variance.sqrt()
    );
}

fn frame_luma_range(frame: &ViewportFrame) -> (f32, f32) {
    let mut min_luma = f32::MAX;
    let mut max_luma = f32::MIN;
    for pixel in frame.rgba.chunks_exact(4) {
        let value = super::luma([pixel[0] as f32, pixel[1] as f32, pixel[2] as f32]);
        min_luma = min_luma.min(value);
        max_luma = max_luma.max(value);
    }
    (min_luma, max_luma)
}

fn average_frame_region_rgb(
    frame: &ViewportFrame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> [f32; 3] {
    let x_end = x.saturating_add(width).min(frame.width);
    let y_end = y.saturating_add(height).min(frame.height);
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for py in y..y_end {
        for px in x..x_end {
            let rgb = frame_pixel_rgb(frame, px, py);
            sum[0] += rgb[0];
            sum[1] += rgb[1];
            sum[2] += rgb[2];
            count += 1.0;
        }
    }
    if count <= 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [sum[0] / count, sum[1] / count, sum[2] / count]
    }
}

fn frame_pixel_rgb(frame: &ViewportFrame, x: u32, y: u32) -> [f32; 3] {
    let index = (y as usize * frame.width as usize + x as usize) * 4;
    [
        frame.rgba[index] as f32,
        frame.rgba[index + 1] as f32,
        frame.rgba[index + 2] as f32,
    ]
}

#[derive(Clone, Debug, Default)]
struct MirrorSphereSourceReferenceSamples {
    render_luma: Vec<f32>,
    reference_luma: Vec<f32>,
    screen_x_sign: Vec<f32>,
    no_v: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct MirrorSphereSourceReferenceGrazingStats {
    sample_count: usize,
    correlation: f32,
    render_right_left_ratio: f32,
    reference_right_left_ratio: f32,
}

fn mirror_sphere_source_reference_samples(
    frame: &ViewportFrame,
    environment: &SourceCubemapEnvironment,
    mut hit_for_ndc: impl FnMut([f32; 2], f32) -> Option<MirrorSphereHit>,
) -> MirrorSphereSourceReferenceSamples {
    let aspect = frame.width as f32 / frame.height.max(1) as f32;
    let mut samples = MirrorSphereSourceReferenceSamples::default();

    for y in 0..frame.height {
        for x in 0..frame.width {
            let ndc = pixel_ndc(frame, x, y);
            let Some(hit) = hit_for_ndc(ndc, aspect) else {
                continue;
            };
            if hit.no_v <= 0.08 {
                continue;
            }

            let reference_rgb = sample_environment_pmrem_mip0(environment, hit.reflection_dir);
            samples
                .render_luma
                .push(super::luma(frame_pixel_rgb(frame, x, y)));
            samples
                .reference_luma
                .push(reference_display_luma(reference_rgb, environment.intensity));
            samples.screen_x_sign.push((ndc[0]).signum());
            samples.no_v.push(hit.no_v);
        }
    }

    samples
}

fn mirror_sphere_source_reference_grazing_stats(
    samples: &MirrorSphereSourceReferenceSamples,
) -> MirrorSphereSourceReferenceGrazingStats {
    let render_range = percentile_range(&samples.render_luma, 0.02, 0.98);
    let reference_range = percentile_range(&samples.reference_luma, 0.02, 0.98);
    let render_normalized = normalize_values(&samples.render_luma, render_range);
    let reference_normalized = normalize_values(&samples.reference_luma, reference_range);
    let mut render_grazing = Vec::new();
    let mut reference_grazing = Vec::new();
    let mut render_left_sum = 0.0_f32;
    let mut render_left_count = 0.0_f32;
    let mut render_right_sum = 0.0_f32;
    let mut render_right_count = 0.0_f32;
    let mut reference_left_sum = 0.0_f32;
    let mut reference_left_count = 0.0_f32;
    let mut reference_right_sum = 0.0_f32;
    let mut reference_right_count = 0.0_f32;

    for index in 0..render_normalized.len() {
        let no_v = samples.no_v[index];
        if !(0.08..=0.42).contains(&no_v) || samples.screen_x_sign[index].abs() <= 0.0 {
            continue;
        }

        let render = render_normalized[index];
        let reference = reference_normalized[index];
        render_grazing.push(render);
        reference_grazing.push(reference);
        if samples.screen_x_sign[index] < 0.0 {
            render_left_sum += render;
            render_left_count += 1.0;
            reference_left_sum += reference;
            reference_left_count += 1.0;
        } else {
            render_right_sum += render;
            render_right_count += 1.0;
            reference_right_sum += reference;
            reference_right_count += 1.0;
        }
    }

    MirrorSphereSourceReferenceGrazingStats {
        sample_count: render_grazing.len(),
        correlation: pearson_correlation(&render_grazing, &reference_grazing),
        render_right_left_ratio: (render_right_sum / render_right_count.max(1.0))
            / (render_left_sum / render_left_count.max(1.0)).max(0.001),
        reference_right_left_ratio: (reference_right_sum / reference_right_count.max(1.0))
            / (reference_left_sum / reference_left_count.max(1.0)).max(0.001),
    }
}

#[derive(Clone, Copy, Debug)]
struct MirrorSphereHit {
    no_v: f32,
    reflection_dir: [f32; 3],
}

fn legacy_front_mirror_sphere_hit(
    ndc: [f32; 2],
    aspect: f32,
    projection_mode: ProjectionMode,
) -> Option<MirrorSphereHit> {
    let (ray_origin, ray_dir, view_dir) = match projection_mode {
        ProjectionMode::Orthographic => {
            let half_width = MIRROR_CAMERA_ORTHO_HALF_HEIGHT * aspect.max(0.001);
            (
                [
                    ndc[0] * half_width,
                    ndc[1] * MIRROR_CAMERA_ORTHO_HALF_HEIGHT,
                    MIRROR_ORTHOGRAPHIC_CAMERA_Z,
                ],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, 1.0],
            )
        }
        ProjectionMode::Perspective => {
            let tan_half_fov = (MIRROR_CAMERA_FOV_Y_RADIANS * 0.5).tan();
            (
                [0.0, 0.0, MIRROR_PERSPECTIVE_CAMERA_Z],
                normalize3([
                    ndc[0] * aspect.max(0.001) * tan_half_fov,
                    ndc[1] * tan_half_fov,
                    -1.0,
                ]),
                [0.0, 0.0, 0.0],
            )
        }
    };

    let hit_position = intersect_sphere(
        ray_origin,
        ray_dir,
        MIRROR_SPHERE_CENTER,
        MIRROR_SPHERE_RADIUS,
    )?;
    let normal = normalize3(sub3(hit_position, MIRROR_SPHERE_CENTER));
    let view_dir = match projection_mode {
        ProjectionMode::Orthographic => view_dir,
        ProjectionMode::Perspective => {
            normalize3(sub3([0.0, 0.0, MIRROR_PERSPECTIVE_CAMERA_Z], hit_position))
        }
    };
    let no_v = dot3(normal, view_dir).clamp(0.0, 1.0);
    Some(MirrorSphereHit {
        no_v,
        reflection_dir: reflect3(neg3(view_dir), normal),
    })
}

fn mirror_sphere_hit(
    ndc: [f32; 2],
    aspect: f32,
    camera_view: SinglePbrSphereCameraView,
) -> Option<MirrorSphereHit> {
    let (forward, right, up) = mirror_sphere_camera_basis(camera_view);
    let eye = camera_view.eye;
    let (ray_origin, ray_dir) = match camera_view.projection_mode {
        ProjectionMode::Orthographic => {
            let half_height = camera_view.ortho_size;
            let half_width = half_height * aspect.max(0.001);
            (
                add3(
                    add3(eye, mul3(right, ndc[0] * half_width)),
                    mul3(up, ndc[1] * half_height),
                ),
                forward,
            )
        }
        ProjectionMode::Perspective => {
            let tan_half_fov = (MIRROR_CAMERA_FOV_Y_RADIANS * 0.5).tan();
            (
                eye,
                normalize3(add3(
                    add3(
                        forward,
                        mul3(right, ndc[0] * aspect.max(0.001) * tan_half_fov),
                    ),
                    mul3(up, ndc[1] * tan_half_fov),
                )),
            )
        }
    };

    let hit_position = intersect_sphere(
        ray_origin,
        ray_dir,
        MIRROR_SPHERE_CENTER,
        MIRROR_SPHERE_RADIUS,
    )?;
    let normal = normalize3(sub3(hit_position, MIRROR_SPHERE_CENTER));
    let view_dir = match camera_view.projection_mode {
        ProjectionMode::Orthographic => neg3(forward),
        ProjectionMode::Perspective => normalize3(sub3(eye, hit_position)),
    };
    let no_v = dot3(normal, view_dir).clamp(0.0, 1.0);
    Some(MirrorSphereHit {
        no_v,
        reflection_dir: reflect3(neg3(view_dir), normal),
    })
}

fn mirror_sphere_camera_basis(
    camera_view: SinglePbrSphereCameraView,
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let forward = normalize3(sub3(camera_view.target, camera_view.eye));
    let mut right = cross3(forward, [0.0, 1.0, 0.0]);
    if dot3(right, right) <= 0.000001 {
        right = [1.0, 0.0, 0.0];
    }
    let right = normalize3(right);
    let up = normalize3(cross3(right, forward));
    (forward, right, up)
}

fn intersect_sphere(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    center: [f32; 3],
    radius: f32,
) -> Option<[f32; 3]> {
    let oc = sub3(ray_origin, center);
    let a = dot3(ray_dir, ray_dir);
    let b = 2.0 * dot3(oc, ray_dir);
    let c = dot3(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let t0 = (-b - sqrt_discriminant) / (2.0 * a);
    let t1 = (-b + sqrt_discriminant) / (2.0 * a);
    let t = if t0 > 0.0 { t0 } else { t1 };
    if t <= 0.0 {
        return None;
    }

    Some(add3(ray_origin, mul3(ray_dir, t)))
}

fn pixel_ndc(frame: &ViewportFrame, x: u32, y: u32) -> [f32; 2] {
    [
        ((x as f32 + 0.5) / frame.width.max(1) as f32) * 2.0 - 1.0,
        1.0 - ((y as f32 + 0.5) / frame.height.max(1) as f32) * 2.0,
    ]
}

fn sample_environment_pmrem_mip0(
    environment: &SourceCubemapEnvironment,
    direction: [f32; 3],
) -> [f32; 3] {
    let mip_chain = &environment.mip_chain;
    let texel = sample_cubemap_linear_at_mip(
        mip_chain.pmrem_texels(),
        mip_chain.pmrem_face_size(),
        mip_chain.pmrem_mip_count(),
        direction,
        0,
    );
    [texel[0], texel[1], texel[2]]
}

fn sample_cubemap_linear_at_mip(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [f32; 3],
    mip_level: u32,
) -> [f32; 4] {
    let (face, scaled_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let texel_x = (scaled_uv[0] * 0.5 + 0.5) * mip_size as f32 - 0.5;
    let texel_y = (scaled_uv[1] * 0.5 + 0.5) * mip_size as f32 - 0.5;
    let x0 = texel_x.floor();
    let y0 = texel_y.floor();
    let tx = texel_x - x0;
    let ty = texel_y - y0;
    let x0 = x0 as i32;
    let y0 = y0 as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let c00 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x0, y0);
    let c10 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x1, y0);
    let c01 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x0, y1);
    let c11 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x1, y1);
    lerp4(lerp4(c00, c10, tx), lerp4(c01, c11, tx), ty)
}

fn sample_cubemap_texel_unwrapped(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
    x: i32,
    y: i32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let mip_size_i32 = mip_size as i32;
    if x >= 0 && x < mip_size_i32 && y >= 0 && y < mip_size_i32 {
        return mip_texel(
            texels, face_size, mip_count, face, mip_level, x as u32, y as u32,
        );
    }

    let scaled_uv = [
        ((x as f32 + 0.5) / mip_size as f32) * 2.0 - 1.0,
        ((y as f32 + 0.5) / mip_size as f32) * 2.0 - 1.0,
    ];
    let direction = cubemap_direction_from_scaled_uv(face, scaled_uv);
    let (sample_face, sample_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let sample_x = texel_coord_from_scaled_axis(sample_uv[0], mip_size);
    let sample_y = texel_coord_from_scaled_axis(sample_uv[1], mip_size);
    mip_texel(
        texels,
        face_size,
        mip_count,
        sample_face,
        mip_level,
        sample_x,
        sample_y,
    )
}

fn mip_texel(
    texels: &[[f32; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [f32; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level);
    texels[offset + y as usize * mip_size as usize + x as usize]
}

fn lerp4(left: [f32; 4], right: [f32; 4], t: f32) -> [f32; 4] {
    [
        left[0] + (right[0] - left[0]) * t,
        left[1] + (right[1] - left[1]) * t,
        left[2] + (right[2] - left[2]) * t,
        left[3] + (right[3] - left[3]) * t,
    ]
}

fn texel_coord_from_scaled_axis(scaled_axis: f32, face_size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * face_size as f32 - 0.5).round() as i32)
        .clamp(0, face_size.saturating_sub(1) as i32) as u32
}

fn reference_display_luma(rgb: [f32; 3], intensity: f32) -> f32 {
    let mapped = rgb.map(|channel| {
        let hdr = (channel * intensity).max(0.0);
        (hdr / (1.0 + hdr)).powf(1.0 / 2.2) * 255.0
    });
    super::luma(mapped)
}

fn percentile_range(values: &[f32], low: f32, high: f32) -> (f32, f32) {
    let mut sorted = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.total_cmp(right));
    if sorted.is_empty() {
        return (0.0, 1.0);
    }
    let low_index = percentile_index(sorted.len(), low);
    let high_index = percentile_index(sorted.len(), high);
    let low_value = sorted[low_index];
    let high_value = sorted[high_index].max(low_value + 0.001);
    (low_value, high_value)
}

fn percentile_index(len: usize, percentile: f32) -> usize {
    ((len.saturating_sub(1)) as f32 * percentile.clamp(0.0, 1.0)).round() as usize
}

fn normalize_values(values: &[f32], range: (f32, f32)) -> Vec<f32> {
    let width = (range.1 - range.0).max(0.001);
    values
        .iter()
        .map(|value| ((*value - range.0) / width).clamp(0.0, 1.0))
        .collect()
}

fn pearson_correlation(first: &[f32], second: &[f32]) -> f32 {
    if first.len() != second.len() || first.is_empty() {
        return 0.0;
    }
    let count = first.len() as f32;
    let first_mean = first.iter().sum::<f32>() / count;
    let second_mean = second.iter().sum::<f32>() / count;
    let mut covariance = 0.0_f32;
    let mut first_variance = 0.0_f32;
    let mut second_variance = 0.0_f32;
    for index in 0..first.len() {
        let first_delta = first[index] - first_mean;
        let second_delta = second[index] - second_mean;
        covariance += first_delta * second_delta;
        first_variance += first_delta * first_delta;
        second_variance += second_delta * second_delta;
    }
    covariance / (first_variance.sqrt() * second_variance.sqrt()).max(0.000001)
}

fn global_ssim(first: &[f32], second: &[f32]) -> f32 {
    if first.len() != second.len() || first.is_empty() {
        return 0.0;
    }
    let count = first.len() as f32;
    let first_mean = first.iter().sum::<f32>() / count;
    let second_mean = second.iter().sum::<f32>() / count;
    let mut first_variance = 0.0_f32;
    let mut second_variance = 0.0_f32;
    let mut covariance = 0.0_f32;
    for index in 0..first.len() {
        let first_delta = first[index] - first_mean;
        let second_delta = second[index] - second_mean;
        first_variance += first_delta * first_delta;
        second_variance += second_delta * second_delta;
        covariance += first_delta * second_delta;
    }
    first_variance /= count;
    second_variance /= count;
    covariance /= count;

    let c1 = 0.01 * 0.01;
    let c2 = 0.03 * 0.03;
    ((2.0 * first_mean * second_mean + c1) * (2.0 * covariance + c2))
        / ((first_mean * first_mean + second_mean * second_mean + c1)
            * (first_variance + second_variance + c2))
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn mul3(value: [f32; 3], scalar: f32) -> [f32; 3] {
    [value[0] * scalar, value[1] * scalar, value[2] * scalar]
}

fn neg3(value: [f32; 3]) -> [f32; 3] {
    [-value[0], -value[1], -value[2]]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let len_sq = dot3(value, value);
    if len_sq <= f32::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    mul3(value, 1.0 / len_sq.sqrt())
}

fn reflect3(incident: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    sub3(incident, mul3(normal, 2.0 * dot3(incident, normal)))
}

#[derive(Clone, Copy, Debug)]
struct MirrorSphereReflectionStats {
    luma_stddev: f32,
    mean_saturation: f32,
    clipped_ratio: f32,
}

#[derive(Clone, Copy, Debug)]
struct MirrorSphereGrazingBalanceStats {
    left_luma: f32,
    right_luma: f32,
    left_luma_stddev: f32,
    right_luma_stddev: f32,
}

fn mirror_sphere_reflection_stats(frame: &ViewportFrame) -> MirrorSphereReflectionStats {
    let center_x = frame.width as f32 * 0.5;
    let center_y = frame.height as f32 * 0.52;
    let radius_x = frame.width as f32 * 0.215;
    let radius_y = frame.height as f32 * 0.205;
    let mut luma_sum = 0.0_f32;
    let mut luma_sq_sum = 0.0_f32;
    let mut saturation_sum = 0.0_f32;
    let mut clipped_count = 0.0_f32;
    let mut count = 0.0_f32;

    let x_min = (center_x - radius_x).max(0.0) as u32;
    let x_max = (center_x + radius_x).min(frame.width.saturating_sub(1) as f32) as u32;
    let y_min = (center_y - radius_y).max(0.0) as u32;
    let y_max = (center_y + radius_y).min(frame.height.saturating_sub(1) as f32) as u32;

    for y in (y_min..=y_max).step_by(3) {
        for x in (x_min..=x_max).step_by(3) {
            let normalized_x = (x as f32 - center_x) / radius_x.max(1.0);
            let normalized_y = (y as f32 - center_y) / radius_y.max(1.0);
            if normalized_x * normalized_x + normalized_y * normalized_y > 1.0 {
                continue;
            }
            let rgb = frame_pixel_rgb(frame, x, y);
            let value = super::luma(rgb);
            luma_sum += value;
            luma_sq_sum += value * value;
            saturation_sum += rgb_saturation(rgb);
            if rgb[0] > 248.0 || rgb[1] > 248.0 || rgb[2] > 248.0 {
                clipped_count += 1.0;
            }
            count += 1.0;
        }
    }

    let mean = luma_sum / count.max(1.0);
    let variance = (luma_sq_sum / count.max(1.0) - mean * mean).max(0.0);
    MirrorSphereReflectionStats {
        luma_stddev: variance.sqrt(),
        mean_saturation: saturation_sum / count.max(1.0),
        clipped_ratio: clipped_count / count.max(1.0),
    }
}

fn mirror_sphere_grazing_balance_stats(frame: &ViewportFrame) -> MirrorSphereGrazingBalanceStats {
    let center_x = frame.width as f32 * 0.5;
    let center_y = frame.height as f32 * 0.52;
    let radius_x = frame.width as f32 * 0.29;
    let radius_y = frame.height as f32 * 0.29;
    let left =
        mirror_sphere_side_grazing_stats(frame, center_x, center_y, radius_x, radius_y, -1.0);
    let right =
        mirror_sphere_side_grazing_stats(frame, center_x, center_y, radius_x, radius_y, 1.0);

    MirrorSphereGrazingBalanceStats {
        left_luma: left.mean_luma,
        right_luma: right.mean_luma,
        left_luma_stddev: left.luma_stddev,
        right_luma_stddev: right.luma_stddev,
    }
}

#[derive(Clone, Copy, Debug)]
struct MirrorSphereSideGrazingStats {
    mean_luma: f32,
    luma_stddev: f32,
}

fn mirror_sphere_side_grazing_stats(
    frame: &ViewportFrame,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    side_sign: f32,
) -> MirrorSphereSideGrazingStats {
    let x_min = (center_x - radius_x).max(0.0) as u32;
    let x_max = (center_x + radius_x).min(frame.width.saturating_sub(1) as f32) as u32;
    let y_min = (center_y - radius_y).max(0.0) as u32;
    let y_max = (center_y + radius_y).min(frame.height.saturating_sub(1) as f32) as u32;
    let mut luma_sum = 0.0_f32;
    let mut luma_sq_sum = 0.0_f32;
    let mut count = 0.0_f32;

    for y in (y_min..=y_max).step_by(3) {
        for x in (x_min..=x_max).step_by(3) {
            let normalized_x = (x as f32 - center_x) / radius_x.max(1.0);
            let normalized_y = (y as f32 - center_y) / radius_y.max(1.0);
            let radial_distance = normalized_x * normalized_x + normalized_y * normalized_y;
            if !(0.55..=1.0).contains(&radial_distance) || normalized_x * side_sign < 0.45 {
                continue;
            }
            let luma = super::luma(frame_pixel_rgb(frame, x, y));
            luma_sum += luma;
            luma_sq_sum += luma * luma;
            count += 1.0;
        }
    }

    let mean = luma_sum / count.max(1.0);
    let variance = (luma_sq_sum / count.max(1.0) - mean * mean).max(0.0);
    MirrorSphereSideGrazingStats {
        mean_luma: mean,
        luma_stddev: variance.sqrt(),
    }
}

fn rgb_saturation(rgb: [f32; 3]) -> f32 {
    let max_channel = rgb[0].max(rgb[1]).max(rgb[2]);
    let min_channel = rgb[0].min(rgb[1]).min(rgb[2]);
    if max_channel <= 0.0 {
        0.0
    } else {
        (max_channel - min_channel) / max_channel
    }
}

fn rgb_distance(first: [f32; 3], second: [f32; 3]) -> f32 {
    let dr = first[0] - second[0];
    let dg = first[1] - second[1];
    let db = first[2] - second[2];
    (dr * dr + dg * dg + db * db).sqrt()
}
