use crate::ui::{
    surface::{
        measure_text_with_cache, measure_text_with_fixed_width_cache, metadata_has_inline_widget,
    },
    text::UiTextMeasureCache,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        BoxConstraints, DesiredSize, StretchMode, UiAxis, UiContainerKind, UiGridBoxConfig,
        UiGridSlotPlacement, UiMargin, UiMasonryBoxConfig, UiSize,
    },
    tree::{UiTemplateNodeMetadata, UiTree},
};

use super::slot::{slot_for_container_child, slot_padding, UiLayoutSlotIndex};
use super::workspace::UiContainerMeasureScratch;
use super::{axis::desired_axis, material::measure_material_content};

mod traversal;

pub(crate) use traversal::{measure_node, measure_node_incremental};

const UNBOUNDED_WRAP_MEASURE_WIDTH: f32 = f32::INFINITY;

const BUTTON_HORIZONTAL_PADDING: f32 = 18.0;
const BUTTON_VERTICAL_PADDING: f32 = 8.0;

fn measure_content_size(
    tree: &UiTree,
    node_id: UiNodeId,
    container: UiContainerKind,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: &mut UiTextMeasureCache,
    container_scratch: &mut UiContainerMeasureScratch,
) -> UiSize {
    if !child_desired.is_empty() && metadata_has_inline_widget(metadata) {
        return measure_leaf_content_size(
            metadata,
            tree.node(node_id).map(|node| node.constraints),
            text_measure_cache,
        );
    }
    if child_desired.is_empty() {
        let constraints = tree.node(node_id).map(|node| node.constraints);
        return measure_leaf_content_size(metadata, constraints, text_measure_cache);
    }

    let content_size = match container {
        UiContainerKind::Free
        | UiContainerKind::Canvas
        | UiContainerKind::Container
        | UiContainerKind::Overlay => {
            measure_stacked_content_size(tree, node_id, container, child_desired, slot_index)
        }
        UiContainerKind::BlockBox => measure_linear_content_size(
            tree,
            node_id,
            container,
            UiAxis::Vertical,
            0.0,
            child_desired,
            slot_index,
        ),
        UiContainerKind::Space => UiSize::default(),
        UiContainerKind::SizeBox(config) => measure_size_box_content_size(
            measure_stacked_content_size(tree, node_id, container, child_desired, slot_index),
            config.aspect_ratio,
        ),
        UiContainerKind::HorizontalBox(config) => measure_linear_content_size(
            tree,
            node_id,
            container,
            UiAxis::Horizontal,
            config.gap,
            child_desired,
            slot_index,
        ),
        UiContainerKind::VerticalBox(config) => measure_linear_content_size(
            tree,
            node_id,
            container,
            UiAxis::Vertical,
            config.gap,
            child_desired,
            slot_index,
        ),
        UiContainerKind::ScrollableBox(config) => {
            measure_plain_linear_content_size(config.axis, config.gap, child_desired)
        }
        UiContainerKind::WrapBox(config) => {
            measure_wrap_content_size(tree, node_id, container, config, child_desired, slot_index)
        }
        UiContainerKind::GridBox(config) => measure_grid_content_size(
            tree,
            node_id,
            config,
            child_desired,
            slot_index,
            container_scratch,
        ),
        UiContainerKind::MasonryBox(config) => measure_masonry_content_size(
            tree,
            node_id,
            config,
            child_desired,
            slot_index,
            container_scratch,
        ),
    };

    measure_material_content(metadata, content_size).unwrap_or(content_size)
}

fn measure_size_box_content_size(content_size: UiSize, aspect_ratio: f32) -> UiSize {
    let Some(aspect_ratio) = normalized_aspect_ratio(aspect_ratio) else {
        return content_size;
    };

    if content_size.width <= 0.0 && content_size.height <= 0.0 {
        return content_size;
    }
    if content_size.width <= 0.0 {
        return UiSize::new(content_size.height * aspect_ratio, content_size.height);
    }
    if content_size.height <= 0.0 {
        return UiSize::new(content_size.width, content_size.width / aspect_ratio);
    }

    let current_ratio = content_size.width / content_size.height;
    if current_ratio > aspect_ratio {
        UiSize::new(content_size.width, content_size.width / aspect_ratio)
    } else {
        UiSize::new(content_size.height * aspect_ratio, content_size.height)
    }
}

fn normalized_aspect_ratio(aspect_ratio: f32) -> Option<f32> {
    aspect_ratio
        .is_finite()
        .then_some(aspect_ratio)
        .filter(|ratio| *ratio > 0.0)
}

fn measure_leaf_content_size(
    metadata: Option<&UiTemplateNodeMetadata>,
    constraints: Option<BoxConstraints>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiSize {
    let text_size = if let Some(width) = constraints.and_then(exact_fixed_width) {
        measure_text_with_fixed_width_cache(metadata, text_measure_cache, width)
    } else {
        measure_text_with_cache(metadata, text_measure_cache)
    };
    let Some(metadata) = metadata else {
        return text_size;
    };

    if let Some(material_size) = measure_material_content(Some(metadata), text_size) {
        return material_size;
    }

    match metadata.component.as_str() {
        "Button" | "IconButton" if text_size.width > 0.0 || text_size.height > 0.0 => UiSize::new(
            text_size.width + BUTTON_HORIZONTAL_PADDING,
            text_size.height + BUTTON_VERTICAL_PADDING,
        ),
        _ => text_size,
    }
}

fn exact_fixed_width(constraints: BoxConstraints) -> Option<f32> {
    let width = constraints.width.resolved();
    (matches!(constraints.width.stretch_mode, StretchMode::Fixed)
        && width.max.is_some_and(|maximum| maximum == width.min)
        && width.min.is_finite()
        && width.min > 0.0)
        .then_some(width.min)
}

fn measure_stacked_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
) -> UiSize {
    UiSize::new(
        child_desired
            .iter()
            .map(|(child_id, size)| {
                size.width
                    + slot_padding_for(tree, slot_index, parent_id, *child_id, container)
                        .horizontal()
            })
            .fold(0.0_f32, f32::max),
        child_desired
            .iter()
            .map(|(child_id, size)| {
                size.height
                    + slot_padding_for(tree, slot_index, parent_id, *child_id, container).vertical()
            })
            .fold(0.0_f32, f32::max),
    )
}

fn measure_linear_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    axis: UiAxis,
    gap: f32,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
) -> UiSize {
    measure_linear_content_size_with_padding(
        tree,
        parent_id,
        container,
        axis,
        gap,
        child_desired,
        slot_index,
    )
}

fn measure_linear_content_size_with_padding(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    axis: UiAxis,
    gap: f32,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
) -> UiSize {
    let gap = gap.max(0.0);
    let count = child_desired.len() as f32;
    match axis {
        UiAxis::Vertical => UiSize::new(
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.width
                        + slot_padding_for(tree, slot_index, parent_id, *child_id, container)
                            .horizontal()
                })
                .fold(0.0_f32, f32::max),
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.height
                        + slot_padding_for(tree, slot_index, parent_id, *child_id, container)
                            .vertical()
                })
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
        ),
        UiAxis::Horizontal => UiSize::new(
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.width
                        + slot_padding_for(tree, slot_index, parent_id, *child_id, container)
                            .horizontal()
                })
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.height
                        + slot_padding_for(tree, slot_index, parent_id, *child_id, container)
                            .vertical()
                })
                .fold(0.0_f32, f32::max),
        ),
    }
}

fn measure_plain_linear_content_size(
    axis: UiAxis,
    gap: f32,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> UiSize {
    let gap = gap.max(0.0);
    let count = child_desired.len() as f32;
    match axis {
        UiAxis::Vertical => UiSize::new(
            child_desired
                .iter()
                .map(|(_, size)| size.width)
                .fold(0.0_f32, f32::max),
            child_desired
                .iter()
                .map(|(_, size)| size.height)
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
        ),
        UiAxis::Horizontal => UiSize::new(
            child_desired
                .iter()
                .map(|(_, size)| size.width)
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
            child_desired
                .iter()
                .map(|(_, size)| size.height)
                .fold(0.0_f32, f32::max),
        ),
    }
}

fn measure_wrap_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    config: zircon_runtime_interface::ui::layout::UiWrapBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
) -> UiSize {
    measure_wrap_content_size_for_width(
        tree,
        parent_id,
        container,
        config,
        child_desired,
        wrap_measure_width(tree, parent_id),
        slot_index,
    )
}

fn measure_grid_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    config: UiGridBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
    container_scratch: &mut UiContainerMeasureScratch,
) -> UiSize {
    let container = UiContainerKind::GridBox(config);
    let (columns, rows) =
        grid_dimensions_for_desired(tree, slot_index, parent_id, config, child_desired);
    container_scratch.primary_extents.clear();
    container_scratch.primary_extents.resize(columns, 0.0);
    container_scratch.secondary_extents.clear();
    container_scratch.secondary_extents.resize(rows, 0.0);

    for (index, (child_id, desired)) in child_desired.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        let column = placement.column.min(columns - 1);
        let row = placement.row.min(rows - 1);
        let column_span = placement.column_span.max(1).min(columns - column);
        let row_span = placement.row_span.max(1).min(rows - row);
        let padding = slot_padding(slot);
        let width_per_column = (desired.width + padding.horizontal()) / column_span as f32;
        let height_per_row = (desired.height + padding.vertical()) / row_span as f32;

        for width in container_scratch
            .primary_extents
            .iter_mut()
            .skip(column)
            .take(column_span)
        {
            *width = width.max(width_per_column);
        }
        for height in container_scratch
            .secondary_extents
            .iter_mut()
            .skip(row)
            .take(row_span)
        {
            *height = height.max(height_per_row);
        }
    }

    UiSize::new(
        container_scratch.primary_extents.iter().sum::<f32>()
            + config.column_gap.max(0.0) * columns.saturating_sub(1) as f32,
        container_scratch.secondary_extents.iter().sum::<f32>()
            + config.row_gap.max(0.0) * rows.saturating_sub(1) as f32,
    )
}

fn measure_masonry_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    config: UiMasonryBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
    slot_index: &UiLayoutSlotIndex,
    container_scratch: &mut UiContainerMeasureScratch,
) -> UiSize {
    let container = UiContainerKind::MasonryBox(config);
    let columns = config.columns.max(1);
    let gap = config.gap.max(0.0);
    container_scratch.primary_extents.clear();
    container_scratch.primary_extents.resize(columns, 0.0);
    container_scratch.secondary_extents.clear();
    container_scratch.secondary_extents.resize(columns, 0.0);
    container_scratch.column_counts.clear();
    container_scratch.column_counts.resize(columns, 0);

    for (index, (child_id, desired)) in child_desired.iter().copied().enumerate() {
        let padding = slot_padding_for(tree, slot_index, parent_id, child_id, container);
        let column = masonry_target_column(
            index,
            config.sequential,
            &container_scratch.secondary_extents,
        );
        if container_scratch.column_counts[column] > 0 {
            container_scratch.secondary_extents[column] += gap;
        }
        container_scratch.primary_extents[column] =
            container_scratch.primary_extents[column].max(desired.width + padding.horizontal());
        container_scratch.secondary_extents[column] += desired.height + padding.vertical();
        container_scratch.column_counts[column] += 1;
    }

    let used_columns = container_scratch
        .column_counts
        .iter()
        .filter(|count| **count > 0)
        .count();
    UiSize::new(
        container_scratch.primary_extents.iter().sum::<f32>()
            + gap * used_columns.saturating_sub(1) as f32,
        container_scratch
            .secondary_extents
            .iter()
            .copied()
            .fold(0.0_f32, f32::max),
    )
}

fn wrap_measure_width(tree: &UiTree, parent_id: UiNodeId) -> f32 {
    let Some(node) = tree.node(parent_id) else {
        return UNBOUNDED_WRAP_MEASURE_WIDTH;
    };
    let resolved = node.constraints.width.resolved();
    if node.constraints.width.stretch_mode
        == zircon_runtime_interface::ui::layout::StretchMode::Fixed
        && resolved.preferred > 0.0
    {
        return resolved.preferred;
    }
    resolved.max.unwrap_or_else(|| {
        if resolved.preferred > 0.0 {
            resolved.preferred
        } else {
            UNBOUNDED_WRAP_MEASURE_WIDTH
        }
    })
}

pub(super) fn measure_wrap_content_size_for_width(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    config: zircon_runtime_interface::ui::layout::UiWrapBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
    available_width: f32,
    slot_index: &UiLayoutSlotIndex,
) -> UiSize {
    let horizontal_gap = config.horizontal_gap.max(0.0);
    let vertical_gap = config.vertical_gap.max(0.0);
    let available_width = if available_width.is_finite() {
        available_width.max(0.0)
    } else {
        available_width
    };

    let mut content_width = 0.0_f32;
    let mut content_height = 0.0_f32;
    let mut cursor_x = 0.0_f32;
    let mut row_height = 0.0_f32;
    let mut row_count = 0usize;
    let mut row_item_count = 0usize;

    for (child_id, desired) in child_desired {
        let padding = slot_padding_for(tree, slot_index, parent_id, *child_id, container);
        let item_width = desired.width.max(config.item_min_width) + padding.horizontal();
        let item_height = desired.height + padding.vertical();
        let next_width = if row_item_count == 0 {
            item_width
        } else {
            cursor_x + horizontal_gap + item_width
        };
        if row_item_count > 0 && next_width > available_width {
            content_width = content_width.max(cursor_x);
            content_height += row_height;
            row_count += 1;
            cursor_x = 0.0;
            row_height = 0.0;
            row_item_count = 0;
        }

        if row_item_count > 0 {
            cursor_x += horizontal_gap;
        }
        cursor_x += item_width;
        row_height = row_height.max(item_height);
        row_item_count += 1;
    }

    if row_item_count > 0 {
        content_width = content_width.max(cursor_x);
        content_height += row_height;
        row_count += 1;
    }

    if row_count > 1 {
        content_height += vertical_gap * row_count.saturating_sub(1) as f32;
    }

    UiSize::new(content_width, content_height)
}

fn grid_dimensions_for_desired(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    config: UiGridBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> (usize, usize) {
    let mut columns = config.columns.max(1);
    let mut rows = config.rows.max(1);
    let container = UiContainerKind::GridBox(config);
    for (index, (child_id, _)) in child_desired.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        columns = columns.max(placement.column + placement.column_span.max(1));
        rows = rows.max(placement.row + placement.row_span.max(1));
    }
    (columns, rows)
}

fn grid_placement_for_child(
    slot: Option<&zircon_runtime_interface::ui::layout::UiSlot>,
    index: usize,
    columns: usize,
) -> UiGridSlotPlacement {
    if let Some(placement) = slot.and_then(|slot| slot.grid_placement) {
        return placement.with_span(placement.column_span, placement.row_span);
    }

    let columns = columns.max(1);
    UiGridSlotPlacement::new(index % columns, index / columns)
}

fn masonry_target_column(index: usize, sequential: bool, column_heights: &[f32]) -> usize {
    if sequential {
        return index % column_heights.len().max(1);
    }

    column_heights
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(column, _)| column)
        .unwrap_or(0)
}

fn slot_padding_for(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> UiMargin {
    slot_padding(slot_for_container_child(
        tree, slot_index, parent_id, child_id, container,
    ))
}

#[cfg(test)]
mod tests;
