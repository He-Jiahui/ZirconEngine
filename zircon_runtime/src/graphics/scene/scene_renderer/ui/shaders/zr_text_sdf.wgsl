@group(0) @binding(0) var sdf_atlas: texture_2d_array<f32>;
@group(0) @binding(1) var distance_field_sampler: sampler;
@group(0) @binding(2) var msdf_atlas: texture_2d_array<f32>;

struct SdfTextMaterial {
    fill_color: vec4<f32>,
    outline_color: vec4<f32>,
    shadow_color: vec4<f32>,
    glow_color: vec4<f32>,
    effect_params: vec4<f32>,
    flags: vec4<u32>,
    projection_params: vec4<f32>,
};

@group(2) @binding(0) var<uniform> text_material: SdfTextMaterial;

const SDF_MODE: u32 = 0u;
const MSDF_MODE: u32 = 1u;
const MTSDF_MODE: u32 = 2u;
const SOLID_PRIMITIVE: u32 = 1u;
const OUTLINE_EFFECT: u32 = 1u;
const SHADOW_EFFECT: u32 = 2u;
const GLOW_EFFECT: u32 = 4u;
const FRAGMENT_DERIVED_RANGE: u32 = 1u;

struct VertexIn {
    @location(0) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) screen_px_range: f32,
    @location(4) atlas_px_range: f32,
    @location(5) page_index: u32,
    @location(6) decode_mode: u32,
    @location(7) primitive_kind: u32,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) screen_px_range: f32,
    @location(3) atlas_px_range: f32,
    @location(4) @interpolate(flat) page_index: u32,
    @location(5) @interpolate(flat) decode_mode: u32,
    @location(6) @interpolate(flat) primitive_kind: u32,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = input.position;
    out.uv = input.uv;
    out.color = input.color;
    out.screen_px_range = input.screen_px_range;
    out.atlas_px_range = input.atlas_px_range;
    out.page_index = input.page_index;
    out.decode_mode = input.decode_mode;
    out.primitive_kind = input.primitive_kind;
    return out;
}

fn resolved_screen_px_range(input: VertexOut) -> f32 {
    if text_material.flags.y != FRAGMENT_DERIVED_RANGE {
        return max(input.screen_px_range, 1.0);
    }
    let atlas_dimensions = max(text_material.projection_params.xy, vec2<f32>(1.0));
    let atlas_unit_range = vec2<f32>(max(input.atlas_px_range, 1.0)) / atlas_dimensions;
    let screen_texture_size = vec2<f32>(1.0)
        / max(fwidth(input.uv), vec2<f32>(0.000001));
    return max(0.5 * dot(atlas_unit_range, screen_texture_size), 1.0);
}

fn median3(value: vec3<f32>) -> f32 {
    return max(min(value.r, value.g), min(max(value.r, value.g), value.b));
}

fn sample_distances_at(input: VertexOut, uv: vec2<f32>) -> vec2<f32> {
    if input.decode_mode == SDF_MODE {
        let distance = textureSample(
            sdf_atlas,
            distance_field_sampler,
            uv,
            i32(input.page_index),
        ).r;
        return vec2<f32>(distance, distance);
    }

    let sample = textureSample(
        msdf_atlas,
        distance_field_sampler,
        uv,
        i32(input.page_index),
    );
    let fill_distance = median3(sample.rgb);
    // MTSDF fill intentionally uses the RGB median. Its alpha true distance is
    // carried independently for the outline/glow milestone.
    let true_distance = select(fill_distance, sample.a, input.decode_mode == MTSDF_MODE);
    return vec2<f32>(fill_distance, true_distance);
}

fn sdf_coverage_with_offset(distance: f32, screen_px_range: f32, expand_px: f32) -> f32 {
    let px_range = max(screen_px_range, 1.0);
    let signed_distance = (distance - 0.5) * px_range + expand_px;
    let aa_width = max(fwidth(distance) * px_range, 1.0);
    return clamp(signed_distance / aa_width + 0.5, 0.0, 1.0);
}

fn mtsdf_glow_coverage(true_distance: f32, screen_px_range: f32, radius_px: f32) -> f32 {
    let px_range = max(screen_px_range, 1.0);
    let signed_distance = (true_distance - 0.5) * px_range;
    let outside_fade = clamp(
        1.0 - max(-signed_distance, 0.0) / max(radius_px, 0.0001),
        0.0,
        1.0,
    );
    let fill_coverage = sdf_coverage_with_offset(true_distance, screen_px_range, 0.0);
    return outside_fade * (1.0 - fill_coverage);
}

fn straight_alpha_over(under: vec4<f32>, over: vec4<f32>) -> vec4<f32> {
    let alpha = over.a + under.a * (1.0 - over.a);
    if alpha <= 0.000001 {
        return vec4<f32>(0.0);
    }
    let under_factor = under.a * (1.0 - over.a);
    let rgb = (over.rgb * over.a + under.rgb * under_factor) / alpha;
    return vec4<f32>(rgb, alpha);
}

fn covered_color(color: vec4<f32>, coverage: f32) -> vec4<f32> {
    return vec4<f32>(color.rgb, color.a * clamp(coverage, 0.0, 1.0));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    if input.primitive_kind == SOLID_PRIMITIVE {
        return input.color;
    }
    let screen_px_range = resolved_screen_px_range(input);
    let distances = sample_distances_at(input, input.uv);
    let fill_coverage = sdf_coverage_with_offset(distances.x, screen_px_range, 0.0);
    var result = vec4<f32>(0.0);

    if (text_material.flags.x & GLOW_EFFECT) != 0u {
        let glow_coverage = mtsdf_glow_coverage(
            distances.y,
            screen_px_range,
            text_material.effect_params.w,
        );
        result = straight_alpha_over(
            result,
            covered_color(text_material.glow_color, glow_coverage),
        );
    }

    if (text_material.flags.x & SHADOW_EFFECT) != 0u {
        let offset = text_material.effect_params.yz;
        let shadow_uv = input.uv - dpdx(input.uv) * offset.x - dpdy(input.uv) * offset.y;
        let shadow_distances = sample_distances_at(input, shadow_uv);
        let shadow_coverage = sdf_coverage_with_offset(
            shadow_distances.x,
            screen_px_range,
            0.0,
        );
        result = straight_alpha_over(
            result,
            covered_color(text_material.shadow_color, shadow_coverage),
        );
    }

    if (text_material.flags.x & OUTLINE_EFFECT) != 0u {
        let expanded_coverage = sdf_coverage_with_offset(
            distances.x,
            screen_px_range,
            text_material.effect_params.x,
        );
        let outline_coverage = max(expanded_coverage - fill_coverage, 0.0);
        result = straight_alpha_over(
            result,
            covered_color(text_material.outline_color, outline_coverage),
        );
    }

    return straight_alpha_over(
        result,
        covered_color(text_material.fill_color, fill_coverage),
    );
}
