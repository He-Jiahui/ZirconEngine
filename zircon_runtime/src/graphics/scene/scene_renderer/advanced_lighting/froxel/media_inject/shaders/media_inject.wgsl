struct MediaInjectParams {
    grid_and_volume_count: vec4<u32>,
    density_height_scattering: vec4<f32>,
    albedo_phase: vec4<f32>,
    view: ZrFroxelViewParams,
};

struct FogVolume {
    bounds_min_density: vec4<f32>,
    bounds_max: vec4<f32>,
    albedo: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: MediaInjectParams;
@group(0) @binding(1) var<storage, read> fog_volumes: array<FogVolume>;
@group(0) @binding(2) var media_texture: texture_storage_3d<rgba16float, write>;

fn inside_bounds(position: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> bool {
    return all(position >= bounds_min) && all(position <= bounds_max);
}

@compute @workgroup_size(4, 4, 4)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let dimensions = params.grid_and_volume_count.xyz;
    if (any(invocation_id >= dimensions)) {
        return;
    }

    let world_position = zr_froxel_world_position(invocation_id, dimensions, params.view);
    let height = max(world_position.y, 0.0);
    let global_density = params.density_height_scattering.x *
        exp(-height * params.density_height_scattering.y);
    let scattering_intensity = params.density_height_scattering.z;
    var extinction = global_density;
    var scattering = params.albedo_phase.xyz * global_density * scattering_intensity;

    for (var volume_index = 0u;
        volume_index < params.grid_and_volume_count.w;
        volume_index += 1u) {
        let volume = fog_volumes[volume_index];
        if (inside_bounds(world_position, volume.bounds_min_density.xyz, volume.bounds_max.xyz)) {
            let local_density = max(volume.bounds_min_density.w, 0.0);
            extinction += local_density;
            scattering += max(volume.albedo.xyz, vec3<f32>(0.0)) *
                local_density * scattering_intensity;
        }
    }

    textureStore(media_texture, invocation_id, vec4<f32>(scattering, extinction));
}
