use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(super) const TABLE_CELL_FONT_SIZE: f32 = METRICS.font_body;
pub(super) const TABLE_CELL_INSET_X: f32 = METRICS.gap_m;
pub(super) const TABLE_CELL_INSET_Y: f32 = METRICS.gap_s;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TABLE_ACTION_WIDTH:
    f32 = 24.0;
pub(super) const TABLE_COLUMN_RATIOS: [f32; 4] = [0.36, 0.27, 0.19, 0.18];
pub(super) const TABLE_COLUMN_MIN_WIDTHS: [f32; 4] = [120.0, 56.0, 56.0, 72.0];
pub(super) const TABLE_COLUMN_DROP_ORDER: [usize; 4] = [3, 2, 1, 0];
