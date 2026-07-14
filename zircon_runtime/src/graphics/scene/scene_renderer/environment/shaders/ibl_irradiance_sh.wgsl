const PI: f32 = 3.14159265358979323846;
const WORKGROUP_THREAD_COUNT: u32 = 64u;
const CUBE_FACE_COUNT: u32 = 6u;

struct IblIrradianceShParams {
    source_face_size: u32,
    sample_face_size: u32,
    source_lod: f32,
    _pad0: u32,
};

struct IblIrradianceShOutput {
    coeffs: array<vec4<f32>, 9>,
};

@group(0) @binding(0) var<uniform> params: IblIrradianceShParams;
@group(0) @binding(1) var source_cubemap: texture_cube<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var<storage, read_write> sh9_output: IblIrradianceShOutput;

var<workgroup> weight_shared: array<f32, 64>;
var<workgroup> sh0_shared: array<vec3<f32>, 64>;
var<workgroup> sh1_shared: array<vec3<f32>, 64>;
var<workgroup> sh2_shared: array<vec3<f32>, 64>;
var<workgroup> sh3_shared: array<vec3<f32>, 64>;
var<workgroup> sh4_shared: array<vec3<f32>, 64>;
var<workgroup> sh5_shared: array<vec3<f32>, 64>;
var<workgroup> sh6_shared: array<vec3<f32>, 64>;
var<workgroup> sh7_shared: array<vec3<f32>, 64>;
var<workgroup> sh8_shared: array<vec3<f32>, 64>;

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

fn area_element(x: f32, y: f32) -> f32 {
    return atan2(x * y, sqrt(x * x + y * y + 1.0));
}

fn texel_solid_angle(pixel: vec2<u32>, face_size: u32) -> f32 {
    let size = max(face_size, 1u);
    let inv_size = 2.0 / f32(size);
    let x0 = f32(pixel.x) * inv_size - 1.0;
    let y0 = f32(pixel.y) * inv_size - 1.0;
    let x1 = f32(pixel.x + 1u) * inv_size - 1.0;
    let y1 = f32(pixel.y + 1u) * inv_size - 1.0;
    return area_element(x0, y0) - area_element(x0, y1) - area_element(x1, y0) + area_element(x1, y1);
}

fn accumulate_sh(dir: vec3<f32>, radiance: vec3<f32>, weight: f32, coeff_index: u32) -> vec3<f32> {
    let x = dir.x;
    let y = dir.y;
    let z = dir.z;
    switch coeff_index {
        case 0u: {
            return radiance * (0.282095 * weight);
        }
        case 1u: {
            return radiance * (0.488603 * z * weight);
        }
        case 2u: {
            return radiance * (0.488603 * y * weight);
        }
        case 3u: {
            return radiance * (0.488603 * x * weight);
        }
        case 4u: {
            return radiance * (1.092548 * x * z * weight);
        }
        case 5u: {
            return radiance * (1.092548 * y * z * weight);
        }
        case 6u: {
            return radiance * (0.315392 * (3.0 * y * y - 1.0) * weight);
        }
        case 7u: {
            return radiance * (1.092548 * x * y * weight);
        }
        default: {
            return radiance * (0.546274 * (x * x - z * z) * weight);
        }
    }
}

fn write_coeff(index: u32, value: vec3<f32>, band_scale: f32) {
    sh9_output.coeffs[index] = vec4<f32>(value * band_scale, 0.0);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(local_invocation_index) local_invocation_index: u32) {
    let sample_size = max(params.sample_face_size, 1u);
    let samples_per_face = sample_size * sample_size;
    let total_sample_count = CUBE_FACE_COUNT * samples_per_face;
    var weight_sum = 0.0;
    var c0 = vec3<f32>(0.0, 0.0, 0.0);
    var c1 = vec3<f32>(0.0, 0.0, 0.0);
    var c2 = vec3<f32>(0.0, 0.0, 0.0);
    var c3 = vec3<f32>(0.0, 0.0, 0.0);
    var c4 = vec3<f32>(0.0, 0.0, 0.0);
    var c5 = vec3<f32>(0.0, 0.0, 0.0);
    var c6 = vec3<f32>(0.0, 0.0, 0.0);
    var c7 = vec3<f32>(0.0, 0.0, 0.0);
    var c8 = vec3<f32>(0.0, 0.0, 0.0);

    var sample_index = local_invocation_index;
    while (sample_index < total_sample_count) {
        let face = sample_index / samples_per_face;
        let face_sample_index = sample_index % samples_per_face;
        let pixel = vec2<u32>(face_sample_index % sample_size, face_sample_index / sample_size);
        let dir = texel_direction(face, pixel, sample_size);
        let weight = texel_solid_angle(pixel, sample_size);
        let radiance = textureSampleLevel(source_cubemap, source_sampler, dir, params.source_lod).rgb;
        weight_sum = weight_sum + weight;
        c0 = c0 + accumulate_sh(dir, radiance, weight, 0u);
        c1 = c1 + accumulate_sh(dir, radiance, weight, 1u);
        c2 = c2 + accumulate_sh(dir, radiance, weight, 2u);
        c3 = c3 + accumulate_sh(dir, radiance, weight, 3u);
        c4 = c4 + accumulate_sh(dir, radiance, weight, 4u);
        c5 = c5 + accumulate_sh(dir, radiance, weight, 5u);
        c6 = c6 + accumulate_sh(dir, radiance, weight, 6u);
        c7 = c7 + accumulate_sh(dir, radiance, weight, 7u);
        c8 = c8 + accumulate_sh(dir, radiance, weight, 8u);
        sample_index = sample_index + WORKGROUP_THREAD_COUNT;
    }

    weight_shared[local_invocation_index] = weight_sum;
    sh0_shared[local_invocation_index] = c0;
    sh1_shared[local_invocation_index] = c1;
    sh2_shared[local_invocation_index] = c2;
    sh3_shared[local_invocation_index] = c3;
    sh4_shared[local_invocation_index] = c4;
    sh5_shared[local_invocation_index] = c5;
    sh6_shared[local_invocation_index] = c6;
    sh7_shared[local_invocation_index] = c7;
    sh8_shared[local_invocation_index] = c8;
    workgroupBarrier();

    var reduction_stride = WORKGROUP_THREAD_COUNT / 2u;
    while (reduction_stride > 0u) {
        if (local_invocation_index < reduction_stride) {
            let other = local_invocation_index + reduction_stride;
            weight_shared[local_invocation_index] = weight_shared[local_invocation_index] + weight_shared[other];
            sh0_shared[local_invocation_index] = sh0_shared[local_invocation_index] + sh0_shared[other];
            sh1_shared[local_invocation_index] = sh1_shared[local_invocation_index] + sh1_shared[other];
            sh2_shared[local_invocation_index] = sh2_shared[local_invocation_index] + sh2_shared[other];
            sh3_shared[local_invocation_index] = sh3_shared[local_invocation_index] + sh3_shared[other];
            sh4_shared[local_invocation_index] = sh4_shared[local_invocation_index] + sh4_shared[other];
            sh5_shared[local_invocation_index] = sh5_shared[local_invocation_index] + sh5_shared[other];
            sh6_shared[local_invocation_index] = sh6_shared[local_invocation_index] + sh6_shared[other];
            sh7_shared[local_invocation_index] = sh7_shared[local_invocation_index] + sh7_shared[other];
            sh8_shared[local_invocation_index] = sh8_shared[local_invocation_index] + sh8_shared[other];
        }
        workgroupBarrier();
        reduction_stride = reduction_stride / 2u;
    }

    if (local_invocation_index == 0u) {
        let normalization = (4.0 * PI) / max(weight_shared[0], 0.0001);
        write_coeff(0u, sh0_shared[0] * normalization, 1.0);
        write_coeff(1u, sh1_shared[0] * normalization, 0.6666667);
        write_coeff(2u, sh2_shared[0] * normalization, 0.6666667);
        write_coeff(3u, sh3_shared[0] * normalization, 0.6666667);
        write_coeff(4u, sh4_shared[0] * normalization, 0.25);
        write_coeff(5u, sh5_shared[0] * normalization, 0.25);
        write_coeff(6u, sh6_shared[0] * normalization, 0.25);
        write_coeff(7u, sh7_shared[0] * normalization, 0.25);
        write_coeff(8u, sh8_shared[0] * normalization, 0.25);
    }
}
