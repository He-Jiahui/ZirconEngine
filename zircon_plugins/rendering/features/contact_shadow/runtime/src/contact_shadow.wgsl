@group(0) @binding(0) var depth_tex: texture_depth_2d;
@group(0) @binding(1) var normal_tex: texture_2d<f32>;
@group(0) @binding(2) var hzb_furthest_tex: texture_2d<f32>;
@group(0) @binding(3) var contact_shadow_out: texture_storage_2d<rgba8unorm, write>;

fn load_depth(coord: vec2<i32>, size: vec2<i32>) -> f32 {
    let clamped = clamp(coord, vec2<i32>(0, 0), size - vec2<i32>(1, 1));
    return textureLoad(depth_tex, clamped, 0);
}

fn load_hzb_furthest(coord: vec2<i32>, viewport_size: vec2<u32>, mip_level: u32) -> f32 {
    let mip_count = textureNumLevels(hzb_furthest_tex);
    let safe_mip = min(mip_level, max(mip_count, 1u) - 1u);
    let hzb_size = max(textureDimensions(hzb_furthest_tex, safe_mip), vec2<u32>(1u, 1u));
    let safe_coord = max(coord, vec2<i32>(0, 0));
    let uv = (vec2<f32>(safe_coord) + vec2<f32>(0.5, 0.5))
        / max(vec2<f32>(viewport_size), vec2<f32>(1.0, 1.0));
    let hzb_coord = min(
        vec2<u32>(uv * vec2<f32>(hzb_size)),
        hzb_size - vec2<u32>(1u, 1u)
    );
    return textureLoad(hzb_furthest_tex, vec2<i32>(hzb_coord), safe_mip).r;
}

fn contact_shadow_sample_weight(offset: vec2<i32>) -> f32 {
    let distance = max(abs(f32(offset.x)) + abs(f32(offset.y)), 1.0);
    return 1.0 / distance;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let viewport_size = textureDimensions(contact_shadow_out);
    if (invocation_id.x >= viewport_size.x || invocation_id.y >= viewport_size.y) {
        return;
    }

    let coord = vec2<i32>(invocation_id.xy);
    let size_i32 = vec2<i32>(viewport_size);
    let center_depth = load_depth(coord, size_i32);
    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));
    let normal_grazing = clamp(1.0 - max(normal.z, 0.0), 0.0, 1.0);

    let ray_offsets = array<vec2<i32>, 12>(
        vec2<i32>(1, 0),
        vec2<i32>(2, 0),
        vec2<i32>(0, 1),
        vec2<i32>(0, 2),
        vec2<i32>(-1, 0),
        vec2<i32>(-2, 0),
        vec2<i32>(0, -1),
        vec2<i32>(0, -2),
        vec2<i32>(1, 1),
        vec2<i32>(-1, 1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, -1),
    );

    var ray_occlusion = 0.0;
    var weight_sum = 0.0;
    for (var i = 0u; i < 12u; i = i + 1u) {
        let offset = ray_offsets[i];
        let sample_depth = load_depth(coord + offset, size_i32);
        let blocker_delta = max(center_depth - sample_depth, 0.0);
        let weight = contact_shadow_sample_weight(offset);
        ray_occlusion = ray_occlusion + smoothstep(0.0008, 0.018, blocker_delta) * weight;
        weight_sum = weight_sum + weight;
    }

    let hzb_furthest = load_hzb_furthest(coord, viewport_size, 1u);
    let hzb_delta = max(center_depth - hzb_furthest, 0.0);
    let hzb_occlusion = smoothstep(0.0015, 0.045, hzb_delta);
    let contact_shadow = clamp(
        (ray_occlusion / max(weight_sum, 0.001)) * 0.72
            + hzb_occlusion * 0.22
            + normal_grazing * 0.06,
        0.0,
        0.88
    );
    let visibility = 1.0 - contact_shadow;

    textureStore(contact_shadow_out, coord, vec4<f32>(visibility, visibility, visibility, 1.0));
}
