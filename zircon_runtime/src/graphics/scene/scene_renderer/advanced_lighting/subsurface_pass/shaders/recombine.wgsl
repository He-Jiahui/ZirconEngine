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
    let scattered_sample = textureLoad(scattered, pixel, 0);
    let specular_sample = textureLoad(specular, pixel, 0);
    return vec4<f32>(scattered_sample.rgb + specular_sample.rgb, 1.0);
}
