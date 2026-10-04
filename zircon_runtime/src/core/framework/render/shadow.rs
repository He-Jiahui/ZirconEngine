/// 一帧阴影图执行的诊断结果；接收端可用性只在图中同时有写入和读取时成立。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderShadowExecutionReport {
    pub shadow_pass_executed: bool,
    pub shadow_pass_count: usize,
    pub shadow_atlas_write_count: usize,
    pub receiver_read_pass_count: usize,
    pub receiver_available: bool,
    pub caster_draw_count: usize,
    pub alpha_mask_caster_draw_count: usize,
    pub shadowed_light_count: usize,
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
        shadowed_light_count: usize,
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
            shadowed_light_count,
            directional_light_ready_count,
        }
    }
}

#[cfg(test)]
#[path = "tests/shadow.rs"]
mod tests;
