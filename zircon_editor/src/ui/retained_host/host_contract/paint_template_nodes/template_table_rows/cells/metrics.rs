use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

pub(super) const TABLE_COLUMN_COUNT: usize = 4;
const TABLE_COLUMN_RATIOS: [f32; TABLE_COLUMN_COUNT] = [0.36, 0.27, 0.19, 0.18];
const TABLE_COLUMN_DROP_ORDER: [usize; TABLE_COLUMN_COUNT] = [3, 2, 1, 0];
const NAME_COLUMN_WIDTH_SAMPLE: &str = "DefaultAssetName";
const TYPE_COLUMN_WIDTH_SAMPLE: &str = "Material";
const SIZE_COLUMN_WIDTH_SAMPLE: &str = "999 MB";
const REVISION_COLUMN_WIDTH_SAMPLE: &str = "Revision 999";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchTableCellMetrics {
    pub font_size: f32,
    pub line_height: f32,
    pub inset_x: f32,
    pub inset_y: f32,
    pub text_clip_guard: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchTableColumnMetrics {
    pub ratios: [f32; TABLE_COLUMN_COUNT],
    pub min_widths: [f32; TABLE_COLUMN_COUNT],
    pub drop_order: [usize; TABLE_COLUMN_COUNT],
}

pub(super) fn table_cell_metrics() -> WorkbenchTableCellMetrics {
    table_cell_metrics_from_host(current_host_metrics())
}

pub(super) fn table_column_metrics() -> WorkbenchTableColumnMetrics {
    table_column_metrics_from_host(current_host_metrics())
}

fn table_cell_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchTableCellMetrics {
    WorkbenchTableCellMetrics {
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        inset_x: metrics.gap_m,
        inset_y: metrics.gap_s,
        text_clip_guard: metrics.text_clip_guard,
    }
}

fn table_column_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchTableColumnMetrics {
    WorkbenchTableColumnMetrics {
        ratios: TABLE_COLUMN_RATIOS,
        min_widths: [
            table_column_min_width(metrics, NAME_COLUMN_WIDTH_SAMPLE, metrics.row_height * 5.0),
            table_column_min_width(
                metrics,
                TYPE_COLUMN_WIDTH_SAMPLE,
                metrics.row_height * 2.0 + metrics.gap_m,
            ),
            table_column_min_width(
                metrics,
                SIZE_COLUMN_WIDTH_SAMPLE,
                metrics.row_height * 2.0 + metrics.gap_m,
            ),
            table_column_min_width(
                metrics,
                REVISION_COLUMN_WIDTH_SAMPLE,
                metrics.row_height * 3.0,
            ),
        ],
        drop_order: TABLE_COLUMN_DROP_ORDER,
    }
}

fn table_column_min_width(metrics: HostControlMetrics, sample_text: &str, floor: f32) -> f32 {
    let text_width = measure_runtime_text_width(sample_text, metrics.font_body);
    (text_width + metrics.gap_m * 2.0 + metrics.text_clip_guard).max(floor)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn table_cell_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.font_body = 11.0;
        host.line_height_ratio = 1.35;
        host.gap_m = 9.0;
        host.gap_s = 5.0;
        host.text_clip_guard = 7.0;

        let metrics = table_cell_metrics_from_host(host);

        assert_eq!(metrics.font_size, 11.0);
        assert!((metrics.line_height - 14.85).abs() < 0.0001);
        assert_eq!(metrics.inset_x, 9.0);
        assert_eq!(metrics.inset_y, 5.0);
        assert_eq!(metrics.text_clip_guard, 7.0);
    }

    #[test]
    fn table_column_metrics_project_readable_minimums_from_host_control_metrics() {
        let mut host = METRICS;
        host.font_body = 0.0;
        host.row_height = 30.0;
        host.gap_m = 10.0;
        host.text_clip_guard = 4.0;

        let metrics = table_column_metrics_from_host(host);

        assert_eq!(metrics.ratios, [0.36, 0.27, 0.19, 0.18]);
        assert_eq!(metrics.min_widths, [150.0, 70.0, 70.0, 90.0]);
        assert_eq!(metrics.drop_order, [3, 2, 1, 0]);
    }

    #[test]
    fn table_column_minimums_include_runtime_text_measurement() {
        let mut host = METRICS;
        host.font_body = 12.0;
        host.row_height = 1.0;
        host.gap_m = 3.0;
        host.text_clip_guard = 2.0;

        let metrics = table_column_metrics_from_host(host);
        let expected_revision_min =
            measure_runtime_text_width(REVISION_COLUMN_WIDTH_SAMPLE, 12.0) + 8.0;

        assert!(metrics.min_widths[3] >= expected_revision_min);
    }
}
