const PI: f32 = 3.14159265358979323846;
const INV_UINT_MAX_PLUS_ONE: f32 = 2.3283064365386963e-10;

struct IblIrradianceCubeParams {
    source_face_size: u32,
    irradiance_face_size: u32,
    sample_count: u32,
    _pad0: u32,
};

@group(0) @binding(0) var<uniform> params: IblIrradianceCubeParams;
@group(0) @binding(1) var source_cubemap: texture_cube<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var irradiance_output: texture_storage_2d_array<rgba16float, write>;

fn cube_face_direction(face: u32, uv: vec2<f32>) -> vec3<f32> {
    switch face {
        case 0u: {
            return normalize(vec3<f32>(1.0, -uv.y, -uv.x));
        }
        case 1u: {
            return normalize(vec3<f32>(-1.0, -uv.y, uv.x));
        }
        case 2u: {
            return normalize(vec3<f32>(uv.x, 1.0, uv.y));
        }
        case 3u: {
            return normalize(vec3<f32>(uv.x, -1.0, -uv.y));
        }
        case 4u: {
            return normalize(vec3<f32>(uv.x, -uv.y, 1.0));
        }
        default: {
            return normalize(vec3<f32>(-uv.x, -uv.y, -1.0));
        }
    }
}

fn texel_direction(face: u32, pixel: vec2<u32>, face_size: u32) -> vec3<f32> {
    let size = max(face_size, 1u);
    let uv = ((vec2<f32>(pixel) + vec2<f32>(0.5, 0.5)) / f32(size)) * 2.0 - vec2<f32>(1.0, 1.0);
    return cube_face_direction(face, uv);
}

fn radical_inverse_vdc(bits_in: u32) -> f32 {
    var bits = bits_in;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * INV_UINT_MAX_PLUS_ONE;
}

fn hammersley(index: u32, sample_count: u32) -> vec2<f32> {
    return vec2<f32>(f32(index) / f32(max(sample_count, 1u)), radical_inverse_vdc(index));
}

fn tangent_from_normal(normal: vec3<f32>) -> vec3<f32> {
    var up = vec3<f32>(0.0, 0.0, 1.0);
    if (abs(normal.z) > 0.999) {
        up = vec3<f32>(1.0, 0.0, 0.0);
    }
    return normalize(cross(up, normal));
}

fn cosine_sample_hemisphere(xi: vec2<f32>, normal: vec3<f32>) -> vec3<f32> {
    let radius = sqrt(xi.x);
    let phi = 2.0 * PI * xi.y;
    let local = vec3<f32>(
        radius * cos(phi),
        radius * sin(phi),
        sqrt(max(0.0, 1.0 - xi.x)),
    );
    let tangent = tangent_from_normal(normal);
    let bitangent = cross(normal, tangent);
    return normalize(tangent * local.x + bitangent * local.y + normal * local.z);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let face_size = max(params.irradiance_face_size, 1u);
    let face = global_id.z;
    if (global_id.x >= face_size || global_id.y >= face_size || face >= 6u) {
        return;
    }

    let normal = texel_direction(face, global_id.xy, face_size);
    let sample_count = max(params.sample_count, 1u);
    var color = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0u; i < sample_count; i = i + 1u) {
        let sample_dir = cosine_sample_hemisphere(hammersley(i, sample_count), normal);
        color = color + textureSampleLevel(source_cubemap, source_sampler, sample_dir, 0.0).rgb;
    }

    let irradiance = color / f32(sample_count);
    textureStore(
        irradiance_output,
        vec2<i32>(global_id.xy),
        i32(face),
        vec4<f32>(max(irradiance, vec3<f32>(0.0, 0.0, 0.0)), 1.0),
    );
}
