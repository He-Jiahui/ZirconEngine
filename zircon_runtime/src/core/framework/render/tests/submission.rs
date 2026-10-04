use super::{RenderSubmissionConfig, DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET};

#[test]
fn submission_defaults_preserve_synchronous_serial_recording() {
    assert_eq!(
        RenderSubmissionConfig::default(),
        RenderSubmissionConfig::synchronous()
    );
    assert!(!RenderSubmissionConfig::default().parallel_record);
    assert!(!RenderSubmissionConfig::default().allow_gpu_timing);
    assert!(!RenderSubmissionConfig::default().hzb_diagnostics_readback);
    assert!(!RenderSubmissionConfig::default().async_pipeline_compile);
    assert_eq!(
        RenderSubmissionConfig::default().min_passes_per_bucket,
        DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET
    );
}

#[test]
fn missing_submission_fields_deserialize_to_synchronous_recording() {
    let config: RenderSubmissionConfig =
        serde_json::from_str("{}").expect("empty submission config should deserialize");

    assert_eq!(config, RenderSubmissionConfig::synchronous());
}

#[test]
fn parallel_recording_is_explicit_and_clamps_the_bucket_threshold() {
    let config = RenderSubmissionConfig::synchronous().with_parallel_recording(0);

    assert!(config.parallel_record);
    assert_eq!(config.min_passes_per_bucket, 1);
    assert!(!RenderSubmissionConfig::pipelined().parallel_record);
}

#[test]
fn async_pipeline_compile_is_explicit_and_independent_from_submission_pipelining() {
    let config = RenderSubmissionConfig::synchronous().with_async_pipeline_compile();

    assert!(config.async_pipeline_compile);
    assert!(!config.pipelined_render);
    assert!(!RenderSubmissionConfig::pipelined().async_pipeline_compile);
}

#[test]
fn gpu_timing_is_explicit_and_independent_from_submission_pipelining() {
    let config = RenderSubmissionConfig::synchronous().with_gpu_timing();

    assert!(config.allow_gpu_timing);
    assert!(!config.pipelined_render);
    assert!(!RenderSubmissionConfig::pipelined().allow_gpu_timing);
}

#[test]
fn hzb_cpu_diagnostics_readback_is_explicit_and_disabled_by_default() {
    let config = RenderSubmissionConfig::synchronous().with_hzb_diagnostics_readback();

    assert!(config.hzb_diagnostics_readback);
    assert!(!RenderSubmissionConfig::default().hzb_diagnostics_readback);
    assert!(!RenderSubmissionConfig::pipelined().hzb_diagnostics_readback);
}
