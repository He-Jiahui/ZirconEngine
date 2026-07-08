use super::super::super::super::data::TemplatePaneNodeData;
use super::metrics::{table_column_metrics, WorkbenchTableColumnMetrics, TABLE_COLUMN_COUNT};

const TABLE_LAYOUT_NARROW_VARIANT: &str = "layoutNarrow";

#[derive(Clone, Copy)]
pub(super) enum TableColumnAlignment {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TableColumnLayoutTier {
    Regular,
    Narrow,
}

#[derive(Clone, Copy)]
pub(super) struct TableColumnLayout {
    widths: [f32; TABLE_COLUMN_COUNT],
}

pub(super) fn allocate_table_columns_for_node(
    node: &TemplatePaneNodeData,
    available_width: f32,
) -> TableColumnLayout {
    let metrics = table_column_metrics();
    allocate_table_columns(available_width, table_column_layout_tier(node), metrics)
}

fn allocate_table_columns(
    available_width: f32,
    layout_tier: TableColumnLayoutTier,
    metrics: WorkbenchTableColumnMetrics,
) -> TableColumnLayout {
    let available_width = available_width.max(1.0);
    let mut visible = visible_columns_for_layout_tier(layout_tier);
    drop_columns_until_minimums_fit(&mut visible, available_width, metrics);
    let mut widths = proportional_visible_widths(visible, available_width, metrics);
    clamp_visible_widths_to_minimums(&mut widths, visible, available_width, metrics);
    TableColumnLayout { widths }
}

pub(super) fn table_column_alignment(index: usize) -> TableColumnAlignment {
    match index {
        2 | 3 => TableColumnAlignment::Right,
        _ => TableColumnAlignment::Left,
    }
}

impl TableColumnLayout {
    pub(super) fn x_offset(self, index: usize) -> f32 {
        self.widths.iter().take(index).sum()
    }

    pub(super) fn width(self, index: usize) -> f32 {
        self.widths.get(index).copied().unwrap_or(0.0)
    }
}

fn visible_columns_for_layout_tier(
    layout_tier: TableColumnLayoutTier,
) -> [bool; TABLE_COLUMN_COUNT] {
    match layout_tier {
        TableColumnLayoutTier::Regular => [true; TABLE_COLUMN_COUNT],
        TableColumnLayoutTier::Narrow => [true, true, false, false],
    }
}

fn table_column_layout_tier(node: &TemplatePaneNodeData) -> TableColumnLayoutTier {
    if component_variant_has_token(node.component_variant.as_str(), TABLE_LAYOUT_NARROW_VARIANT) {
        TableColumnLayoutTier::Narrow
    } else {
        TableColumnLayoutTier::Regular
    }
}

fn component_variant_has_token(variant: &str, token: &str) -> bool {
    variant
        .split_whitespace()
        .any(|candidate| candidate == token)
}

fn drop_columns_until_minimums_fit(
    visible: &mut [bool; TABLE_COLUMN_COUNT],
    width: f32,
    metrics: WorkbenchTableColumnMetrics,
) {
    for index in metrics.drop_order {
        if visible_minimum_width(*visible, metrics) <= width || only_name_column_visible(*visible) {
            break;
        }
        visible[index] = false;
    }
}

fn proportional_visible_widths(
    visible: [bool; TABLE_COLUMN_COUNT],
    available_width: f32,
    metrics: WorkbenchTableColumnMetrics,
) -> [f32; TABLE_COLUMN_COUNT] {
    let ratio_sum = metrics
        .ratios
        .iter()
        .enumerate()
        .filter(|(index, _)| visible[*index])
        .map(|(_, ratio)| *ratio)
        .sum::<f32>()
        .max(f32::EPSILON);
    let mut widths = [0.0; TABLE_COLUMN_COUNT];
    for (index, ratio) in metrics.ratios.iter().enumerate() {
        if visible[index] {
            widths[index] = available_width * (*ratio / ratio_sum);
        }
    }
    widths
}

fn clamp_visible_widths_to_minimums(
    widths: &mut [f32; TABLE_COLUMN_COUNT],
    visible: [bool; TABLE_COLUMN_COUNT],
    available_width: f32,
    metrics: WorkbenchTableColumnMetrics,
) {
    for (index, width) in widths.iter_mut().enumerate() {
        if visible[index] {
            *width = (*width).max(metrics.min_widths[index]);
        }
    }
    reclaim_overflow_from_flexible_columns(widths, visible, available_width, metrics);
}

fn reclaim_overflow_from_flexible_columns(
    widths: &mut [f32; TABLE_COLUMN_COUNT],
    visible: [bool; TABLE_COLUMN_COUNT],
    available_width: f32,
    metrics: WorkbenchTableColumnMetrics,
) {
    let overflow = widths.iter().sum::<f32>() - available_width;
    if overflow <= 0.0 {
        return;
    }
    let flexible_width = widths
        .iter()
        .enumerate()
        .filter(|(index, width)| visible[*index] && **width > metrics.min_widths[*index])
        .map(|(index, width)| *width - metrics.min_widths[index])
        .sum::<f32>();
    if flexible_width <= f32::EPSILON {
        return;
    }
    for (index, width) in widths.iter_mut().enumerate() {
        if !visible[index] || *width <= metrics.min_widths[index] {
            continue;
        }
        let excess = *width - metrics.min_widths[index];
        let reduction = overflow * (excess / flexible_width);
        *width = (*width - reduction).max(metrics.min_widths[index]);
    }
}

fn visible_minimum_width(
    visible: [bool; TABLE_COLUMN_COUNT],
    metrics: WorkbenchTableColumnMetrics,
) -> f32 {
    metrics
        .min_widths
        .iter()
        .enumerate()
        .filter(|(index, _)| visible[*index])
        .map(|(_, width)| *width)
        .sum()
}

fn only_name_column_visible(visible: [bool; TABLE_COLUMN_COUNT]) -> bool {
    visible[0] && !visible[1] && !visible[2] && !visible[3]
}
