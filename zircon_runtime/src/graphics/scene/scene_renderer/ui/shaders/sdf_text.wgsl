@group(0) @binding(0) var sdf_atlas: texture_2d<f32>;
@group(0) @binding(1) var sdf_sampler: sampler;

struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let distance = textureSample(sdf_atlas, sdf_sampler, input.uv).r;
    let coverage = smoothstep(0.42, 0.58, distance);
    return vec4<f32>(input.color.rgb, input.color.a * coverage);
}
