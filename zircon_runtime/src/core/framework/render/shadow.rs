#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderShadowExecutionReport {
    pub shadow_pass_executed: bool,
    pub shadow_pass_count: usize,
    pub shadow_atlas_write_count: usize,
    pub receiver_read_pass_count: usize,
    pub receiver_available: bool,
    pub caster_draw_count: usize,
    pub alpha_mask_caster_draw_count: usize,
    pub directional_light_ready_count: usize,
}

impl RenderShadowExecutionReport {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        shadow_pass_count: usize,
        shadow_atlas_write_count: usize,
        receiver_read_pass_count: usize,
        caster_draw_count: usize,
        alpha_mask_caster_draw_count: usize,
        directional_light_ready_count: usize,
    ) -> Self {
        Self {
            shadow_pass_executed: shadow_pass_count > 0,
            shadow_pass_count,
            shadow_atlas_write_count,
            receiver_read_pass_count,
            receiver_available: shadow_atlas_write_count > 0 && receiver_read_pass_count > 0,
            caster_draw_count,
            alpha_mask_caster_draw_count,
            directional_light_ready_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RenderShadowExecutionReport;

    #[test]
    fn shadow_execution_report_keeps_receiver_availability_graph_bound() {
        let report = RenderShadowExecutionReport::new(1, 1, 3, 8, 2, 1);

        assert!(report.shadow_pass_executed);
        assert!(report.receiver_available);
        assert_eq!(report.shadow_pass_count, 1);
        assert_eq!(report.shadow_atlas_write_count, 1);
        assert_eq!(report.receiver_read_pass_count, 3);
        assert_eq!(report.caster_draw_count, 8);
        assert_eq!(report.alpha_mask_caster_draw_count, 2);
        assert_eq!(report.directional_light_ready_count, 1);
    }

    #[test]
    fn shadow_execution_report_does_not_claim_receiver_without_write_or_read() {
        let no_write = RenderShadowExecutionReport::new(1, 0, 3, 8, 2, 1);
        let no_read = RenderShadowExecutionReport::new(1, 1, 0, 8, 2, 1);

        assert!(!no_write.receiver_available);
        assert!(!no_read.receiver_available);
    }
}
