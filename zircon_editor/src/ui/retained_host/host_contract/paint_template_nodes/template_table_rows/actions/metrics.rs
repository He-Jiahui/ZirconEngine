//! 单元格先预留此操作列宽，按钮绘制再消费同一密度快照；改尺寸必须保持两处预留一致。

use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchTableActionMetrics {
    pub action_column_width: f32,
    pub button_size: f32,
    pub icon_size: f32,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn table_action_metrics() -> WorkbenchTableActionMetrics {
    table_action_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_table_rows) fn table_action_column_width(
) -> f32 {
    table_action_metrics().action_column_width
}

fn table_action_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchTableActionMetrics {
    let icon_size = metrics.gap_m * 2.0;
    let button_size = icon_size + metrics.gap_s;
    WorkbenchTableActionMetrics {
        action_column_width: button_size + metrics.gap_s,
        button_size,
        icon_size,
        border_width: metrics.border_width,
        radius: metrics.radius_control,
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
