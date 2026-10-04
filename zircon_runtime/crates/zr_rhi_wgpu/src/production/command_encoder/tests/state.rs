use super::{pop_debug_group, DebugGroupScope};
use zr_rhi::RhiError;

#[test]
fn debug_group_pop_reports_compute_pass_scope_mismatch() {
    let mut groups = vec![DebugGroupScope::ComputePass];

    assert_eq!(
        pop_debug_group(&mut groups, DebugGroupScope::CommandEncoder).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason:
                "pop_debug_group must close a compute-pass debug group inside the active compute pass"
                    .to_string(),
        }
    );
    assert_eq!(groups, vec![DebugGroupScope::ComputePass]);
}
