use serde::{Deserialize, Serialize};

pub const DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET: usize = 2;

/// Runtime scheduling controls for render submission.
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
}

impl RenderSubmissionConfig {
    pub const fn synchronous() -> Self {
        Self {
            pipelined_render: false,
            parallel_record: false,
            min_passes_per_bucket: DEFAULT_PARALLEL_RECORD_MIN_PASSES_PER_BUCKET,
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
}
