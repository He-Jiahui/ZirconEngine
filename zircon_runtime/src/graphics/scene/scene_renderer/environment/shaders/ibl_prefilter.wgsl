const PI: f32 = 3.14159265358979323846;
const INV_UINT_MAX_PLUS_ONE: f32 = 2.3283064365386963e-10;
const FULL_ROUGHNESS_COSINE_THRESHOLD: f32 = 0.99;
const FIS_SOLID_ANGLE_TEXEL_SCALE: f32 = 2.0;

struct IblPrefilterParams {
    face_size: u32,
    mip_face_size: u32,
    mip_level: u32,
    mip_count: u32,
    sample_count: u32,
    first_face: u32,
    roughness: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> params: IblPrefilterParams;
@group(0) @binding(1) var source_cubemap: texture_cube<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var pmrem_output: texture_storage_2d_array<rgba16float, write>;

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
    return vec2<f32>((f32(index) + 0.5) / f32(max(sample_count, 1u)), radical_inverse_vdc(index));
}

fn tangent_from_normal(normal: vec3<f32>) -> vec3<f32> {
    var up = vec3<f32>(0.0, 0.0, 1.0);
    if (abs(normal.z) > 0.999) {
        up = vec3<f32>(1.0, 0.0, 0.0);
    }
    return normalize(cross(up, normal));
}

fn tangent_to_world(local: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let tangent = tangent_from_normal(normal);
    let bitangent = cross(normal, tangent);
    return normalize(tangent * local.x + bitangent * local.y + normal * local.z);
}

fn importance_sample_ggx(xi: vec2<f32>, normal: vec3<f32>, roughness: f32) -> vec3<f32> {
    let alpha = max(roughness * roughness, 0.0001);
    let alpha2 = alpha * alpha;
    let e_y = clamp(xi.y * 0.995, 0.0, 0.99999);
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - e_y) / max(1.0 + (alpha2 - 1.0) * e_y, 0.0001));
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
    let half_tangent = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
    return tangent_to_world(half_tangent, normal);
}

fn distribution_ggx(no_h: f32, roughness: f32) -> f32 {
    let alpha = max(roughness * roughness, 0.0001);
    let alpha2 = alpha * alpha;
    let denominator = max(no_h * no_h * (alpha2 - 1.0) + 1.0, 0.0001);
    return alpha2 / max(PI * denominator * denominator, 0.000001);
}

fn source_footprint_lod(source_face_size: f32, source_max_mip: f32) -> f32 {
    let destination_face_size = f32(max(params.mip_face_size, 1u));
    return clamp(log2(source_face_size / destination_face_size), 0.0, source_max_mip);
}

fn source_lod_for_pdf(
    pdf: f32,
    sample_count: u32,
    source_face_size: f32,
    source_max_mip: f32,
) -> f32 {
    let texel_solid_angle = 4.0 * PI / (6.0 * source_face_size * source_face_size)
        * FIS_SOLID_ANGLE_TEXEL_SCALE;
    let sample_solid_angle = 1.0 / (f32(max(sample_count, 1u)) * pdf);
    let lod = 0.5 * log2(max(sample_solid_angle / texel_solid_angle, 1.0));
    return clamp(lod, 0.0, source_max_mip);
}

fn source_lod_for_ggx_sample(
    no_h: f32,
    roughness: f32,
    sample_count: u32,
    source_face_size: f32,
    source_max_mip: f32,
) -> f32 {
    if (roughness <= 0.0001) {
        return 0.0;
    }
    let pdf = max(distribution_ggx(no_h, roughness) * 0.25, 0.000001);
    return source_lod_for_pdf(pdf, sample_count, source_face_size, source_max_mip);
}

fn cosine_sample_hemisphere(xi: vec2<f32>) -> vec3<f32> {
    let radius = sqrt(xi.x);
    let phi = 2.0 * PI * xi.y;
    return vec3<f32>(
        cos(phi) * radius,
        sin(phi) * radius,
        sqrt(max(0.0, 1.0 - xi.x)),
    );
}

fn cosine_prefilter_direction(
    normal: vec3<f32>,
    sample_count: u32,
    source_face_size: f32,
    source_max_mip: f32,
) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0u; i < sample_count; i = i + 1u) {
        let xi = hammersley(i, sample_count);
        let local_direction = cosine_sample_hemisphere(xi);
        let pdf = max(local_direction.z / PI, 0.000001);
        let lod = source_lod_for_pdf(pdf, sample_count, source_face_size, source_max_mip);
        let light_dir = tangent_to_world(local_direction, normal);
        color = color + textureSampleLevel(source_cubemap, source_sampler, light_dir, lod).rgb;
    }

    return color / f32(max(sample_count, 1u));
}

fn final_pmrem_face_average(
    sample_count: u32,
    source_face_size: f32,
    source_max_mip: f32,
) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    for (var face = 0u; face < 6u; face = face + 1u) {
        let face_axis = cube_face_direction(face, vec2<f32>(0.0, 0.0));
        color = color + cosine_prefilter_direction(
            face_axis,
            sample_count,
            source_face_size,
            source_max_mip,
        );
    }

    return color / 6.0;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let mip_size = max(params.mip_face_size, 1u);
    let face = params.first_face + global_id.z;
    if (global_id.x >= mip_size || global_id.y >= mip_size || face >= 6u) {
        return;
    }

    let source_face_size = f32(max(textureDimensions(source_cubemap).x, 1u));
    let source_max_mip = f32(max(textureNumLevels(source_cubemap), 1u) - 1u);
    let normal = texel_direction(face, global_id.xy, mip_size);
    if (params.mip_level == 0u || params.roughness <= 0.0001) {
        let source = textureSampleLevel(
            source_cubemap,
            source_sampler,
            normal,
            source_footprint_lod(source_face_size, source_max_mip),
        ).rgb;
        textureStore(pmrem_output, vec2<i32>(global_id.xy), i32(face), vec4<f32>(source, 1.0));
        return;
    }

    let sample_count = max(params.sample_count, 1u);
    if (params.mip_level + 1u >= params.mip_count && mip_size == 1u) {
        let filtered = final_pmrem_face_average(
            sample_count,
            source_face_size,
            source_max_mip,
        );
        textureStore(pmrem_output, vec2<i32>(global_id.xy), i32(face), vec4<f32>(filtered, 1.0));
        return;
    }

    var color = vec3<f32>(0.0, 0.0, 0.0);
    var weight_sum = 0.0;
    if (params.roughness >= FULL_ROUGHNESS_COSINE_THRESHOLD) {
        let filtered = cosine_prefilter_direction(
            normal,
            sample_count,
            source_face_size,
            source_max_mip,
        );
        textureStore(pmrem_output, vec2<i32>(global_id.xy), i32(face), vec4<f32>(filtered, 1.0));
        return;
    }

    for (var i = 0u; i < sample_count; i = i + 1u) {
        let xi = hammersley(i, sample_count);
        let half_vector = importance_sample_ggx(xi, normal, params.roughness);
        let light_dir = normalize(2.0 * dot(normal, half_vector) * half_vector - normal);
        let no_l = max(dot(normal, light_dir), 0.0);
        if (no_l > 0.0) {
            let no_h = max(dot(normal, half_vector), 0.0);
            let lod = source_lod_for_ggx_sample(
                no_h,
                params.roughness,
                sample_count,
                source_face_size,
                source_max_mip,
            );
            color = color + textureSampleLevel(source_cubemap, source_sampler, light_dir, lod).rgb * no_l;
            weight_sum = weight_sum + no_l;
        }
    }

    let filtered = color / max(weight_sum, 0.0001);
    textureStore(pmrem_output, vec2<i32>(global_id.xy), i32(face), vec4<f32>(filtered, 1.0));
}
