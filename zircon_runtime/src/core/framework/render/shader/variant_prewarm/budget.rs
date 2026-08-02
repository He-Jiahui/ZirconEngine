use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ShaderVariantPrewarmExecutionBudgetError {
    #[error(
        "shader prewarm uses one serial WGPU worker; max_in_flight_variants must be 1, got {actual}"
    )]
    ParallelWorkerCount { actual: usize },
    #[error("shader prewarm max_in_flight_source_bytes must be non-zero")]
    ZeroInFlightSourceBytes,
    #[error("shader prewarm max_resident_source_bytes must be non-zero")]
    ZeroResidentSourceBytes,
}

/// Hard bounds for the source payload owned by the serial shader-prewarm worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmExecutionBudget {
    pub max_in_flight_variants: usize,
    pub max_in_flight_source_bytes: usize,
    pub max_resident_source_bytes: usize,
}

impl Default for ShaderVariantPrewarmExecutionBudget {
    fn default() -> Self {
        Self {
            // WGPU validation owns one device context and must remain serial until a separate
            // device-safe worker pool exists.
            max_in_flight_variants: 1,
            max_in_flight_source_bytes: 8 * 1024 * 1024,
            max_resident_source_bytes: 64 * 1024 * 1024,
        }
    }
}

impl ShaderVariantPrewarmExecutionBudget {
    pub fn validate(self) -> Result<(), ShaderVariantPrewarmExecutionBudgetError> {
        if self.max_in_flight_variants != 1 {
            return Err(
                ShaderVariantPrewarmExecutionBudgetError::ParallelWorkerCount {
                    actual: self.max_in_flight_variants,
                },
            );
        }
        if self.max_in_flight_source_bytes == 0 {
            return Err(ShaderVariantPrewarmExecutionBudgetError::ZeroInFlightSourceBytes);
        }
        if self.max_resident_source_bytes == 0 {
            return Err(ShaderVariantPrewarmExecutionBudgetError::ZeroResidentSourceBytes);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmExecutionBudgetSummary {
    pub max_in_flight_variants: usize,
    pub max_in_flight_source_bytes: usize,
    pub max_resident_source_bytes: usize,
    pub resident_source_bytes: usize,
    pub peak_in_flight_variants: usize,
    pub peak_in_flight_source_bytes: usize,
    pub rejected_count: usize,
}

impl ShaderVariantPrewarmExecutionBudgetSummary {
    pub fn configure(&mut self, budget: ShaderVariantPrewarmExecutionBudget) {
        self.max_in_flight_variants = budget.max_in_flight_variants;
        self.max_in_flight_source_bytes = budget.max_in_flight_source_bytes;
        self.max_resident_source_bytes = budget.max_resident_source_bytes;
    }

    pub fn record_source_residency(&mut self, source_bytes: usize) {
        self.resident_source_bytes = source_bytes;
    }

    pub fn record_started_work(&mut self, source_bytes: usize) {
        self.peak_in_flight_variants = self.peak_in_flight_variants.max(1);
        self.peak_in_flight_source_bytes = self.peak_in_flight_source_bytes.max(source_bytes);
    }

    pub fn record_rejected(&mut self) {
        self.rejected_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmExecutionBudgetError};

    #[test]
    fn shader_prewarm_budget_rejects_unbounded_or_parallel_wgpu_work() {
        let parallel_error = ShaderVariantPrewarmExecutionBudget {
            max_in_flight_variants: 2,
            ..Default::default()
        }
        .validate()
        .expect_err("parallel WGPU work must be rejected");
        assert!(matches!(
            parallel_error,
            ShaderVariantPrewarmExecutionBudgetError::ParallelWorkerCount { actual: 2 }
        ));

        let zero_byte_error = ShaderVariantPrewarmExecutionBudget {
            max_in_flight_source_bytes: 0,
            ..Default::default()
        }
        .validate()
        .expect_err("an empty in-flight source-byte budget must be rejected");
        assert!(matches!(
            zero_byte_error,
            ShaderVariantPrewarmExecutionBudgetError::ZeroInFlightSourceBytes
        ));
    }
}
