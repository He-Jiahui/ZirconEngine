@group(1) @binding(23) var<storage, read> zr_light_probe_grid: array<vec4<f32>>;
@group(1) @binding(24) var zr_lightmap_atlas: texture_2d_array<f32>;
@group(1) @binding(28) var zr_lightmap_sampler: sampler;

const ZR_LIGHT_PROBE_HEADER_WORDS: u32 = 3u;
const ZR_LIGHT_PROBE_SH_COEFFICIENTS: u32 = 9u;

fn zr_lightmap_probe_grid_dims() -> vec3<u32> {
    if (arrayLength(&zr_light_probe_grid) < ZR_LIGHT_PROBE_HEADER_WORDS) {
        return vec3<u32>(0u);
    }
    return vec3<u32>(
        bitcast<u32>(zr_light_probe_grid[1].z),
        bitcast<u32>(zr_light_probe_grid[1].w),
        bitcast<u32>(zr_light_probe_grid[2].x),
    );
}

fn zr_lightmap_probe_grid_valid() -> bool {
    if (arrayLength(&zr_light_probe_grid) < ZR_LIGHT_PROBE_HEADER_WORDS) {
        return false;
    }
    let dims = zr_lightmap_probe_grid_dims();
    if (any(dims == vec3<u32>(0u))) {
        return false;
    }
    let probe_count = bitcast<u32>(zr_light_probe_grid[2].w);
    return probe_count == dims.x * dims.y * dims.z
        && arrayLength(&zr_light_probe_grid)
            >= ZR_LIGHT_PROBE_HEADER_WORDS + probe_count * ZR_LIGHT_PROBE_SH_COEFFICIENTS;
}

fn zr_lightmap_probe_index(dims: vec3<u32>, coord: vec3<u32>) -> u32 {
    return coord.x + dims.x * (coord.y + dims.y * coord.z);
}

fn zr_lightmap_eval_probe(probe_index: u32, normal_ws: vec3<f32>) -> vec3<f32> {
    let base = ZR_LIGHT_PROBE_HEADER_WORDS + probe_index * ZR_LIGHT_PROBE_SH_COEFFICIENTS;
    let normal_length = length(normal_ws);
    if (normal_length <= 0.000001) {
        return vec3<f32>(0.0);
    }
    let n = normal_ws / normal_length;
    let x = n.x;
    let y = n.y;
    let z = n.z;
    var irradiance = zr_light_probe_grid[base].rgb * 0.2820948;
    irradiance += zr_light_probe_grid[base + 1u].rgb * (0.48860252 * z);
    irradiance += zr_light_probe_grid[base + 2u].rgb * (0.48860252 * y);
    irradiance += zr_light_probe_grid[base + 3u].rgb * (0.48860252 * x);
    irradiance += zr_light_probe_grid[base + 4u].rgb * (1.0925485 * x * z);
    irradiance += zr_light_probe_grid[base + 5u].rgb * (1.0925485 * z * y);
    irradiance += zr_light_probe_grid[base + 6u].rgb * (0.31539157 * (3.0 * y * y - 1.0));
    irradiance += zr_light_probe_grid[base + 7u].rgb * (1.0925485 * x * y);
    irradiance += zr_light_probe_grid[base + 8u].rgb * (0.54627424 * (x * x - z * z));
    return max(irradiance, vec3<f32>(0.0));
}

fn zr_lightmap_sample_probe_grid(world_position: vec3<f32>, normal_ws: vec3<f32>) -> vec3<f32> {
    if (!zr_lightmap_probe_grid_valid()) {
        return vec3<f32>(0.0);
    }
    let bounds_min = zr_light_probe_grid[0].xyz;
    let cell_size = vec3<f32>(
        zr_light_probe_grid[0].w,
        zr_light_probe_grid[1].x,
        zr_light_probe_grid[1].y,
    );
    if (any(cell_size <= vec3<f32>(0.0))) {
        return vec3<f32>(0.0);
    }
    let dims = zr_lightmap_probe_grid_dims();
    let grid_position = (world_position - bounds_min) / cell_size;
    let max_position = vec3<f32>(dims - vec3<u32>(1u));
    if (any(grid_position < vec3<f32>(0.0)) || any(grid_position > max_position)) {
        return vec3<f32>(0.0);
    }

    let base = vec3<u32>(floor(grid_position));
    let next = min(base + vec3<u32>(1u), dims - vec3<u32>(1u));
    let blend = grid_position - vec3<f32>(base);
    var irradiance = vec3<f32>(0.0);
    for (var z = 0u; z < 2u; z = z + 1u) {
        for (var y = 0u; y < 2u; y = y + 1u) {
            for (var x = 0u; x < 2u; x = x + 1u) {
                let coord = vec3<u32>(
                    select(base.x, next.x, x != 0u),
                    select(base.y, next.y, y != 0u),
                    select(base.z, next.z, z != 0u),
                );
                let weight = select(1.0 - blend.x, blend.x, x != 0u)
                    * select(1.0 - blend.y, blend.y, y != 0u)
                    * select(1.0 - blend.z, blend.z, z != 0u);
                irradiance += zr_lightmap_eval_probe(
                    zr_lightmap_probe_index(dims, coord),
                    normal_ws,
                ) * weight;
            }
        }
    }
    return irradiance;
}

fn zr_lightmap_sample_atlas(instance_index: u32, uv2: vec2<f32>) -> vec3<f32> {
    let uv_rect = zr_gpu_scene_lightmap_uv_rect(instance_index);
    let params = zr_gpu_scene_lightmap_params(instance_index);
    let atlas_uv = clamp(uv2 * uv_rect.xy + uv_rect.zw, vec2<f32>(0.0), vec2<f32>(1.0));
    return max(
        textureSample(zr_lightmap_atlas, zr_lightmap_sampler, atlas_uv, i32(params.x)).rgb,
        vec3<f32>(0.0),
    );
}

fn zr_lightmap_baked_irradiance(
    instance_index: u32,
    uv2: vec2<f32>,
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
) -> vec3<f32> {
    let lightmapped = zr_gpu_scene_has_lightmap(instance_index);
    let local_volume = zr_irradiance_volume_sample(
        world_position,
        normal_ws,
        lightmapped,
    );
    if (local_volume.valid != 0u) {
        return local_volume.irradiance;
    }
    if (lightmapped) {
        return zr_lightmap_sample_atlas(instance_index, uv2);
    }
    return zr_lightmap_sample_probe_grid(world_position, normal_ws);
}
