use super::super::template_fields::workbench_field_metrics;

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const STEPPER_GLYPH_SEGMENTS: &[(f32, f32, f32, f32)] = &[
    (4.0, 2.0, 2.0, 2.0),
    (2.0, 4.0, 6.0, 1.4),
    (2.0, 11.0, 6.0, 1.4),
    (4.0, 13.0, 2.0, 2.0),
];
