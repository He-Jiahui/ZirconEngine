//! 步进器参数直接来自字段共享密度，文字槽和分隔线/图标对同一宽度达成一致。

use super::super::template_fields::workbench_field_metrics;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 从 WorkbenchFieldMetrics 只提取步进器需要的几何字段，避免两套独立尺寸来源。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchFieldStepperMetrics
{
    pub width: f32,
    pub divider_width: f32,
    pub divider_inset_y: f32,
    pub glyph_left_inset: f32,
    pub glyph_width: f32,
    pub glyph_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_field_stepper_metrics(
) -> WorkbenchFieldStepperMetrics {
    let metrics = workbench_field_metrics();
    WorkbenchFieldStepperMetrics {
        width: metrics.stepper_width,
        divider_width: metrics.stepper_divider_width,
        divider_inset_y: metrics.stepper_divider_inset_y,
        glyph_left_inset: metrics.stepper_glyph_left_inset,
        glyph_width: metrics.stepper_glyph_width,
        glyph_height: metrics.stepper_glyph_height,
    }
}
