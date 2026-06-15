const EXPOSURE_BIN_COUNT: u32 = 64u;
const EXPOSURE_MODE_HISTOGRAM: u32 = 1u;

struct ExposureParams {
    viewport_and_mode: vec4<u32>,
    range_and_filter: vec4<f32>,
    speeds_and_compensation: vec4<f32>,
    manual_and_default: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: ExposureParams;
@group(0) @binding(1) var<storage, read> exposure_histogram: array<u32, 64>;
@group(0) @binding(2) var<storage, read> previous_exposure: array<vec4<f32>, 1>;
@group(0) @binding(3) var<storage, read_write> current_exposure: array<vec4<f32>, 1>;

fn histogram_bin_to_ev100(bin: u32) -> f32 {
    let min_ev100 = params.range_and_filter.x;
    let max_ev100 = max(params.range_and_filter.y, min_ev100 + 0.001);
    if (bin == 0u) {
        return min_ev100;
    }
    let normalized = f32(bin - 1u) / f32(EXPOSURE_BIN_COUNT - 2u);
    return mix(min_ev100, max_ev100, clamp(normalized, 0.0, 1.0));
}

fn histogram_average_ev100() -> f32 {
    let low_percent = clamp(params.range_and_filter.z, 0.0, 1.0);
    let high_percent = clamp(params.range_and_filter.w, low_percent, 1.0);
    let pixel_count = max(params.manual_and_default.z, 1.0);
    let low_cut = u32(floor(pixel_count * low_percent));
    let high_cut = max(low_cut + 1u, u32(ceil(pixel_count * high_percent)));

    var cumulative = 0u;
    var weighted_sum = 0.0;
    var weighted_count = 0u;

    for (var bin = 0u; bin < EXPOSURE_BIN_COUNT; bin = bin + 1u) {
        let count = exposure_histogram[bin];
        let next_cumulative = cumulative + count;
        let clipped_begin = max(cumulative, low_cut);
        let clipped_end = min(next_cumulative, high_cut);
        if (clipped_end > clipped_begin) {
            let weight = clipped_end - clipped_begin;
            weighted_sum = weighted_sum + histogram_bin_to_ev100(bin) * f32(weight);
            weighted_count = weighted_count + weight;
        }
        cumulative = next_cumulative;
    }

    if (weighted_count == 0u) {
        return params.manual_and_default.x;
    }
    return weighted_sum / f32(weighted_count);
}

fn adapt_ev100(target_ev100: f32) -> f32 {
    let previous = previous_exposure[0];
    if (previous.w <= 0.5) {
        return target_ev100;
    }

    let previous_ev100 = previous.y;
    let brighten_speed = max(params.speeds_and_compensation.x, 0.0);
    let darken_speed = max(params.speeds_and_compensation.y, 0.0);
    let delta_seconds = max(params.speeds_and_compensation.w, 0.0);
    let speed = select(darken_speed, brighten_speed, target_ev100 < previous_ev100);
    let blend = clamp(1.0 - exp2(-speed * delta_seconds), 0.0, 1.0);
    return mix(previous_ev100, target_ev100, blend);
}

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (any(global_id.xyz != vec3<u32>(0u))) {
        return;
    }

    let mode = params.viewport_and_mode.z;
    let compensation_ev = params.speeds_and_compensation.z;
    let default_ev100 = params.manual_and_default.y;
    var average_ev100 = params.manual_and_default.x;
    var resolved_ev100 = params.manual_and_default.x;

    if (mode == EXPOSURE_MODE_HISTOGRAM) {
        average_ev100 = histogram_average_ev100();
        resolved_ev100 = adapt_ev100(average_ev100);
    }

    let multiplier = exp2(default_ev100 - resolved_ev100 + compensation_ev);
    current_exposure[0] = vec4<f32>(max(multiplier, 0.0), resolved_ev100, average_ev100, 1.0);
}
