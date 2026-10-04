use super::RenderShadowExecutionReport;

#[test]
fn shadow_execution_report_keeps_receiver_availability_graph_bound() {
    let report = RenderShadowExecutionReport::new(1, 1, 3, 8, 2, 4, 1);

    assert!(report.shadow_pass_executed);
    assert!(report.receiver_available);
    assert_eq!(report.shadow_pass_count, 1);
    assert_eq!(report.shadow_atlas_write_count, 1);
    assert_eq!(report.receiver_read_pass_count, 3);
    assert_eq!(report.caster_draw_count, 8);
    assert_eq!(report.alpha_mask_caster_draw_count, 2);
    assert_eq!(report.shadowed_light_count, 4);
    assert_eq!(report.directional_light_ready_count, 1);
}

#[test]
fn shadow_execution_report_does_not_claim_receiver_without_write_or_read() {
    let no_write = RenderShadowExecutionReport::new(1, 0, 3, 8, 2, 4, 1);
    let no_read = RenderShadowExecutionReport::new(1, 1, 0, 8, 2, 4, 1);

    assert!(!no_write.receiver_available);
    assert!(!no_read.receiver_available);
}
