const SSS_SHADING_MODEL_ID: u32 = 16u;

struct SubsurfaceParams {
    viewport_width: u32,
    viewport_height: u32,
    profile_count: u32,
    active_profile_mask: u32,
    inverse_view_projection: mat4x4<f32>,
};

struct IndirectDispatchArgs {
    group_count_x: atomic<u32>,
    group_count_y: u32,
    group_count_z: u32,
    padding: u32,
};

@group(0) @binding(0) var gbuffer_material: texture_2d<f32>;
@group(0) @binding(1) var gbuffer_normal: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> tile_list: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read_write> indirect_args: IndirectDispatchArgs;
@group(0) @binding(4) var<uniform> params: SubsurfaceParams;

var<workgroup> tile_active: atomic<u32>;

fn decode_shading_model(material_sample: vec4<f32>) -> u32 {
    return u32(round(material_sample.a * 255.0)) & 0x7fu;
}

fn profile_is_active(profile_index: u32) -> bool {
    return profile_index < params.profile_count
        && (params.active_profile_mask & (1u << profile_index)) != 0u;
}

@compute @workgroup_size(8, 8, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    if (local_index == 0u && all(workgroup_id == vec3<u32>(0u))) {
        indirect_args.group_count_y = 1u;
        indirect_args.group_count_z = 1u;
    }
    if (local_index == 0u) {
        atomicStore(&tile_active, 0u);
    }
    workgroupBarrier();

    if (global_id.x < params.viewport_width && global_id.y < params.viewport_height) {
        let pixel = vec2<i32>(global_id.xy);
        let material_sample = textureLoad(gbuffer_material, pixel, 0);
        let normal_sample = textureLoad(gbuffer_normal, pixel, 0);
        let profile_index = u32(round(normal_sample.a * 255.0));
        if (decode_shading_model(material_sample) == SSS_SHADING_MODEL_ID
            && profile_is_active(profile_index)) {
            atomicStore(&tile_active, 1u);
        }
    }

    workgroupBarrier();
    if (local_index == 0u && atomicLoad(&tile_active) != 0u) {
        let output_index = atomicAdd(&indirect_args.group_count_x, 1u);
        tile_list[output_index] = workgroup_id.xy;
    }
}
