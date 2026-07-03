@group(0) @binding(0) var sdf_atlas: texture_2d_array<f32>;
@group(0) @binding(1) var sdf_sampler: sampler;

struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) screen_px_range: f32,
    @location(4) page_index: u32,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) screen_px_range: f32,
    @location(3) @interpolate(flat) page_index: u32,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    out.screen_px_range = input.screen_px_range;
    out.page_index = input.page_index;
    return out;
}

fn sdf_coverage(distance: f32, screen_px_range: f32) -> f32 {
    let px_range = max(screen_px_range, 1.0);
    let signed_distance = (distance - 0.5) * px_range;
    let aa_width = max(fwidth(distance) * px_range, 1.0);
    return clamp(signed_distance / aa_width + 0.5, 0.0, 1.0);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let distance = textureSample(sdf_atlas, sdf_sampler, input.uv, i32(input.page_index)).r;
    let coverage = sdf_coverage(distance, input.screen_px_range);
    return vec4<f32>(input.color.rgb, input.color.a * coverage);
}
