use super::super::super::super::data::TemplatePaneNodeData;
use super::metrics::{TABLE_COLUMN_DROP_ORDER, TABLE_COLUMN_MIN_WIDTHS, TABLE_COLUMN_RATIOS};

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
    widths: [f32; TABLE_COLUMN_RATIOS.len()],
}

pub(super) fn allocate_table_columns_for_node(
    node: &TemplatePaneNodeData,
    available_width: f32,
) -> TableColumnLayout {
    allocate_table_columns(available_width, table_column_layout_tier(node))
}

fn allocate_table_columns(
    available_width: f32,
    layout_tier: TableColumnLayoutTier,
) -> TableColumnLayout {
    let available_width = available_width.max(1.0);
    let mut visible = visible_columns_for_layout_tier(layout_tier);
    drop_columns_until_minimums_fit(&mut visible, available_width);
    let mut widths = proportional_visible_widths(visible, available_width);
    clamp_visible_widths_to_minimums(&mut widths, visible, available_width);
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
) -> [bool; TABLE_COLUMN_RATIOS.len()] {
    match layout_tier {
        TableColumnLayoutTier::Regular => [true; TABLE_COLUMN_RATIOS.len()],
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

fn drop_columns_until_minimums_fit(visible: &mut [bool; TABLE_COLUMN_RATIOS.len()], width: f32) {
    for index in TABLE_COLUMN_DROP_ORDER {
        if visible_minimum_width(*visible) <= width || only_name_column_visible(*visible) {
            break;
        }
        visible[index] = false;
    }
}

fn proportional_visible_widths(
    visible: [bool; TABLE_COLUMN_RATIOS.len()],
    available_width: f32,
) -> [f32; TABLE_COLUMN_RATIOS.len()] {
    let ratio_sum = TABLE_COLUMN_RATIOS
        .iter()
        .enumerate()
        .filter(|(index, _)| visible[*index])
        .map(|(_, ratio)| *ratio)
        .sum::<f32>()
        .max(f32::EPSILON);
    let mut widths = [0.0; TABLE_COLUMN_RATIOS.len()];
    for (index, ratio) in TABLE_COLUMN_RATIOS.iter().enumerate() {
        if visible[index] {
            widths[index] = available_width * (*ratio / ratio_sum);
        }
    }
    widths
}

fn clamp_visible_widths_to_minimums(
    widths: &mut [f32; TABLE_COLUMN_RATIOS.len()],
    visible: [bool; TABLE_COLUMN_RATIOS.len()],
    available_width: f32,
) {
    for (index, width) in widths.iter_mut().enumerate() {
        if visible[index] {
            *width = (*width).max(TABLE_COLUMN_MIN_WIDTHS[index]);
        }
    }
    reclaim_overflow_from_flexible_columns(widths, visible, available_width);
}

fn reclaim_overflow_from_flexible_columns(
    widths: &mut [f32; TABLE_COLUMN_RATIOS.len()],
    visible: [bool; TABLE_COLUMN_RATIOS.len()],
    available_width: f32,
) {
    let overflow = widths.iter().sum::<f32>() - available_width;
    if overflow <= 0.0 {
        return;
    }
    let flexible_width = widths
        .iter()
        .enumerate()
        .filter(|(index, width)| visible[*index] && **width > TABLE_COLUMN_MIN_WIDTHS[*index])
        .map(|(index, width)| *width - TABLE_COLUMN_MIN_WIDTHS[index])
        .sum::<f32>();
    if flexible_width <= f32::EPSILON {
        return;
    }
    for (index, width) in widths.iter_mut().enumerate() {
        if !visible[index] || *width <= TABLE_COLUMN_MIN_WIDTHS[index] {
            continue;
        }
        let excess = *width - TABLE_COLUMN_MIN_WIDTHS[index];
        let reduction = overflow * (excess / flexible_width);
        *width = (*width - reduction).max(TABLE_COLUMN_MIN_WIDTHS[index]);
    }
}

fn visible_minimum_width(visible: [bool; TABLE_COLUMN_RATIOS.len()]) -> f32 {
    TABLE_COLUMN_MIN_WIDTHS
        .iter()
        .enumerate()
        .filter(|(index, _)| visible[*index])
        .map(|(_, width)| *width)
        .sum()
}

fn only_name_column_visible(visible: [bool; TABLE_COLUMN_RATIOS.len()]) -> bool {
    visible[0] && !visible[1] && !visible[2] && !visible[3]
}
