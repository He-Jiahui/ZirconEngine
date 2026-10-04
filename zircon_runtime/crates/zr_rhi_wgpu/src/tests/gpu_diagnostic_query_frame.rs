use super::GpuDiagnosticQueryFramePlan;
use zr_rhi::DiagnosticReadbackBudget;

#[test]
fn timestamp_and_statistics_scopes_share_dense_logical_pass_ids() {
    let frame = GpuDiagnosticQueryFramePlan::new(41, DiagnosticReadbackBudget::default());
    let timestamp_a = frame.reserve_timestamp_scope("hzb.build").unwrap();
    let timestamp_b = frame.reserve_timestamp_scope("hzb.build").unwrap();
    let statistics = frame
        .reserve_pipeline_statistics_scope("hzb.build")
        .unwrap();
    let ui = frame.reserve_timestamp_scope("ui").unwrap();
    let snapshot = frame.snapshot();

    assert_eq!(timestamp_a.pass(), timestamp_b.pass());
    assert_eq!(timestamp_a.pass(), statistics.pass());
    assert_ne!(timestamp_a.pass(), ui.pass());
    assert_eq!(snapshot.pass_names(), ["hzb.build", "ui"]);
    assert_eq!(snapshot.plan().pass_count(), 2);
    assert_eq!(snapshot.plan().timestamp_query_count(), 6);
    assert_eq!(snapshot.plan().pipeline_statistics_query_count(), 1);
}
