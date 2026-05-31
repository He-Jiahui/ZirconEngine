pub(in crate::engine::render) fn resample_step(
    source_sample_rate_hz: u32,
    output_sample_rate_hz: u32,
) -> f64 {
    source_sample_rate_hz.max(1) as f64 / output_sample_rate_hz.max(1) as f64
}
