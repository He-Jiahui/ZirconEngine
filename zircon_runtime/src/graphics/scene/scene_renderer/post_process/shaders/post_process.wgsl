struct PostProcessParams {
    viewport_and_clusters: vec4<u32>,
    cluster_dimensions: vec4<u32>,
    feature_flags: vec4<u32>,
    lighting_flags: vec4<u32>,
    hybrid_gi_counts: vec4<u32>,
    hybrid_gi_source_ledger: vec4<u32>,
    anti_alias: vec4<u32>,
    blends: vec4<f32>,
    grading: vec4<f32>,
    tint_and_probe: vec4<f32>,
    hybrid_gi_color_and_intensity: vec4<f32>,
    baked_color_and_intensity: vec4<f32>,
    effect_flags: vec4<u32>,
    effect_tonemap_lut: vec4<f32>,
    effect_blur_dof: vec4<f32>,
    effect_dof_lens: vec4<f32>,
    effect_vignette_grain: vec4<f32>,
    effect_chromatic_fog: vec4<f32>,
    effect_fog_color: vec4<f32>,
    effect_dither_ssr: vec4<f32>,
    effect_ssr_limits: vec4<f32>,
    effect_depth: vec4<f32>,
    effect_projection: vec4<f32>,
    effect_view_x: vec4<f32>,
    effect_view_y: vec4<f32>,
    effect_view_z: vec4<f32>,
    effect_motion_blur: vec4<f32>,
};

struct ReflectionProbe {
    screen_uv_and_radius: vec4<f32>,
    color_and_intensity: vec4<f32>,
};

struct HybridGiProbe {
    screen_uv_and_radius: vec4<f32>,
    irradiance_and_intensity: vec4<f32>,
    hierarchy_irradiance_rgb_and_weight: vec4<f32>,
    hierarchy_rt_lighting_rgb_and_weight: vec4<f32>,
    temporal_signature_and_padding: vec4<f32>,
};

struct HybridGiTraceRegion {
    screen_uv_and_radius: vec4<f32>,
    boost_and_coverage: vec4<f32>,
    rt_lighting_rgb_and_weight: vec4<f32>,
};

@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var ambient_occlusion_tex: texture_2d<f32>;
@group(0) @binding(2) var history_scene_color_tex: texture_2d<f32>;
@group(0) @binding(3) var bloom_tex: texture_2d<f32>;
@group(0) @binding(4) var<uniform> params: PostProcessParams;
@group(0) @binding(5) var<storage, read> cluster_buffer: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> reflection_probe_buffer: array<ReflectionProbe>;
@group(0) @binding(7) var<storage, read> hybrid_gi_probe_buffer: array<HybridGiProbe>;
@group(0) @binding(8) var<storage, read> hybrid_gi_trace_region_buffer: array<HybridGiTraceRegion>;
@group(0) @binding(9) var history_global_illumination_tex: texture_2d<f32>;
@group(0) @binding(10) var effect_lut_tex: texture_2d<f32>;
@group(0) @binding(11) var scene_depth_tex: texture_depth_2d;
@group(0) @binding(12) var effect_lut_3d_tex: texture_3d<f32>;
@group(0) @binding(13) var effect_lut_sampler: sampler;
@group(0) @binding(14) var scene_normal_tex: texture_2d<f32>;
@group(0) @binding(15) var scene_depth_sampler: sampler;
@group(0) @binding(16) var scene_material_tex: texture_2d<f32>;
@group(0) @binding(17) var depth_of_field_coc_tex: texture_2d<f32>;
@group(0) @binding(18) var depth_of_field_bokeh_tex: texture_2d<f32>;
@group(0) @binding(19) var motion_vector_neighbor_max_tex: texture_2d<f32>;
@group(0) @binding(20) var history_screen_space_reflection_tex: texture_2d<f32>;
@group(0) @binding(21) var resolved_screen_space_reflection_tex: texture_2d<f32>;
@group(0) @binding(22) var screen_space_reflection_specular_occlusion_tex: texture_2d<f32>;
@group(0) @binding(23) var screen_space_reflection_depth_pyramid_tex: texture_2d<f32>;
@group(0) @binding(24) var screen_space_reflection_reflection_pyramid_tex: texture_2d<f32>;
@group(0) @binding(25) var screen_space_reflection_depth_pyramid_coarse_tex: texture_2d<f32>;
@group(0) @binding(26) var screen_space_reflection_reflection_pyramid_coarse_tex: texture_2d<f32>;
@group(0) @binding(27) var contact_shadow_tex: texture_2d<f32>;
@group(0) @binding(28) var<storage, read> exposure_buffer: array<vec4<f32>, 1>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct FragmentOutput {
    @location(0) final_color: vec4<f32>,
    @location(1) global_illumination: vec4<f32>,
};

const HYBRID_GI_HISTORY_SUPPORT_REUSE_START: f32 = 0.2;
const HYBRID_GI_HISTORY_SUPPORT_REUSE_RANGE: f32 = 0.45;
const HYBRID_GI_HISTORY_RESOLVE_CONFIDENCE_START: f32 = 0.6;
const HYBRID_GI_HISTORY_RESOLVE_CONFIDENCE_RANGE: f32 = 0.65;
const HYBRID_GI_HISTORY_CONTINUATION_CONFIDENCE_SCALE: f32 = 1.0;
const HYBRID_GI_HISTORY_SCENE_TRUTH_CONFIDENCE_RANGE: f32 = 0.45;
const HYBRID_GI_HISTORY_CONFIDENCE_BLEND_BASE: f32 = 0.05;
const HYBRID_GI_HISTORY_CONFIDENCE_BLEND_RANGE: f32 = 1.0;
const HYBRID_GI_HISTORY_BLEND_MAX: f32 = 0.45;
const HYBRID_GI_HISTORY_SIGNATURE_SCALE: f32 = 255.0;
const HYBRID_GI_SOURCE_FULL_DYNAMIC: u32 = 1u;
const HYBRID_GI_SOURCE_BAKED_BASELINE: u32 = 2u;
const HYBRID_GI_SOURCE_DYNAMIC_DELTA: u32 = 4u;
const DOF_BOKEH_SAMPLE_COUNT: u32 = 12u;
const DOF_BOKEH_RING_SAMPLE_COUNT: u32 = 6u;
const DOF_MAX_FINAL_PASS_RADIUS: f32 = 12.0;
const MOTION_BLUR_MAX_SAMPLES: u32 = 32u;
const MOTION_BLUR_MIN_PIXEL_SPEED_SQUARED: f32 = 1.0;
const TAU: f32 = 6.283185307179586;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn apply_color_grading(color: vec3<f32>) -> vec3<f32> {
    let exposure = params.grading.x;
    let contrast = params.grading.y;
    let saturation = params.grading.z;
    let gamma = params.grading.w;
    var graded = color * exposure;
    let luma = dot(graded, vec3<f32>(0.2126, 0.7152, 0.0722));
    graded = mix(vec3<f32>(luma), graded, saturation);
    graded = ((graded - vec3<f32>(0.5)) * contrast) + vec3<f32>(0.5);
    graded = max(graded, vec3<f32>(0.0));
    graded = pow(graded, vec3<f32>(1.0 / max(gamma, 0.001)));
    return graded * params.tint_and_probe.xyz;
}

fn color_luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

fn hybrid_gi_probe_source_is_valid(source_mask: u32) -> bool {
    let has_full_dynamic = (source_mask & HYBRID_GI_SOURCE_FULL_DYNAMIC) != 0u;
    let has_baked_baseline = (source_mask & HYBRID_GI_SOURCE_BAKED_BASELINE) != 0u;
    let has_dynamic_delta = (source_mask & HYBRID_GI_SOURCE_DYNAMIC_DELTA) != 0u;
    return (has_full_dynamic && !has_baked_baseline && !has_dynamic_delta)
        || (!has_full_dynamic && has_dynamic_delta);
}

fn viewport_size() -> vec2<u32> {
    return max(params.viewport_and_clusters.xy, vec2<u32>(1u, 1u));
}

fn viewport_origin() -> vec2<u32> {
    return params.viewport_and_clusters.zw;
}

fn scene_color_origin() -> vec2<u32> {
    return params.cluster_dimensions.zw;
}

fn local_coord(position: vec4<f32>) -> vec2<u32> {
    let size = viewport_size();
    return min(vec2<u32>(position.xy), size - vec2<u32>(1u, 1u));
}

fn physical_coord(coord: vec2<u32>) -> vec2<u32> {
    return viewport_origin() + coord;
}

fn physical_coord_i32(coord: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(viewport_origin()) + coord;
}

fn scene_color_coord_i32(coord: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(scene_color_origin()) + coord;
}

fn load_scene_color(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec4<f32> {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    return textureLoad(scene_color_tex, scene_color_coord_i32(clamped), 0);
}

fn load_scene_rgb(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec3<f32> {
    return load_scene_color(coord, viewport_size).rgb;
}

fn load_scene_depth(coord: vec2<i32>, viewport_size: vec2<u32>) -> f32 {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    let physical_coord = physical_coord_i32(clamped);
    // Keep depth reads derivative-free when SSR calls this helper from data-dependent loops.
    return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);
}

fn linearize_scene_depth(raw_depth: f32) -> f32 {
    let near_plane = max(params.effect_depth.x, 0.001);
    let far_plane = max(params.effect_depth.y, near_plane + 0.001);
    if (params.effect_depth.w > 0.5) {
        return (near_plane * far_plane)
            / max(far_plane - raw_depth * (far_plane - near_plane), 0.001);
    }
    return mix(near_plane, far_plane, raw_depth);
}

fn normalized_view_depth(view_depth: f32) -> f32 {
    return clamp((view_depth - params.effect_depth.x) * params.effect_depth.z, 0.0, 1.0);
}

fn load_scene_view_depth(coord: vec2<i32>, viewport_size: vec2<u32>) -> f32 {
    return linearize_scene_depth(load_scene_depth(coord, viewport_size));
}

fn apply_fxaa(coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let coord_i32 = vec2<i32>(coord);
    let north = load_scene_rgb(coord_i32 + vec2<i32>(0, -1), viewport_size);
    let south = load_scene_rgb(coord_i32 + vec2<i32>(0, 1), viewport_size);
    let west = load_scene_rgb(coord_i32 + vec2<i32>(-1, 0), viewport_size);
    let east = load_scene_rgb(coord_i32 + vec2<i32>(1, 0), viewport_size);

    let luma_center = color_luminance(color);
    let luma_north = color_luminance(north);
    let luma_south = color_luminance(south);
    let luma_west = color_luminance(west);
    let luma_east = color_luminance(east);
    let luma_min = min(luma_center, min(min(luma_north, luma_south), min(luma_west, luma_east)));
    let luma_max = max(luma_center, max(max(luma_north, luma_south), max(luma_west, luma_east)));
    let luma_range = luma_max - luma_min;
    if (luma_range < 0.03125) {
        return color;
    }

    let horizontal_edge = abs(luma_north + luma_south - 2.0 * luma_center);
    let vertical_edge = abs(luma_east + luma_west - 2.0 * luma_center);
    var neighbor_average = (east + west) * 0.5;
    if (horizontal_edge >= vertical_edge) {
        neighbor_average = (north + south) * 0.5;
    }

    let blend = clamp(luma_range * 1.5, 0.0, 0.75);
    return mix(color, neighbor_average, blend);
}

fn depth_of_field_radius(scene_depth: f32) -> f32 {
    let focus_depth = max(params.effect_blur_dof.y, params.effect_depth.x);
    let focus_range = max(params.effect_dof_lens.y, 0.001);
    let focal_length_scale = clamp(params.effect_dof_lens.x / 50.0, 0.1, 6.0);
    let max_radius = max(params.effect_blur_dof.w, 0.0);
    let focus_error = abs(scene_depth - focus_depth) / focus_range;
    return clamp(
        focus_error * max(params.effect_blur_dof.z, 0.0) * focal_length_scale * max_radius,
        0.0,
        max_radius
    );
}

fn load_depth_of_field_coc(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec2<f32> {
    let max_coord = vec2<i32>(viewport_size) - vec2<i32>(1, 1);
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    let prepared_coc = textureLoad(depth_of_field_coc_tex, clamped, 0);
    return clamp(prepared_coc.rg, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
}

fn depth_of_field_coc_radius(prepared_coc: vec2<f32>) -> f32 {
    let normalized_radius = clamp(max(prepared_coc.x, prepared_coc.y), 0.0, 1.0);
    return normalized_radius * max(params.effect_blur_dof.w, 0.0);
}

fn dilated_depth_of_field_coc(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec2<f32> {
    let center_coc = load_depth_of_field_coc(coord, viewport_size);
    let north_coc = load_depth_of_field_coc(coord + vec2<i32>(0, -1), viewport_size);
    let south_coc = load_depth_of_field_coc(coord + vec2<i32>(0, 1), viewport_size);
    let west_coc = load_depth_of_field_coc(coord + vec2<i32>(-1, 0), viewport_size);
    let east_coc = load_depth_of_field_coc(coord + vec2<i32>(1, 0), viewport_size);

    return max(
        max(center_coc, north_coc),
        max(max(south_coc, west_coc), east_coc)
    );
}

fn load_depth_of_field_bokeh_seed(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec4<f32> {
    let max_coord = vec2<i32>(viewport_size) - vec2<i32>(1, 1);
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    return textureLoad(depth_of_field_bokeh_tex, clamped, 0);
}

fn bokeh_aperture_radius(angle: f32) -> f32 {
    let blade_count = clamp(round(params.effect_dof_lens.z), 3.0, 12.0);
    let sector = TAU / blade_count;
    let local_angle = (angle / sector - floor(angle / sector)) * sector - sector * 0.5;
    return clamp(cos(sector * 0.5) / max(cos(local_angle), 0.001), 0.65, 1.0);
}

fn dof_bokeh_sample_offset(sample_index: u32, radius: f32) -> vec2<i32> {
    let ring_index = sample_index / DOF_BOKEH_RING_SAMPLE_COUNT;
    let spoke_index = sample_index % DOF_BOKEH_RING_SAMPLE_COUNT;
    let ring_radius = mix(0.55, 1.0, f32(ring_index));
    let angle =
        ((f32(spoke_index) + f32(ring_index) * 0.5) / f32(DOF_BOKEH_RING_SAMPLE_COUNT))
        * TAU
        + params.effect_dof_lens.w;
    let aperture_radius = bokeh_aperture_radius(angle);
    let offset = vec2<f32>(cos(angle), sin(angle)) * radius * ring_radius * aperture_radius;
    return vec2<i32>(round(offset));
}

fn depth_of_field_bokeh_layer_weight(center_coc: vec2<f32>, sample_coc: vec2<f32>) -> f32 {
    let far_layer = min(center_coc.x, sample_coc.x);
    let near_layer = sample_coc.y;
    return clamp(max(far_layer, near_layer), 0.0, 1.0);
}

fn sample_depth_of_field_bokeh(
    coord: vec2<u32>,
    viewport_size: vec2<u32>,
    color: vec3<f32>,
    blur_radius: f32
) -> vec3<f32> {
    let coord_i32 = vec2<i32>(coord);
    var accumulated = color;
    var total_weight = 1.0;

    for (var sample_index = 0u; sample_index < DOF_BOKEH_SAMPLE_COUNT; sample_index = sample_index + 1u) {
        let offset = dof_bokeh_sample_offset(sample_index, blur_radius);
        var sample_weight = 1.0;
        if (sample_index < DOF_BOKEH_RING_SAMPLE_COUNT) {
            sample_weight = 0.75;
        }
        accumulated += load_scene_rgb(coord_i32 + offset, viewport_size) * sample_weight;
        total_weight += sample_weight;
    }

    return accumulated / max(total_weight, 0.001);
}

fn sample_prepared_depth_of_field_bokeh(
    coord: vec2<u32>,
    viewport_size: vec2<u32>,
    color: vec3<f32>,
    center_coc: vec2<f32>,
    blur_radius: f32
) -> vec4<f32> {
    let coord_i32 = vec2<i32>(coord);
    var accumulated = color;
    var total_weight = 1.0;
    var bokeh_weight = 0.0;

    for (var sample_index = 0u; sample_index < DOF_BOKEH_SAMPLE_COUNT; sample_index = sample_index + 1u) {
        let offset = dof_bokeh_sample_offset(sample_index, blur_radius);
        let sample_coord = coord_i32 + offset;
        let seed = load_depth_of_field_bokeh_seed(sample_coord, viewport_size);
        let sample_coc = load_depth_of_field_coc(sample_coord, viewport_size);
        var sample_weight = clamp(
            seed.a * depth_of_field_bokeh_layer_weight(center_coc, sample_coc),
            0.0,
            1.0
        );
        if (sample_index < DOF_BOKEH_RING_SAMPLE_COUNT) {
            sample_weight = sample_weight * 0.75;
        }
        accumulated += seed.rgb * sample_weight;
        total_weight += sample_weight;
        bokeh_weight += sample_weight;
    }

    return vec4<f32>(
        accumulated / max(total_weight, 0.001),
        clamp(bokeh_weight / max(total_weight, 0.001), 0.0, 1.0)
    );
}

fn apply_effect_blur_family(coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let coord_i32 = vec2<i32>(coord);
    let scene_depth = load_scene_view_depth(coord_i32, viewport_size);
    let scene_depth_radius = depth_of_field_radius(scene_depth);
    let prepared_coc = load_depth_of_field_coc(coord_i32, viewport_size);
    let dilated_coc = dilated_depth_of_field_coc(coord_i32, viewport_size);
    let prepared_coc_radius = depth_of_field_coc_radius(dilated_coc);
    let blur_radius = max(params.effect_blur_dof.x, max(prepared_coc_radius, scene_depth_radius));
    if (blur_radius <= 0.001) {
        return color;
    }

    let clamped_radius = clamp(blur_radius, 1.0, DOF_MAX_FINAL_PASS_RADIUS);
    let procedural_bokeh = sample_depth_of_field_bokeh(coord, viewport_size, color, clamped_radius);
    let prepared_bokeh = sample_prepared_depth_of_field_bokeh(
        coord,
        viewport_size,
        color,
        prepared_coc,
        clamped_radius
    );
    let prepared_weight = clamp(
        max(
            prepared_coc_radius / max(params.effect_blur_dof.w, 0.001),
            prepared_bokeh.a
        ),
        0.0,
        1.0
    );
    let bokeh = mix(procedural_bokeh, prepared_bokeh.rgb, prepared_weight);
    return mix(color, bokeh, clamp(clamped_radius / DOF_MAX_FINAL_PASS_RADIUS, 0.0, 1.0));
}

fn load_motion_vector_neighbor_max(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec2<f32> {
    let max_coord = vec2<i32>(viewport_size) - vec2<i32>(1, 1);
    let clamped = clamp(coord, vec2<i32>(0, 0), max_coord);
    return clamp(
        textureLoad(motion_vector_neighbor_max_tex, clamped, 0).rg,
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
}

fn motion_blur_sample_weight(center_motion: vec2<f32>, sample_motion: vec2<f32>) -> f32 {
    let center_length = length(center_motion);
    let sample_length = length(sample_motion);
    if (center_length <= 0.00001 || sample_length <= 0.00001) {
        return 0.0;
    }

    return clamp(abs(dot(center_motion, sample_motion) / (center_length * sample_length)), 0.0, 1.0);
}

fn motion_blur_depth_visibility(center_depth: f32, sample_depth: f32) -> f32 {
    let foreground_gap = (center_depth - sample_depth) / max(center_depth, params.effect_depth.x);
    return 1.0 - smoothstep(0.01, 0.05, foreground_gap);
}

fn apply_motion_blur_vector_gather(coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let shutter_fraction = max(params.effect_motion_blur.x, 0.0);
    if (shutter_fraction <= 0.001) {
        return color;
    }

    let sample_count = u32(clamp(params.effect_motion_blur.y, 1.0, f32(MOTION_BLUR_MAX_SAMPLES)));
    let coord_i32 = vec2<i32>(coord);
    let motion_vector = load_motion_vector_neighbor_max(coord_i32, viewport_size);
    let center_depth = load_scene_view_depth(coord_i32, viewport_size);
    let exposure_vector = clamp(
        motion_vector * shutter_fraction,
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
    let viewport_extent = vec2<f32>(viewport_size);
    let pixel_velocity = exposure_vector * viewport_extent;
    if (dot(pixel_velocity, pixel_velocity) < MOTION_BLUR_MIN_PIXEL_SPEED_SQUARED) {
        return color;
    }

    var accumulated = color;
    var weight_total = 1.0;
    for (var sample_index = 1u; sample_index <= MOTION_BLUR_MAX_SAMPLES; sample_index = sample_index + 1u) {
        if (sample_index > sample_count) {
            break;
        }

        let centered_phase = ((f32(sample_index) - 0.5) / f32(sample_count)) - 0.5;
        let sample_pixel = vec2<f32>(coord) + exposure_vector * centered_phase * viewport_extent;
        if (
            sample_pixel.x < 0.0
            || sample_pixel.y < 0.0
            || sample_pixel.x >= viewport_extent.x
            || sample_pixel.y >= viewport_extent.y
        ) {
            continue;
        }

        let sample_coord = vec2<i32>(round(sample_pixel));
        let sample_motion = load_motion_vector_neighbor_max(sample_coord, viewport_size);
        let sample_depth = load_scene_view_depth(sample_coord, viewport_size);
        let sample_weight =
            motion_blur_sample_weight(motion_vector, sample_motion)
            * motion_blur_depth_visibility(center_depth, sample_depth);
        if (sample_weight <= 0.001) {
            continue;
        }

        accumulated = accumulated + load_scene_rgb(sample_coord, viewport_size) * sample_weight;
        weight_total = weight_total + sample_weight;
    }

    return accumulated / max(weight_total, 0.001);
}

fn apply_chromatic_aberration(coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let intensity = params.effect_chromatic_fog.x;
    if (intensity <= 0.001) {
        return color;
    }
    let spread = max(params.effect_chromatic_fog.y, 1.0);
    let offset = i32(round(clamp(intensity * spread * 4.0, 1.0, 12.0)));
    let coord_i32 = vec2<i32>(coord);
    let red_sample = load_scene_rgb(coord_i32 + vec2<i32>(offset, 0), viewport_size);
    let blue_sample = load_scene_rgb(coord_i32 + vec2<i32>(-offset, 0), viewport_size);
    return vec3<f32>(red_sample.r, color.g, blue_sample.b);
}

fn apply_effect_fog(uv: vec2<f32>, coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let density = params.effect_chromatic_fog.z;
    if (density <= 0.001) {
        return color;
    }
    let height_factor = 1.0 + uv.y * max(params.effect_chromatic_fog.w, 0.0);
    let scene_depth = load_scene_view_depth(vec2<i32>(coord), viewport_size);
    let depth_factor = smoothstep(0.0, 1.0, normalized_view_depth(scene_depth));
    let fog_amount = clamp(density * height_factor * (0.25 + depth_factor), 0.0, 1.0);
    return mix(color, params.effect_fog_color.rgb, fog_amount);
}

fn apply_vignette(uv: vec2<f32>, color: vec3<f32>) -> vec3<f32> {
    let intensity = params.effect_vignette_grain.x;
    if (intensity <= 0.001) {
        return color;
    }
    let smoothness = max(params.effect_vignette_grain.y, 0.001);
    let roundness = max(params.effect_vignette_grain.z, 0.001);
    let centered = abs(uv * 2.0 - vec2<f32>(1.0, 1.0));
    let radius = length(centered * vec2<f32>(1.0, roundness));
    let mask = smoothstep(smoothness, 1.0, radius);
    return color * (1.0 - clamp(mask * intensity, 0.0, 1.0));
}

fn effect_noise(coord: vec2<u32>, seed: f32) -> f32 {
    let p = vec3<f32>(vec2<f32>(coord), seed);
    return fract(sin(dot(p, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
}

fn apply_grain_and_dither(coord: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let grain = params.effect_vignette_grain.w * max(params.effect_fog_color.w, 0.0);
    let dither = params.effect_dither_ssr.x / max(params.effect_dither_ssr.y, 1.0);
    let strength = grain + dither / 255.0;
    if (strength <= 0.0001) {
        return color;
    }
    let noise = effect_noise(coord, params.effect_dither_ssr.y) - 0.5;
    return max(color + vec3<f32>(noise * strength), vec3<f32>(0.0));
}

fn lut_axis_index(value: f32, size: u32) -> u32 {
    let max_index = max(size, 1u) - 1u;
    return u32(round(clamp(value, 0.0, 1.0) * f32(max_index)));
}

fn sample_effect_lut_1d_channel(value: f32) -> f32 {
    let dims = textureDimensions(effect_lut_tex);
    let x = i32(lut_axis_index(value, dims.x));
    return textureLoad(effect_lut_tex, vec2<i32>(x, 0), 0).r;
}

fn sample_effect_lut_1d(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        sample_effect_lut_1d_channel(color.r),
        sample_effect_lut_1d_channel(color.g),
        sample_effect_lut_1d_channel(color.b)
    );
}

fn sample_effect_lut_2d_strip(color: vec3<f32>) -> vec3<f32> {
    let dims = textureDimensions(effect_lut_tex);
    let size = max(dims.y, 1u);
    let red = lut_axis_index(color.r, size);
    let green = lut_axis_index(color.g, size);
    let blue = lut_axis_index(color.b, size);
    let x = min(blue * size + red, max(dims.x, 1u) - 1u);
    let y = min(green, size - 1u);
    return textureLoad(effect_lut_tex, vec2<i32>(i32(x), i32(y)), 0).rgb;
}

fn sample_effect_lut_3d(color: vec3<f32>) -> vec3<f32> {
    let dims_u32 = textureDimensions(effect_lut_3d_tex);
    let dims = vec3<f32>(f32(dims_u32.x), f32(dims_u32.y), f32(dims_u32.z));
    let axis_max = max(dims - vec3<f32>(1.0), vec3<f32>(0.0));
    let sample_coord = (clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * axis_max + vec3<f32>(0.5)) / dims;
    return textureSampleLevel(effect_lut_3d_tex, effect_lut_sampler, sample_coord, 0.0).rgb;
}

fn sample_effect_lut(color: vec3<f32>, binding_mode: u32) -> vec3<f32> {
    if (binding_mode == 3u) {
        return sample_effect_lut_3d(color);
    }
    if (binding_mode == 2u) {
        return sample_effect_lut_2d_strip(color);
    }
    return sample_effect_lut_1d(color);
}

fn apply_tonemap_and_lut(color: vec3<f32>) -> vec3<f32> {
    if (params.effect_flags.y == 4u) {
        return sample_effect_lut_3d(color);
    }

    let exposure = exp2(params.effect_tonemap_lut.x) * max(exposure_buffer[0].x, 0.0);
    let white_point = max(params.effect_tonemap_lut.y, 0.001);
    var mapped = max(color * exposure, vec3<f32>(0.0));
    if (params.effect_flags.x == 1u) {
        mapped = mapped / (vec3<f32>(1.0) + mapped / white_point);
    } else if (params.effect_flags.x == 2u) {
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;
        mapped = clamp((mapped * (a * mapped + vec3<f32>(b))) / (mapped * (c * mapped + vec3<f32>(d)) + vec3<f32>(e)), vec3<f32>(0.0), vec3<f32>(1.0));
    } else if (params.effect_flags.x == 3u) {
        mapped = max(vec3<f32>(0.0), mapped - vec3<f32>(0.004));
        mapped = (mapped * (6.2 * mapped + vec3<f32>(0.5))) / (mapped * (6.2 * mapped + vec3<f32>(1.7)) + vec3<f32>(0.06));
    }
    let lut_intensity = clamp(params.effect_tonemap_lut.z, 0.0, 1.0);
    if (params.effect_flags.y != 0u && lut_intensity > 0.0) {
        mapped = mix(mapped, sample_effect_lut(mapped, params.effect_flags.y), lut_intensity);
    }
    return mapped;
}

fn load_resolved_screen_space_reflection(coord: vec2<i32>, viewport_size: vec2<u32>) -> vec4<f32> {
    let max_coord = vec2<i32>(viewport_size - vec2<u32>(1u, 1u));
    let clamped_coord = clamp(coord, vec2<i32>(0, 0), max_coord);
    let resolved = textureLoad(resolved_screen_space_reflection_tex, clamped_coord, 0);
    return vec4<f32>(resolved.rgb, clamp(resolved.a, 0.0, 1.0));
}

fn apply_scene_composite(uv: vec2<f32>, coord: vec2<u32>, viewport_size: vec2<u32>, color: vec3<f32>) -> vec3<f32> {
    let coord_i32 = vec2<i32>(coord);
    let resolved_reflection =
        load_resolved_screen_space_reflection(coord_i32, viewport_size);
    let reflection_weight = resolved_reflection.a;
    var composited = mix(color, resolved_reflection.rgb, reflection_weight);
    composited = apply_effect_fog(uv, coord, viewport_size, composited);
    return composited;
}

@fragment
fn fs_depth_of_field(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = local_coord(position);
    let coord_i32 = vec2<i32>(coord);
    let scene_color = load_scene_color(coord_i32, viewport_size);
    let blurred = apply_effect_blur_family(coord, viewport_size, scene_color.rgb);
    return vec4<f32>(blurred, scene_color.a);
}

@fragment
fn fs_blur(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = local_coord(position);
    let coord_i32 = vec2<i32>(coord);
    let scene_color = load_scene_color(coord_i32, viewport_size);
    let blurred = apply_effect_blur_family(coord, viewport_size, scene_color.rgb);
    return vec4<f32>(blurred, scene_color.a);
}

@fragment
fn fs_motion_blur(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = local_coord(position);
    let coord_i32 = vec2<i32>(coord);
    let scene_color = load_scene_color(coord_i32, viewport_size);
    let blurred = apply_motion_blur_vector_gather(coord, viewport_size, scene_color.rgb);
    return vec4<f32>(blurred, scene_color.a);
}

@fragment
fn fs_scene_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = local_coord(position);
    let coord_i32 = vec2<i32>(coord);
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / vec2<f32>(viewport_size);
    let scene_color = load_scene_color(coord_i32, viewport_size);
    let composited = apply_scene_composite(uv, coord, viewport_size, scene_color.rgb);
    return vec4<f32>(composited, scene_color.a);
}

@fragment
fn fs_screen_space_reflection_resolve(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = viewport_size();
    let coord = local_coord(position);
    return resolve_screen_space_reflection_history(coord, viewport_size);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> FragmentOutput {
    let viewport_size = viewport_size();
    let cluster_dims = max(params.cluster_dimensions.xy, vec2<u32>(1u, 1u));
    let coord = local_coord(position);
    let coord_i32 = vec2<i32>(coord);
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / vec2<f32>(viewport_size);

    let scene_color = load_scene_color(coord_i32, viewport_size);
    var color = scene_color.rgb;

    if (params.lighting_flags.x != 0u) {
        let contact_shadow = textureLoad(contact_shadow_tex, coord_i32, 0).r;
        color = color * max(contact_shadow, 0.18);
    }

    if (params.feature_flags.y != 0u) {
        let tile_size = 16u;
        let tile = min(coord / vec2<u32>(tile_size, tile_size), cluster_dims - vec2<u32>(1u, 1u));
        let cluster_index = tile.y * cluster_dims.x + tile.x;
        let cluster = cluster_buffer[cluster_index];
        color = color * (1.0 + cluster.a * params.blends.y);
        color = color + cluster.rgb * cluster.a * params.blends.z;
    }

    if (params.feature_flags.z != 0u) {
        let history = textureLoad(history_scene_color_tex, physical_coord_i32(coord_i32), 0).rgb;
        color = mix(color, history, params.blends.x);
    }

    if (params.blends.w > 0.0) {
        let bloom = textureLoad(bloom_tex, coord_i32, 0).rgb;
        color = color + bloom * params.blends.w;
    }

    if (params.feature_flags.w > 0u) {
        for (var probe_index = 0u; probe_index < params.feature_flags.w; probe_index = probe_index + 1u) {
            let probe = reflection_probe_buffer[probe_index];
            let probe_uv = probe.screen_uv_and_radius.xy;
            let radius = max(probe.screen_uv_and_radius.z, 0.0001);
            let distance = distance(uv, probe_uv);
            let falloff = max(1.0 - distance / radius, 0.0);
            let influence = falloff * falloff * probe.color_and_intensity.w;
            color = color + probe.color_and_intensity.rgb * influence * params.tint_and_probe.w;
        }
    }

    var global_illumination_history = vec3<f32>(0.0);
    var indirect_light = vec3<f32>(0.0);
    var indirect_light_history_support = 0.0;
    var indirect_light_history_confidence = 0.0;
    var indirect_light_history_signature = 0.0;
    if (params.hybrid_gi_counts.x > 0u && params.hybrid_gi_color_and_intensity.w > 0.0) {
        var gi_light = vec3<f32>(0.0);
        for (var probe_index = 0u; probe_index < params.hybrid_gi_counts.x; probe_index = probe_index + 1u) {
            let probe = hybrid_gi_probe_buffer[probe_index];
            let probe_uv = probe.screen_uv_and_radius.xy;
            let probe_radius = max(probe.screen_uv_and_radius.z, 0.0001);
            let budget_weight = probe.screen_uv_and_radius.w;
            let hierarchy_resolve_weight = probe.irradiance_and_intensity.w;
            let probe_source_mask = u32(round(probe.temporal_signature_and_padding.z));
            let probe_dynamic_weight =
                clamp(probe.temporal_signature_and_padding.w, 0.0, 1.0)
                * select(0.0, 1.0, hybrid_gi_probe_source_is_valid(probe_source_mask));
            let distance_to_probe = distance(uv, probe_uv);
            let falloff = max(1.0 - distance_to_probe / probe_radius, 0.0);
            var trace_support = 1.0;
            var rt_lighting_sum =
                probe.hierarchy_rt_lighting_rgb_and_weight.rgb
                * probe.hierarchy_rt_lighting_rgb_and_weight.w;
            var rt_lighting_weight = probe.hierarchy_rt_lighting_rgb_and_weight.w;
            for (var trace_index = 0u; trace_index < params.hybrid_gi_counts.y; trace_index = trace_index + 1u) {
                let trace_region = hybrid_gi_trace_region_buffer[trace_index];
                let region_uv = trace_region.screen_uv_and_radius.xy;
                let region_radius = max(trace_region.screen_uv_and_radius.z, 0.0001);
                let pixel_region_distance = distance(uv, region_uv);
                let pixel_region_falloff = max(1.0 - pixel_region_distance / region_radius, 0.0);
                let probe_region_distance = distance(probe_uv, region_uv);
                let probe_region_reach = max(region_radius, 0.0001);
                let probe_region_falloff =
                    max(1.0 - probe_region_distance / probe_region_reach, 0.0);
                let region_support =
                    pixel_region_falloff * pixel_region_falloff
                    * probe_region_falloff * probe_region_falloff
                    * trace_region.boost_and_coverage.x
                    * trace_region.boost_and_coverage.y;
                trace_support = trace_support + region_support * 4.0;
                let rt_support = region_support * trace_region.rt_lighting_rgb_and_weight.w;
                rt_lighting_sum =
                    rt_lighting_sum + trace_region.rt_lighting_rgb_and_weight.rgb * rt_support;
                rt_lighting_weight = rt_lighting_weight + rt_support;
            }
            var probe_irradiance = probe.irradiance_and_intensity.rgb;
            if (probe.hierarchy_irradiance_rgb_and_weight.w > 0.0) {
                let hierarchy_irradiance = probe.hierarchy_irradiance_rgb_and_weight.rgb;
                let hierarchy_irradiance_mix =
                    clamp(probe.hierarchy_irradiance_rgb_and_weight.w, 0.0, 0.75);
                probe_irradiance =
                    mix(probe_irradiance, hierarchy_irradiance, hierarchy_irradiance_mix);
            }
            if (rt_lighting_weight > 0.0) {
                let rt_lighting_tint = rt_lighting_sum / rt_lighting_weight;
                let rt_mix = clamp(rt_lighting_weight * 0.45, 0.0, 0.65);
                probe_irradiance = mix(probe_irradiance, rt_lighting_tint, rt_mix);
            }
            let probe_history_support = falloff * falloff * budget_weight;
            let scene_truth_history_confidence =
                clamp(probe.temporal_signature_and_padding.y, 0.0, 1.0);
            let resolve_weight_history_t =
                clamp(
                    (hierarchy_resolve_weight - HYBRID_GI_HISTORY_RESOLVE_CONFIDENCE_START)
                    / HYBRID_GI_HISTORY_RESOLVE_CONFIDENCE_RANGE,
                    0.0,
                    1.0,
                );
            let resolve_weight_history_confidence =
                resolve_weight_history_t * resolve_weight_history_t
                * (3.0 - 2.0 * resolve_weight_history_t);
            let probe_history_confidence =
                resolve_weight_history_confidence
                * (
                    HYBRID_GI_HISTORY_CONTINUATION_CONFIDENCE_SCALE
                    + scene_truth_history_confidence
                        * HYBRID_GI_HISTORY_SCENE_TRUTH_CONFIDENCE_RANGE
                );
            if (probe_history_support > indirect_light_history_support) {
                indirect_light_history_signature = probe.temporal_signature_and_padding.x;
                indirect_light_history_confidence = probe_history_confidence;
            }
            indirect_light_history_support =
                max(indirect_light_history_support, probe_history_support);
            let probe_weight =
                falloff * falloff * budget_weight * hierarchy_resolve_weight * trace_support
                * probe_dynamic_weight;
            gi_light = gi_light + probe_irradiance * probe_weight;
        }

        let probe_count = max(f32(params.hybrid_gi_counts.x), 1.0);
        let source_ledger_valid = params.hybrid_gi_source_ledger.w != 0u;
        let dynamic_source_weight =
            f32(params.hybrid_gi_source_ledger.z) / 255.0 * select(0.0, 1.0, source_ledger_valid);
        indirect_light =
            (gi_light / probe_count)
            * params.hybrid_gi_color_and_intensity.w
            * dynamic_source_weight;
        if (params.hybrid_gi_counts.w != 0u) {
            let current_frame_lighting_sample =
                textureLoad(history_global_illumination_tex, coord_i32, 0);
            let current_frame_luminance =
                color_luminance(current_frame_lighting_sample.rgb);
            let current_frame_blend = clamp(current_frame_luminance * 1.5, 0.0, 0.65);
            indirect_light =
                mix(
                    indirect_light,
                    current_frame_lighting_sample.rgb
                        * params.hybrid_gi_color_and_intensity.w
                        * dynamic_source_weight,
                    current_frame_blend
                );
        }
        if (params.hybrid_gi_counts.z != 0u && params.hybrid_gi_counts.w == 0u) {
            let global_illumination_history_sample =
                textureLoad(history_global_illumination_tex, physical_coord_i32(coord_i32), 0);
            global_illumination_history = global_illumination_history_sample.rgb;
            let spatial_history_blend =
                params.blends.x
                * clamp(
                    (indirect_light_history_support - HYBRID_GI_HISTORY_SUPPORT_REUSE_START)
                    / HYBRID_GI_HISTORY_SUPPORT_REUSE_RANGE,
                    0.0,
                    1.0,
                );
            let current_signature_bucket =
                round(clamp(indirect_light_history_signature, 0.0, 1.0) * HYBRID_GI_HISTORY_SIGNATURE_SCALE);
            let history_signature_bucket =
                round(clamp(global_illumination_history_sample.a, 0.0, 1.0) * HYBRID_GI_HISTORY_SIGNATURE_SCALE);
            let signature_matches =
                current_signature_bucket > 0.0
                && history_signature_bucket > 0.0
                && current_signature_bucket == history_signature_bucket;
            let confidence_history_blend =
                clamp(
                    HYBRID_GI_HISTORY_CONFIDENCE_BLEND_BASE
                    + indirect_light_history_confidence
                        * HYBRID_GI_HISTORY_CONFIDENCE_BLEND_RANGE,
                    0.0,
                    1.0,
                );
            let history_blend =
                min(spatial_history_blend, HYBRID_GI_HISTORY_BLEND_MAX)
                * confidence_history_blend
                * select(0.0, 1.0, signature_matches || history_signature_bucket == 0.0);
            indirect_light = mix(indirect_light, global_illumination_history, history_blend);
        }
        color = color + indirect_light;
    }

    if (params.baked_color_and_intensity.w > 0.0) {
        color = color + params.baked_color_and_intensity.rgb * params.baked_color_and_intensity.w;
    }

    if (params.effect_flags.w != 0u) {
        color = apply_effect_blur_family(coord, viewport_size, color);
        color = apply_motion_blur_vector_gather(coord, viewport_size, color);
        color = apply_chromatic_aberration(coord, viewport_size, color);
        color = apply_scene_composite(uv, coord, viewport_size, color);
        color = apply_vignette(uv, color);
        color = apply_grain_and_dither(coord, color);
    }

    if (params.anti_alias.x != 0u) {
        color = apply_fxaa(coord, viewport_size, color);
    }

    color = apply_tonemap_and_lut(color);
    if (params.effect_flags.y != 4u) {
        color = apply_color_grading(color);
    }
    var output: FragmentOutput;
    output.final_color = vec4<f32>(color, scene_color.a);
    output.global_illumination = vec4<f32>(indirect_light, indirect_light_history_signature);
    return output;
}
