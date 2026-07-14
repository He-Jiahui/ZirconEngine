pub(super) fn normalize_percentile(values: &[f32], low: f32, high: f32) -> Vec<f32> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let low = sorted[((sorted.len() - 1) as f32 * low).round() as usize];
    let high = sorted[((sorted.len() - 1) as f32 * high).round() as usize].max(low + 1.0e-6);
    values
        .iter()
        .map(|value| ((*value - low) / (high - low)).clamp(0.0, 1.0))
        .collect()
}

pub(super) fn global_ssim(first: &[f32], second: &[f32]) -> f32 {
    assert_eq!(first.len(), second.len());
    let count = first.len() as f32;
    let first_mean = first.iter().sum::<f32>() / count;
    let second_mean = second.iter().sum::<f32>() / count;
    let mut first_variance = 0.0;
    let mut second_variance = 0.0;
    let mut covariance = 0.0;
    for (&first, &second) in first.iter().zip(second) {
        first_variance += (first - first_mean).powi(2);
        second_variance += (second - second_mean).powi(2);
        covariance += (first - first_mean) * (second - second_mean);
    }
    first_variance /= count;
    second_variance /= count;
    covariance /= count;
    let c1 = 0.01_f32.powi(2);
    let c2 = 0.03_f32.powi(2);
    ((2.0 * first_mean * second_mean + c1) * (2.0 * covariance + c2))
        / ((first_mean.powi(2) + second_mean.powi(2) + c1)
            * (first_variance + second_variance + c2))
}

pub(super) fn linear_rgb_to_lab(rgb: [f32; 3]) -> [f32; 3] {
    let mapped = rgb.map(|value| value.max(0.0) / (1.0 + value.max(0.0)));
    let xyz = [
        mapped[0] * 0.412_456_4 + mapped[1] * 0.357_576_1 + mapped[2] * 0.180_437_5,
        mapped[0] * 0.212_672_9 + mapped[1] * 0.715_152_2 + mapped[2] * 0.072_175,
        mapped[0] * 0.019_333_9 + mapped[1] * 0.119_192 + mapped[2] * 0.950_304_1,
    ];
    let f = |value: f32| {
        if value > 0.008_856 {
            value.cbrt()
        } else {
            7.787 * value + 16.0 / 116.0
        }
    };
    let fx = f(xyz[0] / 0.950_47);
    let fy = f(xyz[1]);
    let fz = f(xyz[2] / 1.088_83);
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

pub(super) fn delta_e_76(first: [f32; 3], second: [f32; 3]) -> f32 {
    ((first[0] - second[0]).powi(2)
        + (first[1] - second[1]).powi(2)
        + (first[2] - second[2]).powi(2))
    .sqrt()
}

pub(super) fn variance(values: &[f32]) -> f32 {
    let mean = mean(values);
    values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / values.len().max(1) as f32
}

pub(super) fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len().max(1) as f32
}

pub(super) fn rotate_y(value: [f32; 3], radians: f32) -> [f32; 3] {
    let (sin, cos) = radians.sin_cos();
    [
        value[0] * cos - value[2] * sin,
        value[1],
        value[0] * sin + value[2] * cos,
    ]
}

pub(super) fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let inverse_length = 1.0
        / (value[0] * value[0] + value[1] * value[1] + value[2] * value[2])
            .sqrt()
            .max(f32::EPSILON);
    mul3_scalar(value, inverse_length)
}

pub(super) fn mix3(first: [f32; 3], second: [f32; 3], amount: f32) -> [f32; 3] {
    lerp3(first, second, amount.clamp(0.0, 1.0))
}

pub(super) fn lerp2(first: [f32; 2], second: [f32; 2], amount: f32) -> [f32; 2] {
    [
        first[0] + (second[0] - first[0]) * amount,
        first[1] + (second[1] - first[1]) * amount,
    ]
}

pub(super) fn lerp3(first: [f32; 3], second: [f32; 3], amount: f32) -> [f32; 3] {
    [
        first[0] + (second[0] - first[0]) * amount,
        first[1] + (second[1] - first[1]) * amount,
        first[2] + (second[2] - first[2]) * amount,
    ]
}

pub(super) fn lerp4(first: [f32; 4], second: [f32; 4], amount: f32) -> [f32; 4] {
    [
        first[0] + (second[0] - first[0]) * amount,
        first[1] + (second[1] - first[1]) * amount,
        first[2] + (second[2] - first[2]) * amount,
        first[3] + (second[3] - first[3]) * amount,
    ]
}

pub(super) fn mul3_scalar(value: [f32; 3], scalar: f32) -> [f32; 3] {
    [value[0] * scalar, value[1] * scalar, value[2] * scalar]
}

pub(super) fn mul3_components(first: [f32; 3], second: [f32; 3]) -> [f32; 3] {
    [
        first[0] * second[0],
        first[1] * second[1],
        first[2] * second[2],
    ]
}

pub(super) fn div3_components(first: [f32; 3], second: [f32; 3]) -> [f32; 3] {
    [
        first[0] / second[0],
        first[1] / second[1],
        first[2] / second[2],
    ]
}

pub(super) fn sub3(first: [f32; 3], second: [f32; 3]) -> [f32; 3] {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

pub(super) fn max3(value: [f32; 3], minimum: f32) -> [f32; 3] {
    [
        value[0].max(minimum),
        value[1].max(minimum),
        value[2].max(minimum),
    ]
}
