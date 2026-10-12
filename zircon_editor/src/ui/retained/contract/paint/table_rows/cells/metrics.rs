//! 列可读最小宽度按宿主字体度量与密度推导，固定比例只分配剩余空间；列内距与文字高度共同限定可绘区。

use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

pub(super) const TABLE_COLUMN_COUNT: usize = 4;
const TABLE_COLUMN_RATIOS: [f32; TABLE_COLUMN_COUNT] = [0.36, 0.27, 0.19, 0.18];
const TABLE_COLUMN_DROP_ORDER: [usize; TABLE_COLUMN_COUNT] = [3, 2, 1, 0];
// Column minimums guarantee readable headers. Variable row values may clip
// inside their slot; they must not force a low-priority column out globally.
const NAME_COLUMN_WIDTH_SAMPLE: &str = "Name";
const TYPE_COLUMN_WIDTH_SAMPLE: &str = "Type";
const SIZE_COLUMN_WIDTH_SAMPLE: &str = "Size";
const REVISION_COLUMN_WIDTH_SAMPLE: &str = "Revision";

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
            table_column_min_width(
                metrics,
                NAME_COLUMN_WIDTH_SAMPLE,
                metrics.row_height * 4.0 + metrics.gap_m,
            ),
            table_column_min_width(metrics, TYPE_COLUMN_WIDTH_SAMPLE, metrics.row_height * 2.0),
            table_column_min_width(metrics, SIZE_COLUMN_WIDTH_SAMPLE, metrics.row_height * 2.0),
            table_column_min_width(
                metrics,
                REVISION_COLUMN_WIDTH_SAMPLE,
                metrics.row_height * 2.0 + metrics.gap_l + metrics.gap_s,
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
#[path = "tests/metrics.rs"]
mod tests;
