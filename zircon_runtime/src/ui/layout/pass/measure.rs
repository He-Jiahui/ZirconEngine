use crate::ui::{surface::measure_text, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        DesiredSize, UiAxis, UiContainerKind, UiGridBoxConfig, UiGridSlotPlacement, UiMargin,
        UiSize,
    },
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeError},
};

use super::slot::{ordered_children_for_container, slot_for_container_child, slot_padding};
use super::{axis::desired_axis, material::measure_material_content};

const UNBOUNDED_WRAP_MEASURE_WIDTH: f32 = f32::INFINITY;

const BUTTON_HORIZONTAL_PADDING: f32 = 18.0;
const BUTTON_VERTICAL_PADDING: f32 = 8.0;

pub(crate) fn measure_node(
    tree: &mut UiTree,
    node_id: UiNodeId,
) -> Result<DesiredSize, UiTreeError> {
    let (children, layout_boundary, constraints, container, template_metadata) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.effective_visibility().occupies_layout() {
            return collapse_node_measure(tree, node_id);
        }
        (
            node.children.clone(),
            node.layout_boundary,
            node.constraints,
            node.container,
            node.template_metadata.clone(),
        )
    };

    let mut child_desired = Vec::with_capacity(children.len());
    for child_id in &children {
        let desired = measure_node(tree, *child_id)?;
        if tree
            .node(*child_id)
            .is_some_and(|child| child.effective_visibility().occupies_layout())
        {
            child_desired.push((*child_id, desired));
        }
    }

    let content_size = measure_content_size(
        tree,
        node_id,
        container,
        &child_desired,
        template_metadata.as_ref(),
    );
    let desired = DesiredSize::new(
        desired_axis(layout_boundary, constraints.width, content_size.width),
        desired_axis(layout_boundary, constraints.height, content_size.height),
    );

    let node = tree
        .node_mut(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    node.layout_cache.desired_size = desired;
    node.layout_cache.content_size = content_size;
    if !node.container.is_scrollable() {
        node.layout_cache.virtual_window = None;
    }

    Ok(desired)
}

fn collapse_node_measure(tree: &mut UiTree, node_id: UiNodeId) -> Result<DesiredSize, UiTreeError> {
    let children = {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.layout_cache.desired_size = DesiredSize::default();
        node.layout_cache.content_size = UiSize::default();
        node.layout_cache.virtual_window = None;
        node.children.clone()
    };
    for child_id in children {
        let _ = collapse_node_measure(tree, child_id)?;
    }
    Ok(DesiredSize::default())
}

fn measure_content_size(
    tree: &UiTree,
    node_id: UiNodeId,
    container: UiContainerKind,
    child_desired: &[(UiNodeId, DesiredSize)],
    metadata: Option<&UiTemplateNodeMetadata>,
) -> UiSize {
    if child_desired.is_empty() {
        return measure_leaf_content_size(metadata);
    }

    let content_size = match container {
        UiContainerKind::Free | UiContainerKind::Container | UiContainerKind::Overlay => {
            measure_stacked_content_size(tree, node_id, container, child_desired)
        }
        UiContainerKind::Space => UiSize::default(),
        UiContainerKind::HorizontalBox(config) => measure_linear_content_size(
            tree,
            node_id,
            container,
            UiAxis::Horizontal,
            config.gap,
            child_desired,
        ),
        UiContainerKind::VerticalBox(config) => measure_linear_content_size(
            tree,
            node_id,
            container,
            UiAxis::Vertical,
            config.gap,
            child_desired,
        ),
        UiContainerKind::ScrollableBox(config) => {
            measure_plain_linear_content_size(config.axis, config.gap, child_desired)
        }
        UiContainerKind::WrapBox(config) => {
            measure_wrap_content_size(tree, node_id, container, config, child_desired)
        }
        UiContainerKind::GridBox(config) => {
            measure_grid_content_size(tree, node_id, config, child_desired)
        }
    };

    measure_material_content(metadata, content_size).unwrap_or(content_size)
}

fn measure_leaf_content_size(metadata: Option<&UiTemplateNodeMetadata>) -> UiSize {
    let text_size = measure_text(metadata);
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

fn measure_stacked_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> UiSize {
    UiSize::new(
        child_desired
            .iter()
            .map(|(child_id, size)| {
                size.width + slot_padding_for(tree, parent_id, *child_id, container).horizontal()
            })
            .fold(0.0_f32, f32::max),
        child_desired
            .iter()
            .map(|(child_id, size)| {
                size.height + slot_padding_for(tree, parent_id, *child_id, container).vertical()
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
) -> UiSize {
    let ordered_children = ordered_children_for_container(
        tree,
        parent_id,
        &child_desired
            .iter()
            .map(|(child_id, _)| *child_id)
            .collect::<Vec<_>>(),
        container,
    );
    let ordered_desired: Vec<_> = ordered_children
        .iter()
        .filter_map(|ordered_child_id| {
            child_desired
                .iter()
                .find(|(child_id, _)| child_id == ordered_child_id)
                .copied()
        })
        .collect();
    measure_linear_content_size_with_padding(
        tree,
        parent_id,
        container,
        axis,
        gap,
        &ordered_desired,
    )
}

fn measure_linear_content_size_with_padding(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
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
                .map(|(child_id, size)| {
                    size.width
                        + slot_padding_for(tree, parent_id, *child_id, container).horizontal()
                })
                .fold(0.0_f32, f32::max),
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.height + slot_padding_for(tree, parent_id, *child_id, container).vertical()
                })
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
        ),
        UiAxis::Horizontal => UiSize::new(
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.width
                        + slot_padding_for(tree, parent_id, *child_id, container).horizontal()
                })
                .sum::<f32>()
                + gap * (count - 1.0).max(0.0),
            child_desired
                .iter()
                .map(|(child_id, size)| {
                    size.height + slot_padding_for(tree, parent_id, *child_id, container).vertical()
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
) -> UiSize {
    let ordered_desired =
        ordered_child_desired_for_container(tree, parent_id, container, child_desired);
    measure_wrap_content_size_for_width(
        tree,
        parent_id,
        container,
        config,
        &ordered_desired,
        wrap_measure_width(tree, parent_id),
    )
}

fn measure_grid_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    config: UiGridBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> UiSize {
    let container = UiContainerKind::GridBox(config);
    let ordered_desired =
        ordered_child_desired_for_container(tree, parent_id, container, child_desired);
    let (columns, rows) = grid_dimensions_for_desired(tree, parent_id, config, &ordered_desired);
    let mut column_widths = vec![0.0_f32; columns];
    let mut row_heights = vec![0.0_f32; rows];

    for (index, (child_id, desired)) in ordered_desired.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, parent_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        let column = placement.column.min(columns - 1);
        let row = placement.row.min(rows - 1);
        let column_span = placement.column_span.max(1).min(columns - column);
        let row_span = placement.row_span.max(1).min(rows - row);
        let padding = slot_padding(slot);
        let width_per_column = (desired.width + padding.horizontal()) / column_span as f32;
        let height_per_row = (desired.height + padding.vertical()) / row_span as f32;

        for width in column_widths.iter_mut().skip(column).take(column_span) {
            *width = width.max(width_per_column);
        }
        for height in row_heights.iter_mut().skip(row).take(row_span) {
            *height = height.max(height_per_row);
        }
    }

    UiSize::new(
        column_widths.iter().sum::<f32>()
            + config.column_gap.max(0.0) * columns.saturating_sub(1) as f32,
        row_heights.iter().sum::<f32>() + config.row_gap.max(0.0) * rows.saturating_sub(1) as f32,
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
        let padding = slot_padding_for(tree, parent_id, *child_id, container);
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

fn ordered_child_desired_for_container(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> Vec<(UiNodeId, DesiredSize)> {
    let ordered_children = ordered_children_for_container(
        tree,
        parent_id,
        &child_desired
            .iter()
            .map(|(child_id, _)| *child_id)
            .collect::<Vec<_>>(),
        container,
    );

    ordered_children
        .iter()
        .filter_map(|ordered_child_id| {
            child_desired
                .iter()
                .find(|(child_id, _)| child_id == ordered_child_id)
                .copied()
        })
        .collect()
}

fn grid_dimensions_for_desired(
    tree: &UiTree,
    parent_id: UiNodeId,
    config: UiGridBoxConfig,
    child_desired: &[(UiNodeId, DesiredSize)],
) -> (usize, usize) {
    let mut columns = config.columns.max(1);
    let mut rows = config.rows.max(1);
    let container = UiContainerKind::GridBox(config);
    for (index, (child_id, _)) in child_desired.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, parent_id, child_id, container);
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

fn slot_padding_for(
    tree: &UiTree,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> UiMargin {
    slot_padding(slot_for_container_child(
        tree, parent_id, child_id, container,
    ))
}
