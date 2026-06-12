struct SceneUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj: mat4x4<f32>,
    motion_params: vec4<f32>,
    point_light_position_range: array<vec4<f32>, 8>,
    point_light_color_intensity: array<vec4<f32>, 8>,
    point_light_params: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
    shadow_params: vec4<f32>,
    previous_model: mat4x4<f32>,
    motion_params: vec4<f32>,
};

struct SkinnedJointPaletteUniform {
    joint_matrices: array<mat4x4<f32>, 256>,
    params: vec4<u32>,
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
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(1) @binding(1) var<uniform> skinned_joint_palette: SkinnedJointPaletteUniform;
@group(1) @binding(2) var<uniform> previous_skinned_joint_palette: SkinnedJointPaletteUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(2) var normal_tex: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;
@group(2) @binding(4) var metallic_roughness_tex: texture_2d<f32>;
@group(2) @binding(5) var metallic_roughness_sampler: sampler;
@group(2) @binding(6) var occlusion_tex: texture_2d<f32>;
@group(2) @binding(7) var occlusion_sampler: sampler;
@group(2) @binding(8) var emissive_tex: texture_2d<f32>;
@group(2) @binding(9) var emissive_sampler: sampler;
@group(3) @binding(0) var<uniform> material_properties: MaterialPropertyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joint_indices: vec4<u32>,
    @location(4) joint_weights: vec4<f32>,
    @location(5) tangent: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) uv1: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) uv1: vec2<f32>,
};

const EPSILON: f32 = 0.000001;
const POINT_LIGHT_UNIFORM_LIMIT: u32 = 8u;

fn normalize_or_up(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    if (value_length <= EPSILON) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return value / value_length;
}

fn skin_weight(joint_index: u32, weight: f32) -> f32 {
    if (weight <= EPSILON || joint_index >= skinned_joint_palette.params.x) {
        return 0.0;
    }
    return weight;
}

fn skin_weight_sum(joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> f32 {
    return skin_weight(joint_indices.x, joint_weights.x)
        + skin_weight(joint_indices.y, joint_weights.y)
        + skin_weight(joint_indices.z, joint_weights.z)
        + skin_weight(joint_indices.w, joint_weights.w);
}

fn skin_vertex_position(position: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> vec3<f32> {
    if (model_data.motion_params.y <= 0.5 || skinned_joint_palette.params.x == 0u) {
        return position;
    }

    let weight_sum = skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return position;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.x] * vec4<f32>(position, 1.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.y] * vec4<f32>(position, 1.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.z] * vec4<f32>(position, 1.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.w] * vec4<f32>(position, 1.0)).xyz * (weight_w / weight_sum);
    }
    return skinned;
}

fn skin_vertex_normal(normal: vec3<f32>, joint_indices: vec4<u32>, joint_weights: vec4<f32>) -> vec3<f32> {
    if (model_data.motion_params.y <= 0.5 || skinned_joint_palette.params.x == 0u) {
        return normal;
    }

    let weight_sum = skin_weight_sum(joint_indices, joint_weights);
    if (weight_sum <= EPSILON) {
        return normal;
    }

    var skinned = vec3<f32>(0.0, 0.0, 0.0);
    let weight_x = skin_weight(joint_indices.x, joint_weights.x);
    let weight_y = skin_weight(joint_indices.y, joint_weights.y);
    let weight_z = skin_weight(joint_indices.z, joint_weights.z);
    let weight_w = skin_weight(joint_indices.w, joint_weights.w);
    if (weight_x > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.x] * vec4<f32>(normal, 0.0)).xyz * (weight_x / weight_sum);
    }
    if (weight_y > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.y] * vec4<f32>(normal, 0.0)).xyz * (weight_y / weight_sum);
    }
    if (weight_z > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.z] * vec4<f32>(normal, 0.0)).xyz * (weight_z / weight_sum);
    }
    if (weight_w > 0.0) {
        skinned = skinned + (skinned_joint_palette.joint_matrices[joint_indices.w] * vec4<f32>(normal, 0.0)).xyz * (weight_w / weight_sum);
    }
    return normalize_or_up(skinned);
}

fn transform_material_uv(uv: vec2<f32>, transform: vec4<f32>) -> vec2<f32> {
    return uv * transform.xy + transform.zw;
}

fn material_uv_channel(channel: f32) -> u32 {
    return select(0u, 1u, channel >= 0.5);
}

fn select_material_uv(uv0: vec2<f32>, uv1: vec2<f32>, channel: f32) -> vec2<f32> {
    if (material_uv_channel(channel) == 1u) {
        return uv1;
    }
    return uv0;
}

fn transform_material_uv_channel(uv0: vec2<f32>, uv1: vec2<f32>, transform: vec4<f32>, channel: f32) -> vec2<f32> {
    return transform_material_uv(select_material_uv(uv0, uv1, channel), transform);
}

fn point_light(position: vec3<f32>, normal: vec3<f32>, light_position: vec3<f32>, color: vec3<f32>, radius: f32, strength: f32) -> vec3<f32> {
    let to_light = light_position - position;
    let distance_to_light = length(to_light);
    if (distance_to_light >= radius) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let falloff = pow(1.0 - distance_to_light / radius, 2.0);
    let diffuse = max(dot(normal, light_vector), 0.0);
    return color * diffuse * falloff * strength;
}

fn scene_point_lights(position: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let light_count = min(u32(max(scene.point_light_params.x, 0.0)), POINT_LIGHT_UNIFORM_LIMIT);
    var accumulated = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0u; i < POINT_LIGHT_UNIFORM_LIMIT; i = i + 1u) {
        if (i >= light_count) {
            break;
        }
        let position_range = scene.point_light_position_range[i];
        let color_intensity = scene.point_light_color_intensity[i];
        accumulated = accumulated + point_light(
            position,
            normal,
            position_range.xyz,
            color_intensity.rgb,
            max(position_range.w, EPSILON),
            max(color_intensity.w, 0.0) * 0.075,
        );
    }
    return accumulated;
}

fn graveyard_detail_color(position: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let grid = abs(fract(position.xz * 0.42) - vec2<f32>(0.5, 0.5));
    let mortar = 1.0 - smoothstep(0.455, 0.493, max(grid.x, grid.y));
    let damp = smoothstep(0.2, 0.86, fract(position.x * 0.37 + position.z * 0.19));
    let stone_noise = fract(sin(dot(floor(position.xz * 0.42), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let stone = mix(vec3<f32>(0.155, 0.165, 0.17), vec3<f32>(0.235, 0.225, 0.22), stone_noise);
    let moss = vec3<f32>(0.055, 0.105, 0.072) * damp;
    return stone + moss + vec3<f32>(0.045, 0.048, 0.052) * mortar;
}

fn is_arena_floor(base_color: vec4<f32>, normal: vec3<f32>) -> bool {
    let flat_floor = normal.y > 0.92;
    let authored_floor = base_color.r < 0.30 && base_color.g < 0.30 && base_color.b < 0.30 && base_color.a > 0.99;
    return flat_floor && authored_floor;
}

fn forest_noise_2d(value: vec2<f32>) -> f32 {
    return fract(sin(dot(value, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn forest_grass_detail_mask() -> bool {
    return material_properties.data0.a > 0.92 && material_properties.data0.a < 0.955;
}

fn forest_ground_detail_mask() -> bool {
    return material_properties.data0.a > 0.955 && material_properties.data0.a < 0.96;
}

fn is_jungle_ground_surface(base_color: vec4<f32>, normal: vec3<f32>) -> bool {
    return normal.y > 0.72
        && base_color.r > 0.20
        && base_color.g > 0.42
        && base_color.b < 0.62
        && base_color.a > 0.99;
}

fn is_jungle_foliage_surface(base_color: vec4<f32>) -> bool {
    return base_color.r < 0.18
        && base_color.g > 0.22
        && base_color.b < 0.24
        && base_color.a > 0.99;
}

fn forest_ground_detail_color(base_color: vec4<f32>, position: vec3<f32>, uv: vec2<f32>) -> vec4<f32> {
    let canopy_cell = floor(position.xz * 0.33);
    let canopy_noise = forest_noise_2d(canopy_cell);
    let root_band = 1.0 - smoothstep(0.10, 0.22, abs(fract(position.x * 0.16 + position.z * 0.09) - 0.5));
    let leaf_litter = smoothstep(0.48, 0.86, forest_noise_2d(floor(position.xz * 0.95)));
    let moss = smoothstep(0.15, 0.72, fract(uv.x * 1.7 + uv.y * 2.3));
    var detail = mix(base_color.rgb, vec3<f32>(0.24, 0.44, 0.16), 0.50);
    detail *= mix(0.86, 1.20, canopy_noise);
    detail = mix(detail, vec3<f32>(0.16, 0.25, 0.095), root_band * 0.18);
    detail = mix(detail, vec3<f32>(0.30, 0.29, 0.12), leaf_litter * 0.20);
    detail += vec3<f32>(0.035, 0.11, 0.030) * moss;
    detail = max(detail, vec3<f32>(0.18, 0.30, 0.12));
    return vec4<f32>(clamp(detail, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.25, 1.25, 1.25)), base_color.a);
}

fn forest_foliage_detail_color(base_color: vec4<f32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>) -> vec4<f32> {
    let vein = 1.0 - smoothstep(0.035, 0.13, abs(fract(uv.x * 4.0 + uv.y * 0.7) - 0.5));
    let leaf_noise = forest_noise_2d(floor(position.xz * 1.35 + uv * 8.0));
    let rim = pow(1.0 - abs(normal.y), 1.7);
    var detail = base_color.rgb * mix(0.74, 1.20, leaf_noise);
    detail = mix(detail, detail + vec3<f32>(0.055, 0.12, 0.025), vein * 0.30);
    detail += vec3<f32>(0.025, 0.09, 0.045) * rim;
    return vec4<f32>(clamp(detail, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.20, 1.25, 1.20)), base_color.a);
}

fn forest_grass_detail_color(base_color: vec4<f32>, position: vec3<f32>, uv: vec2<f32>) -> vec4<f32> {
    let blade_center = 1.0 - smoothstep(0.31, 0.50, abs(uv.x - 0.5));
    let blade_height = smoothstep(0.0, 1.0, uv.y);
    let strand_noise = forest_noise_2d(floor(position.xz * 3.8 + vec2<f32>(uv.x * 9.0, uv.y * 5.0)));
    let root_color = vec3<f32>(0.035, 0.12, 0.035);
    let tip_color = vec3<f32>(0.25, 0.54, 0.12);
    var detail = mix(root_color, tip_color, blade_height);
    detail *= mix(0.58, 1.06, blade_center);
    detail *= mix(0.82, 1.18, strand_noise);
    detail = mix(detail, base_color.rgb, 0.22);
    return vec4<f32>(clamp(detail, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.85, 1.10, 0.75)), 1.0);
}

fn vampire_actor_detail_mask() -> bool {
    return material_properties.data0.a > 0.96 && material_properties.data0.a < 0.995;
}

fn vampire_actor_detail_band(value: f32, width: f32) -> f32 {
    let band = abs(fract(value) - 0.5);
    return 1.0 - smoothstep(width, width + 0.035, band);
}

fn vampire_actor_detail_color(base_color: vec4<f32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>) -> vec4<f32> {
    let fabric_noise = 0.5 + 0.5 * sin(dot(uv, vec2<f32>(43.0, 71.0)) + position.y * 11.0);
    let vertical_band = vampire_actor_detail_band(uv.x * 5.0 + position.y * 0.65, 0.055);
    let horizontal_band = vampire_actor_detail_band(uv.y * 7.0 + position.x * 0.08, 0.045);
    let body_edge = pow(1.0 - abs(normal.y), 2.2);
    let cold_rim = pow(1.0 - max(dot(normal, normalize(vec3<f32>(0.0, 0.4, -1.0))), 0.0), 2.6);
    var detail = base_color.rgb;
    detail *= mix(0.76, 1.18, fabric_noise);
    detail = mix(detail, detail * vec3<f32>(0.52, 0.18, 0.18), vertical_band * 0.24);
    detail = mix(detail, detail * vec3<f32>(1.18, 1.12, 0.96), horizontal_band * 0.16);
    detail += vec3<f32>(0.05, 0.075, 0.13) * body_edge;
    detail += vec3<f32>(0.08, 0.14, 0.24) * cold_rim;
    return vec4<f32>(clamp(detail, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.45, 1.45, 1.45)), base_color.a);
}

fn vampire_detail_normal(normal: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let normal_sample = textureSample(normal_tex, normal_sampler, uv).xyz * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    let tangent = normalize_or_up(vec3<f32>(1.0, 0.0, 0.0) - normal * dot(normal, vec3<f32>(1.0, 0.0, 0.0)));
    let bitangent = normalize_or_up(cross(normal, tangent));
    return normalize_or_up(normal + tangent * normal_sample.x * 0.11 + bitangent * normal_sample.y * 0.11);
}

fn vampire_camera_world_position() -> vec3<f32> {
    let world_center = scene.inverse_view_proj * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return world_center.xyz / max(world_center.w, EPSILON);
}

fn vampire_shadowed_direct_visibility(position: vec3<f32>, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    let canopy_cell = floor(position.xz * 0.24);
    let canopy_noise = forest_noise_2d(canopy_cell);
    let canopy_shadow = smoothstep(0.34, 0.86, canopy_noise);
    let slope_visibility = smoothstep(-0.12, 0.55, dot(normal, light_dir));
    let low_contact = smoothstep(0.02, 0.42, position.y + 0.08);
    let authored_shadow_bias = clamp(model_data.shadow_params.x, 0.0, 1.0);
    let canopy_visibility = mix(1.0, 0.54, canopy_shadow * 0.58);
    let contact_visibility = mix(0.68, 1.0, low_contact);
    return clamp(mix(0.46, 1.0, slope_visibility) * canopy_visibility * contact_visibility * mix(0.88, 1.0, authored_shadow_bias), 0.28, 1.0);
}

fn vampire_micro_occlusion(position: vec3<f32>, normal: vec3<f32>, roughness: f32, texture_occlusion: f32) -> f32 {
    let root_cell = floor(position.xz * 0.72);
    let root_noise = forest_noise_2d(root_cell);
    let ground_mask = smoothstep(0.62, 0.94, normal.y);
    let root_shadow = smoothstep(0.48, 0.88, root_noise) * ground_mask;
    let crevice = mix(1.0, 0.74, root_shadow * clamp(roughness, 0.0, 1.0));
    return clamp(texture_occlusion * crevice, 0.34, 1.0);
}

fn vampire_material_specular(normal: vec3<f32>, view_dir: vec3<f32>, light_dir: vec3<f32>, roughness: f32, metallic: f32, shadow_visibility: f32) -> vec3<f32> {
    let half_dir = normalize_or_up(view_dir + light_dir);
    let gloss = mix(72.0, 9.0, clamp(roughness, 0.0, 1.0));
    let highlight = pow(max(dot(normal, half_dir), 0.0), gloss);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 5.0);
    let f0 = mix(vec3<f32>(0.035, 0.04, 0.05), scene.light_color.rgb, clamp(metallic, 0.0, 1.0));
    return f0 * highlight * mix(0.35, 1.15, fresnel) * shadow_visibility * mix(1.20, 0.42, roughness);
}

fn vampire_wet_surface_reflection(position: vec3<f32>, normal: vec3<f32>, view_dir: vec3<f32>, roughness: f32, metallic: f32, base_color: vec3<f32>) -> vec3<f32> {
    let flatness = smoothstep(0.68, 0.97, normal.y);
    let puddle_noise = forest_noise_2d(floor(position.xz * 0.58));
    let wet_mask = flatness * smoothstep(0.34, 0.86, puddle_noise) * (1.0 - clamp(roughness, 0.0, 1.0) * 0.72);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 4.0);
    let moon_reflection = scene.light_color.rgb * vec3<f32>(0.55, 0.70, 1.0);
    let dark_water = mix(base_color * 0.18, moon_reflection, 0.58 + fresnel * 0.30);
    return dark_water * wet_mask * mix(0.78, 1.28, metallic);
}

fn vampire_ground_light_floor(base_color: vec4<f32>, normal: vec3<f32>) -> vec3<f32> {
    if (forest_ground_detail_mask() || is_jungle_ground_surface(base_color, normal)) {
        return vec3<f32>(0.34, 0.42, 0.24);
    }
    if (is_arena_floor(base_color, normal)) {
        return vec3<f32>(0.30, 0.32, 0.34);
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let local_position = skin_vertex_position(input.position, input.joint_indices, input.joint_weights);
    let local_normal = skin_vertex_normal(input.normal, input.joint_indices, input.joint_weights);
    let world_position = model_data.model * vec4<f32>(local_position, 1.0);
    out.clip_position = scene.view_proj * world_position;
    out.world_normal = normalize_or_up((model_data.model * vec4<f32>(local_normal, 0.0)).xyz);
    out.vertex_color = input.color;
    out.world_position = world_position.xyz;
    out.uv = input.uv;
    out.uv1 = input.uv1;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo_sample = textureSample(albedo_tex, albedo_sampler, input.uv);
    let metallic_roughness_sample = textureSample(metallic_roughness_tex, metallic_roughness_sampler, input.uv).rgb;
    let occlusion_sample = textureSample(occlusion_tex, occlusion_sampler, input.uv).r;
    var base_color = material_properties.data0 * albedo_sample * model_data.tint * input.vertex_color;
    var normal = vampire_detail_normal(normalize_or_up(input.world_normal), input.uv);
    if (forest_ground_detail_mask()) {
        base_color = forest_ground_detail_color(base_color, input.world_position, input.uv);
    } else if (vampire_actor_detail_mask()) {
        base_color = vampire_actor_detail_color(base_color, input.world_position, normal, input.uv);
    } else if (forest_grass_detail_mask()) {
        base_color = forest_grass_detail_color(base_color, input.world_position, input.uv);
    } else if (is_jungle_ground_surface(base_color, normal)) {
        base_color = forest_ground_detail_color(base_color, input.world_position, input.uv);
    } else if (is_jungle_foliage_surface(base_color)) {
        base_color = forest_foliage_detail_color(base_color, input.world_position, normal, input.uv);
    } else if (is_arena_floor(base_color, normal)) {
        base_color = vec4<f32>(graveyard_detail_color(input.world_position, input.uv), base_color.a);
    }
    let metallic = clamp(material_properties.data1.w * metallic_roughness_sample.b, 0.0, 1.0);
    let roughness = clamp(material_properties.data2.x * metallic_roughness_sample.g, 0.045, 1.0);
    let emissive = material_properties.data1.rgb * textureSample(emissive_tex, emissive_sampler, input.uv).rgb;
    let view_dir = normalize_or_up(vampire_camera_world_position() - input.world_position);
    let light_dir = normalize(-scene.light_dir.xyz);
    let lambert = max(dot(normal, light_dir), 0.0);
    let half_lambert = lambert * 0.72 + 0.28;
    let shadow_visibility = vampire_shadowed_direct_visibility(input.world_position, normal, light_dir);
    let micro_occlusion = vampire_micro_occlusion(input.world_position, normal, roughness, occlusion_sample);
    let surface_response = mix(1.0, 0.72, clamp(metallic, 0.0, 1.0)) * mix(1.08, 0.9, clamp(roughness, 0.0, 1.0));
    let moon_ambient = scene.ambient_color.rgb * vec3<f32>(0.26, 0.34, 0.72);
    let moon_direct = scene.light_color.rgb * half_lambert * 0.62 * shadow_visibility;
    let rim = pow(1.0 - max(dot(normal, normalize(vec3<f32>(0.0, 0.35, -1.0))), 0.0), 2.25);
    let rim_light = vec3<f32>(0.16, 0.25, 0.52) * rim;
    let material_specular = vampire_material_specular(normal, view_dir, light_dir, roughness, metallic, shadow_visibility);
    let wet_reflection = vampire_wet_surface_reflection(input.world_position, normal, view_dir, roughness, metallic, base_color.rgb);
    let local_lights = scene_point_lights(input.world_position, normal) * 0.48;
    let distance_fog = clamp((length(input.world_position.xz) - 7.0) / 24.0, 0.0, 1.0);
    let fog_color = vec3<f32>(0.060, 0.105, 0.070);
    let ground_light_floor = vampire_ground_light_floor(base_color, normal);
    let light_sum = max(moon_ambient * micro_occlusion + moon_direct + rim_light + local_lights, ground_light_floor);
    let lit = base_color.rgb * surface_response * micro_occlusion * min(light_sum, vec3<f32>(1.65, 1.65, 1.85)) + material_specular + wet_reflection;
    let graded = mix(lit + emissive, fog_color, distance_fog * 0.32);
    return vec4<f32>(graded, base_color.a);
}
