struct SolidVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
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

fn rounded_box_alpha(local_position: vec2<f32>, half_extent: vec2<f32>, radius: f32, softness: f32) -> f32 {
    let q = abs(local_position) - half_extent + vec2<f32>(radius);
    let outside_distance = length(max(q, vec2<f32>(0.0))) - radius;
    return 1.0 - smoothstep(-softness, softness, outside_distance);
}

fn material_solid_color(color: vec4<f32>) -> vec4<f32> {
    return premultiply_alpha(material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0)));
}

fn material_image_color(color: vec4<f32>) -> vec4<f32> {
    return premultiply_alpha(material_tint(color, vec4<f32>(1.0, 1.0, 1.0, 1.0)));
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var output: SolidVertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn solid_fs_main(input: SolidVertexOutput) -> @location(0) vec4<f32> {
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
