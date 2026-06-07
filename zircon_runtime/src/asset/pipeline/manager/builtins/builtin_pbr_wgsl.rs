pub(in crate::asset::pipeline::manager) fn builtin_pbr_wgsl() -> &'static str {
    r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
};

struct ShadowReceiverUniform {
    light_view_proj: mat4x4<f32>,
    params: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
    shadow_params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;
@group(4) @binding(0) var shadow_map_tex: texture_depth_2d;
@group(4) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;
@group(4) @binding(2) var shadow_compare_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
};

const EPSILON: f32 = 0.000001;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((model.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.world_position = world_position.xyz;
    return out;
}

fn world_to_shadow_coord(world_position: vec3<f32>) -> vec4<f32> {
    let light_clip = shadow_receiver.light_view_proj * vec4<f32>(world_position, 1.0);
    if (abs(light_clip.w) <= EPSILON) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let light_ndc = light_clip.xyz / light_clip.w;
    if (any(light_ndc.xy < vec2<f32>(-1.0, -1.0)) || light_ndc.z < 0.0 || any(light_ndc > vec3<f32>(1.0, 1.0, 1.0))) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let shadow_uv = light_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return vec4<f32>(shadow_uv, light_ndc.z, 1.0);
}

fn sample_shadow_visibility(shadow_uv: vec2<f32>, receiver_depth: f32, offset: vec2<i32>) -> f32 {
    let shadow_size = max(textureDimensions(shadow_map_tex), vec2<u32>(1u, 1u));
    let shadow_texel = vec2<f32>(1.0, 1.0) / vec2<f32>(shadow_size);
    let sample_uv = clamp(shadow_uv + vec2<f32>(offset) * shadow_texel, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    return textureSampleCompare(shadow_map_tex, shadow_compare_sampler, sample_uv, receiver_depth);
}

fn shadow_visibility(world_position: vec3<f32>) -> f32 {
    if (shadow_receiver.params.x <= 0.5) {
        return 1.0;
    }
    if (model.shadow_params.z <= 0.5) {
        return 1.0;
    }

    let shadow_coord = world_to_shadow_coord(world_position);
    if (shadow_coord.w <= 0.0) {
        return 1.0;
    }

    let receiver_depth = clamp(shadow_coord.z - shadow_receiver.params.y, 0.0, 1.0);
    let offsets = array<vec2<i32>, 9>(
        vec2<i32>(-1, -1),
        vec2<i32>(0, -1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, 0),
        vec2<i32>(0, 0),
        vec2<i32>(1, 0),
        vec2<i32>(-1, 1),
        vec2<i32>(0, 1),
        vec2<i32>(1, 1),
    );
    var lit = 0.0;
    for (var i = 0u; i < 9u; i = i + 1u) {
        lit = lit + sample_shadow_visibility(shadow_coord.xy, receiver_depth, offsets[i]);
    }

    return mix(clamp(shadow_receiver.params.z, 0.0, 1.0), 1.0, lit / 9.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * model.tint;
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let direct_visibility = shadow_visibility(input.world_position);
    let lighting = scene.ambient_color.rgb + scene.light_color.rgb * ndotl * direct_visibility;
    return vec4<f32>(albedo.rgb * lighting, albedo.a);
}
"#
}

#[cfg(test)]
mod tests {
    use super::builtin_pbr_wgsl;

    #[test]
    fn builtin_pbr_shader_matches_current_scene_uniform_layout() {
        let shader = builtin_pbr_wgsl();

        let view_proj = shader.find("view_proj: mat4x4<f32>").unwrap();
        let inverse_view_proj = shader.find("inverse_view_proj: mat4x4<f32>").unwrap();
        let light_dir = shader.find("light_dir: vec4<f32>").unwrap();
        let light_color = shader.find("light_color: vec4<f32>").unwrap();
        let ambient_color = shader.find("ambient_color: vec4<f32>").unwrap();

        assert!(view_proj < inverse_view_proj);
        assert!(inverse_view_proj < light_dir);
        assert!(light_dir < light_color);
        assert!(light_color < ambient_color);
    }

    #[test]
    fn builtin_pbr_shader_receives_forward_shadow_map_resources() {
        let shader = builtin_pbr_wgsl();

        assert!(shader.contains("@group(4) @binding(0) var shadow_map_tex: texture_depth_2d;"));
        assert!(shader.contains(
            "@group(4) @binding(1) var<uniform> shadow_receiver: ShadowReceiverUniform;"
        ));
        assert!(shader
            .contains("@group(4) @binding(2) var shadow_compare_sampler: sampler_comparison;"));
        assert!(shader.contains("textureSampleCompare"));
        assert!(shader.contains("if (model.shadow_params.z <= 0.5)"));
        assert!(shader.contains("let direct_visibility = shadow_visibility(input.world_position);"));
        assert!(shader.contains(
            "let lighting = scene.ambient_color.rgb + scene.light_color.rgb * ndotl * direct_visibility;"
        ));
    }
}
