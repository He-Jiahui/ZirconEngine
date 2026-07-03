pub(in crate::asset::pipeline::manager) fn builtin_pbr_wgsl() -> &'static str {
    concat!(
        include_str!("../../../../graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl"),
        "\n",
        include_str!(
            "../../../../graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl"
        ),
        "\n",
        include_str!("../../../../graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl"),
        "\n",
        r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
};

struct MaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;
@group(2) @binding(1) var color_texture: texture_2d<f32>;
@group(2) @binding(2) var color_sampler: sampler;

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
    @location(3) tint: vec4<f32>,
    @location(4) shadow_params: vec4<f32>,
};

const EPSILON: f32 = 0.000001;

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return value / value_length;
}

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = zr_world_from_local(instance_index);
    let world_position = world_from_local * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((world_from_local * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.world_position = world_position.xyz;
    out.tint = zr_gpu_scene_tint(instance_index);
    out.shadow_params = zr_gpu_scene_shadow_params(instance_index);
    return out;
}

fn shade_gpu_light_index(light_index: u32, world_position: vec3<f32>, normal: vec3<f32>, shadow_params: vec4<f32>, view_z: f32) -> vec3<f32> {
    if (light_index >= zr_gpu_scene_light_count()) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let radiance = max(light.color_intensity.rgb, vec3<f32>(0.0, 0.0, 0.0)) * max(light.color_intensity.w, 0.0);
    if (length(radiance) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = normalize_or_zero(-light.direction_type.xyz);
        var direct_visibility = 1.0;
        if (shadow_params.z > 0.5) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        return radiance * max(dot(normal, light_vector), 0.0) * direct_visibility;
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let range = max(light.position_range.w, EPSILON);
    if (distance_to_light >= range) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let attenuation = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    var shadow_visibility = 1.0;
    if (shadow_params.z > 0.5) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return radiance * max(dot(normal, light_vector), 0.0) * attenuation * shadow_visibility;
}

fn gpu_light_lighting(frag_coord: vec2<f32>, world_position: vec3<f32>, normal: vec3<f32>, shadow_params: vec4<f32>) -> vec3<f32> {
    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let tile_base = zr_light_tile_base(frag_coord, zr_light_grid_params);
    var lighting = vec3<f32>(0.0, 0.0, 0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            lighting = lighting + shade_gpu_light_index(light_index, world_position, normal, shadow_params, view_z);
            mask = mask & (mask - 1u);
        }
    }
    return lighting;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * input.tint;
    let normal = normalize_or_zero(input.world_normal);
    let lighting = scene.ambient_color.rgb + gpu_light_lighting(input.position.xy, input.world_position, normal, input.shadow_params);

    return vec4<f32>(albedo.rgb * lighting, albedo.a);
}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::builtin_pbr_wgsl;

    #[test]
    fn builtin_pbr_shader_matches_current_scene_uniform_layout() {
        let shader = builtin_pbr_wgsl();

        let view_proj = shader.find("view_proj: mat4x4<f32>").unwrap();
        let view_proj_unjittered = shader.find("view_proj_unjittered: mat4x4<f32>").unwrap();
        let inverse_view_proj = shader.find("inverse_view_proj: mat4x4<f32>").unwrap();
        let ambient_color = shader.find("ambient_color: vec4<f32>").unwrap();

        assert!(view_proj < view_proj_unjittered);
        assert!(view_proj_unjittered < inverse_view_proj);
        assert!(inverse_view_proj < ambient_color);
        assert!(!shader.contains("light_dir: vec4<f32>"));
        assert!(!shader.contains("light_color: vec4<f32>"));
    }

    #[test]
    fn builtin_pbr_shader_receives_shadow_atlas_resources() {
        let shader = builtin_pbr_wgsl();

        for expected in [
            "@group(1) @binding(8) var zr_shadow_atlas: texture_depth_2d;",
            "@group(1) @binding(9) var zr_shadow_sampler: sampler_comparison;",
            "@group(1) @binding(10) var<storage, read> zr_shadow_slots",
            "@group(1) @binding(11) var<uniform> zr_shadow_globals",
            "fn zr_gpu_light_shadow_visibility",
            "fn zr_sample_shadow_slot",
            "fn zr_shadow_slot_pcf_quality",
            "ZR_SHADOW_PCF_QUALITY_MEDIUM",
            "ZR_SHADOW_PCF_MEDIUM_RADIUS_TEXELS",
            "ZR_SHADOW_PCF_HIGH_RADIUS_TEXELS",
            "zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z)",
        ] {
            assert!(
                shader.contains(expected),
                "builtin PBR shader should expose shadow atlas resource `{expected}`"
            );
        }
        assert!(shader.contains("fn shade_gpu_light_index"));
        assert!(shader.contains("let light = zr_gpu_light(light_index);"));
        assert!(shader.contains("zr_gpu_light_casts_shadow(light)"));
        assert!(!shader.contains("ShadowReceiverUniform"));
        assert!(!shader.contains("shadow_map_tex"));
        assert!(!shader.contains("shadow_compare_sampler"));
        assert!(!shader.contains("sample_shadow_visibility"));
        assert!(!shader.contains("world_to_shadow_coord"));
    }

    #[test]
    fn builtin_pbr_shader_receives_light_grid_resources() {
        let shader = builtin_pbr_wgsl();

        for expected in [
            "@group(1) @binding(20) var<uniform> zr_light_grid_params",
            "@group(1) @binding(21) var<storage, read> zr_light_zbins",
            "@group(1) @binding(22) var<storage, read> zr_light_tile_masks",
            "zr_light_mask_word(tile_base, bin, word, zr_light_grid_params)",
            "firstTrailingBit(mask)",
            "gpu_light_lighting(input.position.xy",
        ] {
            assert!(
                shader.contains(expected),
                "builtin PBR shader should use `{expected}` for light-grid lighting"
            );
        }
        assert!(!shader.contains("for (var i = 0u; i < light_count; i = i + 1u)"));
    }
}
