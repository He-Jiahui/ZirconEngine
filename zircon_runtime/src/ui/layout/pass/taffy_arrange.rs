use crate::ui::{layout::taffy_style_for_container, tree::UiRuntimeTreeAccessExt};
use taffy::geometry::Rect;
use taffy::prelude::{
    fr, line, AlignContent, AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap,
    GridPlacement, LengthPercentageAuto, Line, Size as TaffySize, Style, TaffyTree,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        AxisConstraint, BoxConstraints, Pivot, Position, StretchMode, UiAlignment, UiAxis,
        UiContainerKind, UiFrame, UiGridBoxConfig, UiGridSlotPlacement,
        UiLayoutEngineFallbackReason, UiLayoutEngineTaffyTreeBuildStats, UiLinearSlotSizeRule,
        UiLinearSlotSizing, UiMargin, UiSize, UiSlot,
    },
    tree::{UiTree, UiTreeError, UiTreeNode},
};

use super::arrange::arrange_node;
use super::engine::UiLayoutPassEngineContext;
use super::slot::{ordered_children_for_container, slot_for_container_child};

pub(super) fn try_arrange_taffy_owned_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<bool, UiTreeError> {
    let container = tree
        .node(parent_id)
        .ok_or(UiTreeError::MissingNode(parent_id))?
        .container;
    let Some(axis) = taffy_main_axis(container) else {
        return Ok(false);
    };
    if !taffy_supports_parent_layout_values(container, frame) {
        engine_context.record_taffy_fallback(
            parent_id,
            container,
            UiLayoutEngineFallbackReason::InvalidLayoutValue,
            None,
        );
        return Ok(false);
    }
    if let Some(reason) = taffy_child_contracts_unsupported(tree, parent_id, children, container)? {
        engine_context.record_taffy_fallback(parent_id, container, reason, None);
        return Ok(false);
    }

    let ordered_children = ordered_children_for_container(tree, parent_id, children, container);
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut taffy_children = Vec::with_capacity(ordered_children.len());
    for child_id in &ordered_children {
        let child = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        let slot = slot_for_container_child(tree, parent_id, *child_id, container);
        let Ok(taffy_child) = taffy.new_leaf(taffy_child_style(child, axis, container, slot))
        else {
            engine_context.record_taffy_fallback(
                parent_id,
                container,
                UiLayoutEngineFallbackReason::TaffyTreeBuildFailed,
                Some(taffy_tree_stats(taffy_children.len())),
            );
            return Ok(false);
        };
        taffy_children.push(taffy_child);
    }

    let Some(parent_style) = taffy_parent_style(tree, parent_id, container, frame) else {
        engine_context.record_taffy_fallback(
            parent_id,
            container,
            UiLayoutEngineFallbackReason::TaffyStyleUnavailable,
            Some(taffy_tree_stats(taffy_children.len())),
        );
        return Ok(false);
    };
    let Ok(taffy_parent) = taffy.new_with_children(parent_style, &taffy_children) else {
        engine_context.record_taffy_fallback(
            parent_id,
            container,
            UiLayoutEngineFallbackReason::TaffyTreeBuildFailed,
            Some(taffy_tree_stats(taffy_children.len())),
        );
        return Ok(false);
    };
    let complete_taffy_tree_build = complete_taffy_tree_stats(taffy_children.len());
    if taffy
        .compute_layout(
            taffy_parent,
            TaffySize {
                width: AvailableSpace::Definite(frame.width.max(0.0)),
                height: AvailableSpace::Definite(frame.height.max(0.0)),
            },
        )
        .is_err()
    {
        engine_context.record_taffy_fallback(
            parent_id,
            container,
            UiLayoutEngineFallbackReason::TaffyComputeFailed,
            Some(complete_taffy_tree_build),
        );
        return Ok(false);
    }

    let mut child_frames = Vec::with_capacity(ordered_children.len());
    for (child_id, taffy_child) in ordered_children.iter().copied().zip(taffy_children) {
        let Ok(layout) = taffy.layout(taffy_child) else {
            engine_context.record_taffy_fallback(
                parent_id,
                container,
                UiLayoutEngineFallbackReason::TaffyComputeFailed,
                Some(complete_taffy_tree_build),
            );
            return Ok(false);
        };
        child_frames.push((
            child_id,
            UiFrame::new(
                frame.x + layout.location.x,
                frame.y + layout.location.y,
                layout.size.width.max(0.0),
                layout.size.height.max(0.0),
            ),
        ));
    }

    engine_context.record_taffy_native(parent_id, container, complete_taffy_tree_build);
    for (child_id, child_frame) in child_frames {
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
    }

    Ok(true)
}

fn taffy_tree_stats(node_count: usize) -> UiLayoutEngineTaffyTreeBuildStats {
    UiLayoutEngineTaffyTreeBuildStats::new(u64::try_from(node_count).unwrap_or(u64::MAX))
}

fn complete_taffy_tree_stats(child_count: usize) -> UiLayoutEngineTaffyTreeBuildStats {
    taffy_tree_stats(child_count.saturating_add(1))
}

fn taffy_child_contracts_unsupported(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
) -> Result<Option<UiLayoutEngineFallbackReason>, UiTreeError> {
    for child_id in children {
        let child = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        if !child.effective_visibility().occupies_layout() {
            return Ok(Some(
                UiLayoutEngineFallbackReason::UnsupportedChildVisibility,
            ));
        }
        // Template metadata carries render/event descriptors only; Taffy eligibility is decided by
        // authored placement and slot policies so v2 template assets can use the shared layout pass.
        if child.anchor != Default::default()
            || child.pivot != Pivot::default()
            || child.position != Position::default()
        {
            return Ok(Some(UiLayoutEngineFallbackReason::ChildPlacementPolicy));
        }
        if !taffy_supports_axis_constraint_priority(child, taffy_main_axis(container).flatten()) {
            return Ok(Some(UiLayoutEngineFallbackReason::AxisConstraintPriority));
        }
        if !taffy_supports_child_layout_values(child) {
            return Ok(Some(UiLayoutEngineFallbackReason::InvalidLayoutValue));
        }

        let slot = slot_for_container_child(tree, parent_id, *child_id, container);
        if let Some(slot) = slot {
            if slot.canvas_placement.is_some() {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotCanvasPlacement));
            }
            if !taffy_supports_slot_layout_values(slot, container) {
                return Ok(Some(UiLayoutEngineFallbackReason::InvalidLayoutValue));
            }
            if !taffy_supports_slot_padding(slot.padding) {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotFramePolicy));
            }
            if !taffy_supports_slot_alignment(child, slot, container) {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotFramePolicy));
            }
        }
    }

    Ok(None)
}

fn taffy_parent_style(
    tree: &UiTree,
    parent_id: UiNodeId,
    container: UiContainerKind,
    frame: UiFrame,
) -> Option<Style> {
    let parent = tree.node(parent_id)?;
    let mut style = taffy_style_for_container(
        container,
        BoxConstraints {
            width: fixed_axis(frame.width),
            height: fixed_axis(frame.height),
        },
    )?;
    style.size = TaffySize {
        width: Dimension::length(frame.width.max(0.0)),
        height: Dimension::length(frame.height.max(0.0)),
    };
    style.min_size = TaffySize {
        width: Dimension::length(0.0),
        height: Dimension::length(0.0),
    };
    style.max_size = TaffySize {
        width: Dimension::auto(),
        height: Dimension::auto(),
    };
    style.align_items = Some(AlignItems::Stretch);
    style.align_content = Some(AlignContent::Start);

    match container {
        UiContainerKind::GridBox(config) => configure_grid_parent(&mut style, tree, parent, config),
        UiContainerKind::WrapBox(_) => {
            style.flex_wrap = FlexWrap::Wrap;
        }
        UiContainerKind::HorizontalBox(_) => {
            style.flex_direction = FlexDirection::Row;
        }
        UiContainerKind::VerticalBox(_) => {
            style.flex_direction = FlexDirection::Column;
        }
        _ => {}
    }

    Some(style)
}

fn configure_grid_parent(
    style: &mut Style,
    tree: &UiTree,
    parent: &UiTreeNode,
    config: UiGridBoxConfig,
) {
    let (columns, rows) = taffy_grid_dimensions(tree, parent, config);
    style.grid_template_columns = vec![fr(1.0); columns];
    style.grid_template_rows = vec![fr(1.0); rows];
}

fn taffy_grid_dimensions(
    tree: &UiTree,
    parent: &UiTreeNode,
    config: UiGridBoxConfig,
) -> (usize, usize) {
    let mut columns = config.columns.max(1);
    let mut rows = config.rows.max(1);
    let mut visible_child_count = 0usize;

    let container = UiContainerKind::GridBox(config);
    let ordered_children =
        ordered_children_for_container(tree, parent.node_id, &parent.children, container);
    for (index, child_id) in ordered_children.iter().copied().enumerate() {
        if !tree
            .node(child_id)
            .is_some_and(|node| node.effective_visibility().occupies_layout())
        {
            continue;
        }
        visible_child_count += 1;
        let slot = slot_for_container_child(tree, parent.node_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        columns = columns.max(placement.column + placement.column_span.max(1));
        rows = rows.max(placement.row + placement.row_span.max(1));
    }

    rows = rows.max(visible_child_count.div_ceil(columns).max(1));
    (columns, rows)
}

fn taffy_child_style(
    child: &UiTreeNode,
    parent_axis: Option<UiAxis>,
    parent_container: UiContainerKind,
    slot: Option<&UiSlot>,
) -> Style {
    let desired = UiSize::new(
        child.layout_cache.desired_size.width,
        child.layout_cache.desired_size.height,
    );
    let mut style = Style {
        size: TaffySize {
            width: child_axis_dimension(
                child.constraints.width,
                desired.width,
                parent_axis == Some(UiAxis::Horizontal),
                child.layout_stretch_width,
            ),
            height: child_axis_dimension(
                child.constraints.height,
                desired.height,
                parent_axis == Some(UiAxis::Vertical),
                child.layout_stretch_height,
            ),
        },
        min_size: TaffySize {
            width: min_dimension(child.constraints.width),
            height: min_dimension(child.constraints.height),
        },
        max_size: TaffySize {
            width: max_dimension(child.constraints.width),
            height: max_dimension(child.constraints.height),
        },
        ..Style::default()
    };

    if let Some(axis) = parent_axis {
        let (constraint, desired_extent, preserve_stretch) = match axis {
            UiAxis::Horizontal => (
                child.constraints.width,
                desired.width,
                child.layout_stretch_width,
            ),
            UiAxis::Vertical => (
                child.constraints.height,
                desired.height,
                child.layout_stretch_height,
            ),
        };
        style.flex_basis = flex_basis_for_axis(constraint, desired_extent, preserve_stretch);
        style.flex_grow = flex_grow_for_axis(constraint, desired_extent, preserve_stretch);
        style.flex_shrink = 1.0;
        if let Some(sizing) = linear_slot_sizing_for_taffy(parent_container, slot) {
            apply_linear_slot_sizing(&mut style, axis, constraint, desired_extent, sizing);
        }
    }

    if let Some(slot) = slot {
        apply_slot_frame_policy(&mut style, child, parent_container, slot);
    }

    if let UiContainerKind::WrapBox(config) = parent_container {
        style.min_size.width = Dimension::length(
            desired
                .width
                .max(config.item_min_width)
                .max(child.constraints.width.min)
                .max(0.0),
        );
    }
    if let UiContainerKind::GridBox(_) = parent_container {
        if let Some(placement) = slot.and_then(|slot| slot.grid_placement) {
            apply_grid_placement(&mut style, placement);
        }
    }

    style
}

fn taffy_supports_parent_layout_values(container: UiContainerKind, frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && taffy_supports_container_layout_values(container)
}

fn taffy_supports_container_layout_values(container: UiContainerKind) -> bool {
    match container {
        UiContainerKind::HorizontalBox(config) | UiContainerKind::VerticalBox(config) => {
            config.gap.is_finite()
        }
        UiContainerKind::WrapBox(config) => {
            config.horizontal_gap.is_finite()
                && config.vertical_gap.is_finite()
                && config.item_min_width.is_finite()
        }
        UiContainerKind::GridBox(config) => {
            config.column_gap.is_finite() && config.row_gap.is_finite()
        }
        _ => true,
    }
}

fn taffy_supports_axis_constraint_priority(
    child: &UiTreeNode,
    parent_axis: Option<UiAxis>,
) -> bool {
    let Some(parent_axis) = parent_axis else {
        return true;
    };
    match parent_axis {
        UiAxis::Horizontal => child.constraints.width.priority == 0,
        UiAxis::Vertical => child.constraints.height.priority == 0,
    }
}

fn taffy_supports_child_layout_values(child: &UiTreeNode) -> bool {
    axis_constraint_values_are_finite(child.constraints.width)
        && axis_constraint_values_are_finite(child.constraints.height)
        && child.layout_cache.desired_size.width.is_finite()
        && child.layout_cache.desired_size.height.is_finite()
}

fn axis_constraint_values_are_finite(constraint: AxisConstraint) -> bool {
    constraint.min.is_finite()
        && constraint.max.is_finite()
        && constraint.preferred.is_finite()
        && constraint.weight.is_finite()
}

fn linear_slot_sizing_for_taffy(
    parent_container: UiContainerKind,
    slot: Option<&UiSlot>,
) -> Option<UiLinearSlotSizing> {
    // WrapBox uses Flow slots; legacy wrap only uses those slots for order/padding/alignment.
    // Keep Taffy native wrap on that same contract instead of treating Flow as flex growth.
    match parent_container {
        UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => {
            slot.and_then(|slot| slot.linear_sizing)
        }
        _ => None,
    }
}

fn taffy_supports_slot_layout_values(slot: &UiSlot, parent_container: UiContainerKind) -> bool {
    match linear_slot_sizing_for_taffy(parent_container, Some(slot)) {
        Some(sizing) => linear_slot_sizing_values_are_finite(sizing),
        None => true,
    }
}

fn linear_slot_sizing_values_are_finite(sizing: UiLinearSlotSizing) -> bool {
    sizing.value.is_finite()
        && sizing.shrink_value.is_finite()
        && sizing.min.is_finite()
        && sizing.max.is_finite()
}

fn taffy_supports_slot_alignment(
    child: &UiTreeNode,
    slot: &UiSlot,
    parent_container: UiContainerKind,
) -> bool {
    match parent_container {
        UiContainerKind::HorizontalBox(_) | UiContainerKind::WrapBox(_) => {
            main_axis_alignment_supported(slot.alignment.horizontal)
                && axis_alignment_supported(slot.alignment.vertical, child.constraints.height)
        }
        UiContainerKind::VerticalBox(_) => {
            main_axis_alignment_supported(slot.alignment.vertical)
                && axis_alignment_supported(slot.alignment.horizontal, child.constraints.width)
        }
        UiContainerKind::GridBox(_) => {
            axis_alignment_supported(slot.alignment.horizontal, child.constraints.width)
                && axis_alignment_supported(slot.alignment.vertical, child.constraints.height)
        }
        _ => slot.alignment == Default::default(),
    }
}

fn taffy_supports_slot_padding(padding: UiMargin) -> bool {
    [padding.left, padding.right, padding.top, padding.bottom]
        .into_iter()
        .all(|value| value.is_finite() && value >= 0.0)
}

fn main_axis_alignment_supported(alignment: UiAlignment) -> bool {
    matches!(alignment, UiAlignment::Start | UiAlignment::Fill)
}

fn axis_alignment_supported(alignment: UiAlignment, constraint: AxisConstraint) -> bool {
    matches!(alignment, UiAlignment::Start | UiAlignment::Fill)
        || constraint.stretch_mode == StretchMode::Fixed
}

fn apply_slot_frame_policy(
    style: &mut Style,
    child: &UiTreeNode,
    parent_container: UiContainerKind,
    slot: &UiSlot,
) {
    style.margin = taffy_margin(slot.padding);
    match parent_container {
        UiContainerKind::HorizontalBox(_) | UiContainerKind::WrapBox(_) => {
            style.align_self =
                taffy_self_alignment(slot.alignment.vertical, child.constraints.height);
        }
        UiContainerKind::VerticalBox(_) => {
            style.align_self =
                taffy_self_alignment(slot.alignment.horizontal, child.constraints.width);
        }
        UiContainerKind::GridBox(_) => {
            style.justify_self =
                taffy_self_alignment(slot.alignment.horizontal, child.constraints.width);
            style.align_self =
                taffy_self_alignment(slot.alignment.vertical, child.constraints.height);
        }
        _ => {}
    }
}

fn taffy_margin(padding: UiMargin) -> Rect<LengthPercentageAuto> {
    Rect {
        left: LengthPercentageAuto::length(finite_spacing(padding.left)),
        right: LengthPercentageAuto::length(finite_spacing(padding.right)),
        top: LengthPercentageAuto::length(finite_spacing(padding.top)),
        bottom: LengthPercentageAuto::length(finite_spacing(padding.bottom)),
    }
}

fn finite_spacing(value: f32) -> f32 {
    value.is_finite().then_some(value.max(0.0)).unwrap_or(0.0)
}

fn taffy_self_alignment(alignment: UiAlignment, constraint: AxisConstraint) -> Option<AlignItems> {
    match alignment {
        UiAlignment::Start => None,
        UiAlignment::Center if constraint.stretch_mode == StretchMode::Fixed => {
            Some(AlignItems::Center)
        }
        UiAlignment::End if constraint.stretch_mode == StretchMode::Fixed => Some(AlignItems::End),
        UiAlignment::Fill => Some(AlignItems::Stretch),
        UiAlignment::Center | UiAlignment::End => None,
    }
}

fn apply_linear_slot_sizing(
    style: &mut Style,
    axis: UiAxis,
    constraint: AxisConstraint,
    desired_extent: f32,
    sizing: UiLinearSlotSizing,
) {
    let resolved = constraint.resolved();
    let basis = desired_extent.max(resolved.preferred).max(0.0);
    match sizing.rule {
        UiLinearSlotSizeRule::Auto => {
            style.flex_basis = positive_length_or_auto(basis);
            style.flex_grow = 0.0;
            style.flex_shrink = sizing.shrink_value.max(0.0);
        }
        UiLinearSlotSizeRule::Stretch => {
            style.flex_basis = Dimension::length(sizing.min.max(0.0));
            style.flex_grow = sizing.value.max(0.0);
            style.flex_shrink = sizing.shrink_value.max(0.0);
        }
        UiLinearSlotSizeRule::StretchContent => {
            style.flex_basis = positive_length_or_auto(basis);
            style.flex_grow = sizing.value.max(0.0);
            style.flex_shrink = sizing.shrink_value.max(0.0);
        }
    }
    apply_linear_slot_bounds(style, axis, sizing);
}

fn apply_linear_slot_bounds(style: &mut Style, axis: UiAxis, sizing: UiLinearSlotSizing) {
    let min_value = sizing.min.max(0.0);
    let max_value = (sizing.max >= 0.0).then(|| sizing.max.max(min_value));
    match axis {
        UiAxis::Horizontal => {
            style.min_size.width = max_dimension_length(style.min_size.width, min_value);
            if let Some(max_value) = max_value {
                style.max_size.width = merge_max_dimension(style.max_size.width, max_value);
            }
        }
        UiAxis::Vertical => {
            style.min_size.height = max_dimension_length(style.min_size.height, min_value);
            if let Some(max_value) = max_value {
                style.max_size.height = merge_max_dimension(style.max_size.height, max_value);
            }
        }
    }
}

fn max_dimension_length(current: Dimension, value: f32) -> Dimension {
    Dimension::length(current.into_option().unwrap_or(0.0).max(value.max(0.0)))
}

fn merge_max_dimension(current: Dimension, value: f32) -> Dimension {
    if let Some(current) = current.into_option() {
        Dimension::length(current.min(value.max(0.0)))
    } else {
        Dimension::length(value.max(0.0))
    }
}

fn apply_grid_placement(style: &mut Style, placement: UiGridSlotPlacement) {
    let start_column = grid_line(placement.column);
    let end_column = grid_line(placement.column + placement.column_span.max(1));
    let start_row = grid_line(placement.row);
    let end_row = grid_line(placement.row + placement.row_span.max(1));
    style.grid_column = Line {
        start: line::<GridPlacement>(start_column),
        end: line::<GridPlacement>(end_column),
    };
    style.grid_row = Line {
        start: line::<GridPlacement>(start_row),
        end: line::<GridPlacement>(end_row),
    };
}

fn grid_placement_for_child(
    slot: Option<&UiSlot>,
    index: usize,
    columns: usize,
) -> UiGridSlotPlacement {
    if let Some(placement) = slot.and_then(|slot| slot.grid_placement) {
        return placement.with_span(placement.column_span, placement.row_span);
    }

    let columns = columns.max(1);
    UiGridSlotPlacement::new(index % columns, index / columns)
}

fn grid_line(origin_zero_index: usize) -> i16 {
    origin_zero_index.saturating_add(1).min(i16::MAX as usize) as i16
}

fn taffy_main_axis(container: UiContainerKind) -> Option<Option<UiAxis>> {
    match container {
        UiContainerKind::HorizontalBox(_) => Some(Some(UiAxis::Horizontal)),
        UiContainerKind::VerticalBox(_) => Some(Some(UiAxis::Vertical)),
        UiContainerKind::WrapBox(_) => Some(Some(UiAxis::Horizontal)),
        UiContainerKind::GridBox(_) => Some(None),
        _ => None,
    }
}

fn child_axis_dimension(
    constraint: AxisConstraint,
    desired_extent: f32,
    main_axis: bool,
    preserve_stretch: bool,
) -> Dimension {
    let resolved = constraint.resolved();
    if constraint.stretch_mode == StretchMode::Fixed {
        return positive_length_or_auto(resolved.preferred.max(desired_extent));
    }
    if main_axis {
        let should_stretch = preserve_stretch || desired_extent <= 0.0;
        if should_stretch {
            return Dimension::auto();
        }
        return positive_length_or_auto(desired_extent.max(resolved.preferred));
    }
    Dimension::auto()
}

fn flex_basis_for_axis(
    constraint: AxisConstraint,
    desired_extent: f32,
    preserve_stretch: bool,
) -> Dimension {
    let resolved = constraint.resolved();
    if constraint.stretch_mode == StretchMode::Fixed {
        return positive_length_or_auto(resolved.preferred.max(desired_extent));
    }
    if preserve_stretch || desired_extent <= 0.0 {
        Dimension::length(resolved.preferred.max(0.0))
    } else {
        positive_length_or_auto(desired_extent.max(resolved.preferred))
    }
}

fn flex_grow_for_axis(
    constraint: AxisConstraint,
    desired_extent: f32,
    preserve_stretch: bool,
) -> f32 {
    if constraint.stretch_mode == StretchMode::Stretch
        && should_preserve_main_axis_stretch(constraint, desired_extent, preserve_stretch)
    {
        constraint.resolved().weight.max(0.0)
    } else {
        0.0
    }
}

fn should_preserve_main_axis_stretch(
    constraint: AxisConstraint,
    desired_extent: f32,
    preserve_stretch: bool,
) -> bool {
    preserve_stretch || desired_extent <= 0.0 || constraint.resolved().preferred > 0.0
}

fn positive_length_or_auto(value: f32) -> Dimension {
    if value > 0.0 {
        Dimension::length(value)
    } else {
        Dimension::auto()
    }
}

fn min_dimension(constraint: AxisConstraint) -> Dimension {
    Dimension::length(constraint.resolved().min.max(0.0))
}

fn max_dimension(constraint: AxisConstraint) -> Dimension {
    constraint
        .resolved()
        .max
        .map(|value| Dimension::length(value.max(0.0)))
        .unwrap_or_else(Dimension::auto)
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value.max(0.0),
        preferred: value.max(0.0),
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
