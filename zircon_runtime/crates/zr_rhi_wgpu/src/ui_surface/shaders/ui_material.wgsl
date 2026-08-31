struct SolidVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) local_position: vec2<f32>,
    @location(3) half_extent: vec2<f32>,
    @location(4) corner_radius: f32,
    @location(5) border_width: f32,
    @location(6) fill_color: vec4<f32>,
};

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) @interpolate(flat) half_extent: vec2<f32>,
    @location(3) @interpolate(flat) corner_radius: f32,
    @location(4) @interpolate(flat) border_width: f32,
    @location(5) @interpolate(flat) fill_color: vec4<f32>,
};

struct SolidFlatVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct SolidInstanceInput {
    @location(0) min_position: vec2<f32>,
    @location(1) max_position: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct ImageVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct ImageVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn damage_clear_vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn damage_clear_fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

fn material_tint(color: vec4<f32>, tint: vec4<f32>) -> vec4<f32> {
    return color * tint;
}

fn premultiply_alpha(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn srgb_to_linear_channel(value: f32) -> f32 {
    if value <= 0.04045 {
        return value / 12.92;
    }
    return pow((value + 0.055) / 1.055, 2.4);
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_to_linear_channel(color.r),
        srgb_to_linear_channel(color.g),
        srgb_to_linear_channel(color.b),
    );
}

fn linear_to_srgb_channel(value: f32) -> f32 {
    if value <= 0.0031308 {
        return value * 12.92;
    }
    return 1.055 * pow(value, 1.0 / 2.4) - 0.055;
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        linear_to_srgb_channel(color.r),
        linear_to_srgb_channel(color.g),
        linear_to_srgb_channel(color.b),
    );
}

fn rounded_box_distance(local_position: vec2<f32>, half_extent: vec2<f32>, radius: f32) -> f32 {
    let q = abs(local_position) - half_extent + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn rounded_box_alpha(signed_distance: f32, distance_width: f32) -> f32 {
    return 1.0 - smoothstep(
        distance_width * -0.5,
        distance_width * 0.5,
        signed_distance,
    );
}

fn material_solid_color(color: vec4<f32>, linear_target: bool) -> vec4<f32> {
    let tinted = material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0));
    if linear_target {
        return premultiply_alpha(vec4<f32>(srgb_to_linear(tinted.rgb), tinted.a));
    }
    return premultiply_alpha(tinted);
}

fn material_image_color(color: vec4<f32>, linear_target: bool) -> vec4<f32> {
    let tinted = material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0));
    if linear_target || tinted.a <= 0.0 {
        return tinted;
    }
    let straight_srgb = linear_to_srgb(clamp(tinted.rgb / tinted.a, vec3<f32>(0.0), vec3<f32>(1.0)));
    return vec4<f32>(straight_srgb * tinted.a, tinted.a);
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var output: SolidVertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.local_position = input.local_position;
    output.half_extent = input.half_extent;
    output.corner_radius = input.corner_radius;
    output.border_width = input.border_width;
    output.fill_color = input.fill_color;
    return output;
}

@vertex
fn solid_instance_vs_main(
    input: SolidInstanceInput,
    @builtin(vertex_index) vertex_index: u32,
) -> SolidFlatVertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    let corner = corners[vertex_index];
    var output: SolidFlatVertexOutput;
    output.position = vec4<f32>(mix(input.min_position, input.max_position, corner), 0.0, 1.0);
    output.color = input.color;
    return output;
}

fn rounded_box_coverages(
    local_position: vec2<f32>,
    half_extent: vec2<f32>,
    corner_radius: f32,
    border_width: f32,
) -> vec2<f32> {
    // Integrate a 4x4 local grid in each physical target pixel. Each sub-sample
    // keeps a quarter-pixel analytic filter so added detail does not widen edges.
    let pixel_step = vec2<f32>(
        max(fwidth(local_position.x), 0.0001),
        max(fwidth(local_position.y), 0.0001),
    );
    let inner_half_extent = half_extent - vec2<f32>(border_width);
    let has_inner_edge = border_width > 0.0 && all(inner_half_extent > vec2<f32>(0.0));
    let outer_distance = rounded_box_distance(local_position, half_extent, corner_radius);
    let distance_width = max(fwidth(outer_distance), 0.0001);
    let coverage_guard = distance_width * 0.75;
    let safe_inner_half_extent = max(inner_half_extent, vec2<f32>(0.0001));
    let inner_distance = rounded_box_distance(
        local_position,
        safe_inner_half_extent,
        max(corner_radius - border_width, 0.0),
    );
    let inner_distance_width = max(fwidth(inner_distance), 0.0001);
    let inner_coverage_guard = inner_distance_width * 0.75;

    if !has_inner_edge {
        if outer_distance <= coverage_guard * -1.0 {
            return vec2<f32>(1.0, 0.0);
        }
        if outer_distance >= coverage_guard {
            return vec2<f32>(0.0, 0.0);
        }
    } else {
        if outer_distance >= coverage_guard {
            return vec2<f32>(0.0, 0.0);
        }
        if inner_distance <= inner_coverage_guard * -1.0 {
            return vec2<f32>(1.0, 1.0);
        }
        if outer_distance <= coverage_guard * -1.0
            && inner_distance >= inner_coverage_guard
        {
            return vec2<f32>(1.0, 0.0);
        }
    }

    let subpixel_filter_scale = 0.25;
    let sample_offsets = array<vec2<f32>, 16>(
        vec2<f32>(-0.375, -0.375),
        vec2<f32>(-0.125, -0.375),
        vec2<f32>(0.125, -0.375),
        vec2<f32>(0.375, -0.375),
        vec2<f32>(-0.375, -0.125),
        vec2<f32>(-0.125, -0.125),
        vec2<f32>(0.125, -0.125),
        vec2<f32>(0.375, -0.125),
        vec2<f32>(-0.375, 0.125),
        vec2<f32>(-0.125, 0.125),
        vec2<f32>(0.125, 0.125),
        vec2<f32>(0.375, 0.125),
        vec2<f32>(-0.375, 0.375),
        vec2<f32>(-0.125, 0.375),
        vec2<f32>(0.125, 0.375),
        vec2<f32>(0.375, 0.375),
    );
    var outer_coverage_sum = 0.0;
    var inner_coverage_sum = 0.0;
    for (var sample_index = 0u; sample_index < 16u; sample_index++) {
        let sample_position = local_position + sample_offsets[sample_index] * pixel_step;
        let sample_outer_distance = rounded_box_distance(
            sample_position,
            half_extent,
            corner_radius,
        );
        let outer_coverage = rounded_box_alpha(
            sample_outer_distance,
            distance_width * subpixel_filter_scale,
        );
        var inner_coverage = 0.0;
        if has_inner_edge {
            let sample_inner_distance = rounded_box_distance(
                sample_position,
                inner_half_extent,
                max(corner_radius - border_width, 0.0),
            );
            inner_coverage = rounded_box_alpha(
                sample_inner_distance,
                inner_distance_width * subpixel_filter_scale,
            );
        }
        outer_coverage_sum += outer_coverage;
        inner_coverage_sum += inner_coverage;
    }
    return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625;
}

fn rounded_box_coverage(
    local_position: vec2<f32>,
    half_extent: vec2<f32>,
    corner_radius: f32,
    border_width: f32,
) -> f32 {
    let coverages = rounded_box_coverages(
        local_position,
        half_extent,
        corner_radius,
        border_width,
    );
    if border_width > 0.0 {
        return max(coverages.x - coverages.y, 0.0);
    }
    return coverages.x;
}

fn solid_fragment_color(input: SolidVertexOutput, linear_target: bool) -> vec4<f32> {
    if input.border_width > 0.0 && input.fill_color.a > 0.0 {
        let coverages = rounded_box_coverages(
            input.local_position,
            input.half_extent,
            input.corner_radius,
            input.border_width,
        );
        // Fill and border partition one outer coverage; they must never source-over each other.
        let inner_coverage = min(coverages.y, coverages.x);
        let fill = material_solid_color(
            vec4<f32>(input.fill_color.rgb, input.fill_color.a * inner_coverage),
            linear_target,
        );
        let border = material_solid_color(
            vec4<f32>(
                input.color.rgb,
                input.color.a * (coverages.x - inner_coverage),
            ),
            linear_target,
        );
        return fill + border;
    }
    let coverage = rounded_box_coverage(
        input.local_position,
        input.half_extent,
        input.corner_radius,
        input.border_width,
    );
    return material_solid_color(
        vec4<f32>(input.color.rgb, input.color.a * coverage),
        linear_target,
    );
}

@fragment
fn solid_fs_linear_target(input: SolidVertexOutput) -> @location(0) vec4<f32> {
    return solid_fragment_color(input, true);
}

@fragment
fn solid_fs_byte_target(input: SolidVertexOutput) -> @location(0) vec4<f32> {
    return solid_fragment_color(input, false);
}

@fragment
fn solid_instance_fs_linear_target(input: SolidFlatVertexOutput) -> @location(0) vec4<f32> {
    return material_solid_color(input.color, true);
}

@fragment
fn solid_instance_fs_byte_target(input: SolidFlatVertexOutput) -> @location(0) vec4<f32> {
    return material_solid_color(input.color, false);
}

@vertex
fn image_vs_main(input: ImageVertexInput) -> ImageVertexOutput {
    var output: ImageVertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn image_fs_linear_target(input: ImageVertexOutput) -> @location(0) vec4<f32> {
    return material_image_color(textureSample(source_texture, source_sampler, input.uv), true);
}

@fragment
fn image_fs_byte_target(input: ImageVertexOutput) -> @location(0) vec4<f32> {
    return material_image_color(textureSample(source_texture, source_sampler, input.uv), false);
}
