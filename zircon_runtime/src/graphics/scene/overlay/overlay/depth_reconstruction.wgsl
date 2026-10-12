// 统一以物理像素描述两个视口区域，片元坐标再映射回源深度纹理。
struct DepthRegion {
    source_origin: vec2<u32>,
    source_size: vec2<u32>,
    output_origin: vec2<u32>,
    output_size: vec2<u32>,
};

@group(0) @binding(0) var source_depth: texture_depth_2d;
@group(0) @binding(1) var<uniform> region: DepthRegion;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @builtin(frag_depth) f32 {
    let output_pixel = vec2<u32>(position.xy) - region.output_origin;
    // 按两个区域的尺寸比例选取源 texel，并钳住边缘，避免输出末像素映射到纹理范围之外。
    let source_pixel = min(
        output_pixel * region.source_size / region.output_size,
        region.source_size - vec2<u32>(1u),
    );
    return textureLoad(source_depth, vec2<i32>(region.source_origin + source_pixel), 0);
}
