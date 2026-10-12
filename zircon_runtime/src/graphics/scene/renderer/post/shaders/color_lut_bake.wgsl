// 把曝光、tonemap、用户 LUT 和 grading 合成内部 3D LUT，供显示映射 pass 复用。
// CPU 的 binding mode 区分逐通道曲线、二维 strip 与三维 LUT；两类纹理槽均须有合法绑定。
// 布局与 ColorLutBakeParams 对应；曝光 buffer 必须是本帧 ExposureResolve 的产物。
struct ColorLutBakeParams {
    lut_size_and_flags: vec4<u32>,
    tonemap_lut: vec4<f32>,
    grading: vec4<f32>,
    tint_and_exposure: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: ColorLutBakeParams;
@group(0) @binding(1) var<storage, read> exposure_buffer: array<vec4<f32>, 1>;
@group(0) @binding(2) var user_lut_tex: texture_2d<f32>;
@group(0) @binding(3) var user_lut_3d_tex: texture_3d<f32>;
@group(0) @binding(4) var user_lut_sampler: sampler;
@group(0) @binding(5) var color_lut_out: texture_storage_3d<rgba16float, write>;

const USER_LUT_2D: u32 = 1u;
const USER_LUT_2D_STRIP: u32 = 2u;
const USER_LUT_3D: u32 = 3u;

fn lut_axis_index(value: f32, size: u32) -> u32 {
    let max_index = max(size, 1u) - 1u;
    return u32(round(clamp(value, 0.0, 1.0) * f32(max_index)));
}

// 普通二维 LUT 在此约定为第零行的单通道曲线，并分别作用于 RGB；
// 二维 strip 则把蓝轴切片横向排布，资源准备端须先确认相应布局。
fn sample_user_lut_1d_channel(value: f32) -> f32 {
    let dims = textureDimensions(user_lut_tex);
    let x = i32(lut_axis_index(value, dims.x));
    return textureLoad(user_lut_tex, vec2<i32>(x, 0), 0).r;
}

fn sample_user_lut_1d(color: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        sample_user_lut_1d_channel(color.r),
        sample_user_lut_1d_channel(color.g),
        sample_user_lut_1d_channel(color.b)
    );
}

fn sample_user_lut_2d_strip(color: vec3<f32>) -> vec3<f32> {
    let dims = textureDimensions(user_lut_tex);
    let size = max(dims.y, 1u);
    let red = lut_axis_index(color.r, size);
    let green = lut_axis_index(color.g, size);
    let blue = lut_axis_index(color.b, size);
    let x = min(blue * size + red, max(dims.x, 1u) - 1u);
    let y = min(green, size - 1u);
    return textureLoad(user_lut_tex, vec2<i32>(i32(x), i32(y)), 0).rgb;
}

// 纹理坐标落在 LUT 格点中心，以线性采样在颜色格点间插值；输入域为归一化颜色。
fn sample_user_lut_3d(color: vec3<f32>) -> vec3<f32> {
    let dims_u32 = textureDimensions(user_lut_3d_tex);
    let dims = vec3<f32>(f32(dims_u32.x), f32(dims_u32.y), f32(dims_u32.z));
    let axis_max = max(dims - vec3<f32>(1.0), vec3<f32>(0.0));
    let sample_coord =
        (clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * axis_max + vec3<f32>(0.5)) / dims;
    return textureSampleLevel(user_lut_3d_tex, user_lut_sampler, sample_coord, 0.0).rgb;
}

fn sample_user_lut(color: vec3<f32>) -> vec3<f32> {
    let binding_mode = params.lut_size_and_flags.y;
    if (binding_mode == USER_LUT_3D) {
        return sample_user_lut_3d(color);
    }
    if (binding_mode == USER_LUT_2D_STRIP) {
        return sample_user_lut_2d_strip(color);
    }
    return sample_user_lut_1d(color);
}

fn apply_tonemap(color: vec3<f32>) -> vec3<f32> {
    let exposure = exp2(params.tonemap_lut.x) * max(exposure_buffer[0].x, 0.0);
    let white_point = max(params.tonemap_lut.y, 0.001);
    var mapped = max(color * exposure, vec3<f32>(0.0));
    if (params.lut_size_and_flags.z == 1u) {
        mapped = mapped / (vec3<f32>(1.0) + mapped / white_point);
    } else if (params.lut_size_and_flags.z == 2u) {
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;
        mapped = clamp(
            (mapped * (a * mapped + vec3<f32>(b)))
                / (mapped * (c * mapped + vec3<f32>(d)) + vec3<f32>(e)),
            vec3<f32>(0.0),
            vec3<f32>(1.0)
        );
    } else if (params.lut_size_and_flags.z == 3u) {
        mapped = max(vec3<f32>(0.0), mapped - vec3<f32>(0.004));
        mapped = (mapped * (6.2 * mapped + vec3<f32>(0.5)))
            / (mapped * (6.2 * mapped + vec3<f32>(1.7)) + vec3<f32>(0.06));
    }

    let lut_intensity = clamp(params.tonemap_lut.z, 0.0, 1.0);
    if (params.lut_size_and_flags.y != 0u && lut_intensity > 0.0) {
        mapped = mix(mapped, sample_user_lut(mapped), lut_intensity);
    }
    return mapped;
}

fn apply_color_grading(color: vec3<f32>) -> vec3<f32> {
    let exposure = params.grading.x;
    let contrast = params.grading.y;
    let saturation = params.grading.z;
    let gamma = params.grading.w;
    var graded = color * exposure;
    let luma = dot(graded, vec3<f32>(0.2126, 0.7152, 0.0722));
    graded = mix(vec3<f32>(luma), graded, saturation);
    graded = ((graded - vec3<f32>(0.5)) * contrast) + vec3<f32>(0.5);
    graded = max(graded, vec3<f32>(0.0));
    graded = pow(graded, vec3<f32>(1.0 / max(gamma, 0.001)));
    return graded * params.tint_and_exposure.rgb;
}

// 烘焙顺序与未烘焙显示路径一致；使用内部 LUT 的消费者须跳过再次 grading。
// 格点源颜色目前覆盖 [0,1]，HDR 输入域的约定见消费端 CR-POST-SHADER-0004。
@compute @workgroup_size(4, 4, 4)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let lut_size = max(params.lut_size_and_flags.x, 1u);
    if (any(global_id >= vec3<u32>(lut_size))) {
        return;
    }

    let axis_max = max(f32(lut_size - 1u), 1.0);
    let source_color = vec3<f32>(global_id) / vec3<f32>(axis_max);
    let baked = apply_color_grading(apply_tonemap(source_color));
    textureStore(color_lut_out, vec3<i32>(global_id), vec4<f32>(baked, 1.0));
}
