@group(1) @binding(33) var zr_light_cookie_atlas: texture_2d<f32>;
@group(1) @binding(34) var zr_light_cookie_sampler: sampler;

const ZR_COOKIE_PROJECTION_DIRECTIONAL: u32 = 1u;
const ZR_COOKIE_PROJECTION_SPOT: u32 = 2u;
const ZR_COOKIE_PROJECTION_POINT_OCTAHEDRAL: u32 = 3u;
const ZR_COOKIE_WRAP_REPEAT: u32 = 1u;
const ZR_COOKIE_ATLAS_TEXEL: f32 = 0.0009765625;
const ZR_COOKIE_EPSILON: f32 = 0.000001;

fn zr_cookie_light_basis(direction: vec3<f32>) -> mat3x3<f32> {
    let forward_length = length(direction);
    let forward = select(vec3<f32>(0.0, 0.0, -1.0), direction / forward_length, forward_length > ZR_COOKIE_EPSILON);
    let reference_up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(dot(forward, vec3<f32>(0.0, 1.0, 0.0))) >= 0.999);
    let right = normalize(cross(forward, reference_up));
    let up = normalize(cross(right, forward));
    return mat3x3<f32>(right, up, forward);
}

fn zr_cookie_octahedral_uv(direction: vec3<f32>) -> vec2<f32> {
    let denominator = dot(abs(direction), vec3<f32>(1.0));
    if (denominator <= ZR_COOKIE_EPSILON) {
        return vec2<f32>(0.5);
    }
    let normalized = direction / denominator;
    var folded = normalized.xy;
    if (normalized.z < 0.0) {
        folded = vec2<f32>(
            (1.0 - abs(normalized.y)) * sign(normalized.x),
            (1.0 - abs(normalized.x)) * sign(normalized.y),
        );
    }
    return folded * 0.5 + vec2<f32>(0.5);
}

fn zr_light_cookie_factor(light: ZrGpuLightData, world_position: vec3<f32>) -> vec3<f32> {
    if (light.cookie_misc.x == 0u) {
        return vec3<f32>(1.0);
    }
    let projection = light.cookie_misc.x;
    var uv = vec2<f32>(0.5);
    if (projection == ZR_COOKIE_PROJECTION_DIRECTIONAL) {
        let basis = zr_cookie_light_basis(light.direction_type.xyz);
        uv = vec2<f32>(dot(world_position, basis[0]), dot(world_position, basis[1]))
            * light.spot_angles_size.zw + light.position_range.xy;
    } else if (projection == ZR_COOKIE_PROJECTION_SPOT) {
        let basis = zr_cookie_light_basis(light.direction_type.xyz);
        let local = world_position - light.position_range.xyz;
        let depth = dot(local, basis[2]);
        let outer_angle = acos(clamp(light.spot_angles_size.y, -1.0, 1.0));
        let half_extent = depth * tan(outer_angle);
        if (depth <= ZR_COOKIE_EPSILON || abs(half_extent) <= ZR_COOKIE_EPSILON) {
            return vec3<f32>(0.0);
        }
        uv = vec2<f32>(dot(local, basis[0]), dot(local, basis[1])) / (2.0 * half_extent)
            + vec2<f32>(0.5);
    } else if (projection == ZR_COOKIE_PROJECTION_POINT_OCTAHEDRAL) {
        uv = zr_cookie_octahedral_uv(world_position - light.position_range.xyz);
    }
    if (light.cookie_misc.y == ZR_COOKIE_WRAP_REPEAT) {
        uv = fract(uv);
    } else if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec3<f32>(0.0);
    }
    let inset = vec2<f32>(ZR_COOKIE_ATLAS_TEXEL * 0.5);
    let atlas_min = light.cookie_uv_rect.xy + inset;
    let atlas_max = light.cookie_uv_rect.xy + light.cookie_uv_rect.zw - inset;
    let atlas_uv = mix(atlas_min, atlas_max, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    return textureSampleLevel(zr_light_cookie_atlas, zr_light_cookie_sampler, atlas_uv, 0.0).rgb;
}
