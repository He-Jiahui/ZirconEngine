// 延迟光照为次表面材质预留漫反射与高光；此通道将散射漫反射与高光重组，
// 仅覆盖对应材质像素，其余场景色沿原图附件保留。
const SSS_SHADING_MODEL_ID: u32 = 16u;

@group(0) @binding(0) var scattered: texture_2d<f32>;
@group(0) @binding(1) var specular: texture_2d<f32>;
@group(0) @binding(2) var gbuffer_material: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(input.position.xy);
    let material_sample = textureLoad(gbuffer_material, pixel, 0);
    let shading_model = u32(round(material_sample.a * 255.0)) & 0x7fu;
    if (shading_model != SSS_SHADING_MODEL_ID) {
        discard;
    }
    // TODO: [CR-SCENE-ENV-0002] 确认未配置或无效 profile 的 SSS 材质应如何回退；
    // setup/scatter 会跳过该 profile，此处只判断材质类型。下一步补缺失 profile 的完整帧颜色用例。
    let scattered_sample = textureLoad(scattered, pixel, 0);
    let specular_sample = textureLoad(specular, pixel, 0);
    return vec4<f32>(scattered_sample.rgb + specular_sample.rgb, 1.0);
}
