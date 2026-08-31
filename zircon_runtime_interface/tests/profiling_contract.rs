use zircon_runtime_interface::profiling::{ProfileCaptureConfig, PROFILE_DEFAULT_FRAME_BUDGET_MS};

#[test]
fn profile_capture_normalization_rejects_non_finite_frame_budgets() {
    for invalid_budget in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let normalized = ProfileCaptureConfig {
            frame_budget_ms: invalid_budget,
            ..ProfileCaptureConfig::default()
        }
        .normalized();

        assert_eq!(normalized.frame_budget_ms, PROFILE_DEFAULT_FRAME_BUDGET_MS);
        assert!(normalized.frame_budget_ms.is_finite());
        assert!(normalized.frame_budget_ms.is_sign_positive());
    }
}
