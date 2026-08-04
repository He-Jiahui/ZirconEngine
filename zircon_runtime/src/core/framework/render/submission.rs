use serde::{Deserialize, Serialize};

pub const DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET: usize = 2;

/// Runtime scheduling and opt-in observation controls for render submission.
///
/// The default preserves synchronous offscreen and native-surface submission.
/// Pipelining reports the result of frame N when frame N + 1 is submitted.
/// An explicit frame capture synchronizes pending work and reports its result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSubmissionConfig {
    #[serde(default)]
    pub pipelined_render: bool,
    #[serde(default)]
    pub parallel_record: bool,
    #[serde(default = "default_parallel_record_min_passes_per_bucket")]
    pub min_passes_per_bucket: usize,
    /// Creates timestamp-query resources only while explicitly enabled and supported.
    #[serde(default)]
    pub allow_gpu_timing: bool,
    /// Enables asynchronous CPU inspection of GPU-resident HZB indirect arguments.
    #[serde(default)]
    pub hzb_indirect_args_readback: bool,
    #[serde(default)]
    pub async_pipeline_compile: bool,
}

impl RenderSubmissionConfig {
    pub const fn synchronous() -> Self {
        Self {
            pipelined_render: false,
            parallel_record: false,
            min_passes_per_bucket: DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET,
            allow_gpu_timing: false,
            hzb_indirect_args_readback: false,
            async_pipeline_compile: false,
        }
    }

    pub const fn pipelined() -> Self {
        Self {
            pipelined_render: true,
            ..Self::synchronous()
        }
    }

    pub const fn with_parallel_recording(mut self, min_passes_per_bucket: usize) -> Self {
        self.parallel_record = true;
        self.min_passes_per_bucket = if min_passes_per_bucket == 0 {
            1
        } else {
            min_passes_per_bucket
        };
        self
    }

    pub const fn with_async_pipeline_compile(mut self) -> Self {
        self.async_pipeline_compile = true;
        self
    }

    pub const fn with_gpu_timing(mut self) -> Self {
        self.allow_gpu_timing = true;
        self
    }

    pub const fn with_hzb_indirect_args_readback(mut self) -> Self {
        self.hzb_indirect_args_readback = true;
        self
    }
}

const fn default_parallel_record_min_passes_per_bucket() -> usize {
    DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET
}

impl Default for RenderSubmissionConfig {
    fn default() -> Self {
        Self::synchronous()
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderSubmissionConfig, DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET};

    #[test]
    fn submission_defaults_preserve_synchronous_serial_recording() {
        assert_eq!(
            RenderSubmissionConfig::default(),
            RenderSubmissionConfig::synchronous()
        );
        assert!(!RenderSubmissionConfig::default().parallel_record);
        assert!(!RenderSubmissionConfig::default().allow_gpu_timing);
        assert!(!RenderSubmissionConfig::default().hzb_indirect_args_readback);
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
    fn hzb_indirect_args_readback_is_explicit_and_disabled_by_default() {
        let config = RenderSubmissionConfig::synchronous().with_hzb_indirect_args_readback();

        assert!(config.hzb_indirect_args_readback);
        assert!(!RenderSubmissionConfig::default().hzb_indirect_args_readback);
        assert!(!RenderSubmissionConfig::pipelined().hzb_indirect_args_readback);
    }
}
