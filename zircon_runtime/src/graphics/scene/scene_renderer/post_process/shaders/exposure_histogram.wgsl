const EXPOSURE_BIN_COUNT: u32 = 64u;
const EXPOSURE_LOCAL_THREAD_COUNT: u32 = 256u;
const LUMINANCE_EPSILON: f32 = 0.0001;
const RGB_TO_LUMINANCE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

struct ExposureParams {
    viewport_and_mode: vec4<u32>,
    range_and_filter: vec4<f32>,
    speeds_and_compensation: vec4<f32>,
    manual_and_default: vec4<f32>,
};

@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: ExposureParams;
@group(0) @binding(2) var<storage, read_write> exposure_histogram: array<atomic<u32>, 64>;

var<workgroup> local_histogram: array<atomic<u32>, 64>;

fn luminance_to_bin(luminance: f32) -> u32 {
    if (luminance <= LUMINANCE_EPSILON) {
        return 0u;
    }

    let min_ev100 = params.range_and_filter.x;
    let max_ev100 = max(params.range_and_filter.y, min_ev100 + 0.001);
    let ev100 = clamp(log2(luminance), min_ev100, max_ev100);
    let normalized = clamp((ev100 - min_ev100) / (max_ev100 - min_ev100), 0.0, 1.0);
    return min(EXPOSURE_BIN_COUNT - 1u, 1u + u32(floor(normalized * 62.999)));
}

@compute @workgroup_size(16, 16, 1)
fn cs_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    if (local_index < EXPOSURE_BIN_COUNT) {
        atomicStore(&local_histogram[local_index], 0u);
    }
    workgroupBarrier();

    let viewport_size = params.viewport_and_mode.xy;
    if (all(global_id.xy < viewport_size)) {
        let color = max(textureLoad(scene_color_tex, vec2<i32>(global_id.xy), 0).rgb, vec3<f32>(0.0));
        let luminance = max(dot(color, RGB_TO_LUMINANCE), 0.0);
        let bin = luminance_to_bin(luminance);
        atomicAdd(&local_histogram[bin], 1u);
    }
    workgroupBarrier();

    if (local_index < EXPOSURE_BIN_COUNT) {
        let count = atomicLoad(&local_histogram[local_index]);
        if (count > 0u) {
            atomicAdd(&exposure_histogram[local_index], count);
        }
    }
}
