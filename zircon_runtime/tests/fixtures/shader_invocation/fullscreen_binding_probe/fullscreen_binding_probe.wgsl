struct FullscreenBindingProbeParams {
    tint: vec4<f32>,
    exposure: f32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@group(0) @binding(2)
var<uniform> params: FullscreenBindingProbeParams;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(source_texture));
    let uv = position.xy / max(dimensions, vec2<f32>(1.0));
    return textureSample(source_texture, source_sampler, uv) * params.tint * params.exposure;
}
