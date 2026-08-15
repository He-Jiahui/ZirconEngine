struct SolidVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) local_position: vec2<f32>,
    @location(3) half_extent: vec2<f32>,
    @location(4) corner_radius: f32,
    @location(5) border_width: f32,
};

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_position: vec2<f32>,
    @location(2) @interpolate(flat) half_extent: vec2<f32>,
    @location(3) @interpolate(flat) corner_radius: f32,
    @location(4) @interpolate(flat) border_width: f32,
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

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

fn material_tint(color: vec4<f32>, tint: vec4<f32>) -> vec4<f32> {
    return color * tint;
}

fn premultiply_alpha(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn rounded_box_distance(local_position: vec2<f32>, half_extent: vec2<f32>, radius: f32) -> f32 {
    let q = abs(local_position) - half_extent + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn rounded_box_alpha(signed_distance: f32, distance_width: f32) -> f32 {
    return smoothstep(distance_width * 0.5, distance_width * -0.5, signed_distance);
}

fn material_solid_color(color: vec4<f32>) -> vec4<f32> {
    return premultiply_alpha(material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0)));
}

fn material_image_color(color: vec4<f32>) -> vec4<f32> {
    return material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0));
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

@fragment
fn solid_fs_main(input: SolidVertexOutput) -> @location(0) vec4<f32> {
    let outer_distance = rounded_box_distance(
        input.local_position,
        input.half_extent,
        input.corner_radius,
    );
    let distance_width = max(fwidth(outer_distance), 0.0001);
    let outer_coverage = rounded_box_alpha(outer_distance, distance_width);
    var coverage = outer_coverage;
    let inner_half_extent = input.half_extent - vec2<f32>(input.border_width);
    if input.border_width > 0.0 && all(inner_half_extent > vec2<f32>(0.0)) {
        let inner_distance = rounded_box_distance(
            input.local_position,
            inner_half_extent,
            max(input.corner_radius - input.border_width, 0.0),
        );
        let inner_coverage = rounded_box_alpha(inner_distance, distance_width);
        coverage = max(outer_coverage - inner_coverage, 0.0);
    }
    return material_solid_color(vec4<f32>(input.color.rgb, input.color.a * coverage));
}

@fragment
fn solid_instance_fs_main(input: SolidFlatVertexOutput) -> @location(0) vec4<f32> {
    return material_solid_color(input.color);
}

@vertex
fn image_vs_main(input: ImageVertexInput) -> ImageVertexOutput {
    var output: ImageVertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn image_fs_main(input: ImageVertexOutput) -> @location(0) vec4<f32> {
    return material_image_color(textureSample(source_texture, source_sampler, input.uv));
}
