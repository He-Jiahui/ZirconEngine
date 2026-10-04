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
