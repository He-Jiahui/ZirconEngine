const PI: f32 = 3.14159265358979323846;

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
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (any(global_id != vec3<u32>(0u, 0u, 0u))) {
        return;
    }

    let sample_size = max(params.sample_face_size, 1u);
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

    for (var face = 0u; face < 6u; face = face + 1u) {
        for (var y = 0u; y < sample_size; y = y + 1u) {
            for (var x = 0u; x < sample_size; x = x + 1u) {
                let pixel = vec2<u32>(x, y);
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
            }
        }
    }

    let normalization = (4.0 * PI) / max(weight_sum, 0.0001);
    write_coeff(0u, c0 * normalization, 1.0);
    write_coeff(1u, c1 * normalization, 0.6666667);
    write_coeff(2u, c2 * normalization, 0.6666667);
    write_coeff(3u, c3 * normalization, 0.6666667);
    write_coeff(4u, c4 * normalization, 0.25);
    write_coeff(5u, c5 * normalization, 0.25);
    write_coeff(6u, c6 * normalization, 0.25);
    write_coeff(7u, c7 * normalization, 0.25);
    write_coeff(8u, c8 * normalization, 0.25);
}
