use taffy::geometry::Rect;
use taffy::prelude::{
    fr, line, AlignContent, AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap,
    GridPlacement, LengthPercentageAuto, Line, NodeId, Size as TaffySize, Style, TaffyTree,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAlignment, UiAxis, UiContainerKind, UiFrame,
        UiGridSlotPlacement, UiLayoutEngineFallbackReason, UiLayoutEngineTaffyTreeBuildStats,
        UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiSize, UiSlot,
    },
    tree::UiTreeNode,
};

use super::taffy_style_for_container;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TaffyChildLayoutInput<'a> {
    pub node_id: UiNodeId,
    pub node: &'a UiTreeNode,
    pub slot: Option<&'a UiSlot>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TaffyLayoutChildFrame {
    pub node_id: UiNodeId,
    pub frame: UiFrame,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TaffyLayoutOutcome {
    pub tree_build: UiLayoutEngineTaffyTreeBuildStats,
}

#[derive(Debug)]
pub(crate) struct TaffyLayoutBridgeScratch {
    taffy: TaffyTree<()>,
    child_node_ids: Vec<UiNodeId>,
    taffy_children: Vec<NodeId>,
    child_frames: Vec<TaffyLayoutChildFrame>,
    grid_columns: usize,
    grid_rows: usize,
    grid_visible_child_count: usize,
}

impl Default for TaffyLayoutBridgeScratch {
    fn default() -> Self {
        let mut taffy = TaffyTree::new();
        // Zircon projections preserve authored fractional metrics such as 30.5px controls.
        taffy.disable_rounding();
        Self {
            taffy,
            child_node_ids: Vec::new(),
            taffy_children: Vec::new(),
            child_frames: Vec::new(),
            grid_columns: 0,
            grid_rows: 0,
            grid_visible_child_count: 0,
        }
    }
}

impl TaffyLayoutBridgeScratch {
    pub(crate) fn begin_children(&mut self, container: UiContainerKind) {
        self.clear();
        if let UiContainerKind::GridBox(config) = container {
            self.grid_columns = config.columns.max(1);
            self.grid_rows = config.rows.max(1);
        }
    }

    pub(crate) fn push_child(
        &mut self,
        parent_container: UiContainerKind,
        parent_axis: Option<UiAxis>,
        child: TaffyChildLayoutInput<'_>,
    ) -> Result<(), TaffyLayoutBridgeError> {
        if matches!(parent_container, UiContainerKind::GridBox(_)) {
            let placement = grid_placement_for_child(
                child.slot.and_then(|slot| slot.grid_placement),
                self.child_node_ids.len(),
                self.grid_columns,
            );
            self.grid_visible_child_count = self.grid_visible_child_count.saturating_add(1);
            self.grid_columns = self.grid_columns.max(
                placement
                    .column
                    .saturating_add(placement.column_span.max(1)),
            );
            self.grid_rows = self
                .grid_rows
                .max(placement.row.saturating_add(placement.row_span.max(1)));
        }

        let taffy_child = self
            .taffy
            .new_leaf(taffy_child_style(
                child.node,
                parent_axis,
                parent_container,
                child.slot,
            ))
            .map_err(|_| TaffyLayoutBridgeError::TreeBuildFailed {
                tree_build: taffy_tree_stats(self.taffy_children.len()),
            })?;
        self.child_node_ids.push(child.node_id);
        self.taffy_children.push(taffy_child);
        Ok(())
    }

    pub(crate) fn child_frames(&self) -> &[TaffyLayoutChildFrame] {
        &self.child_frames
    }

    pub(crate) fn clear(&mut self) {
        self.taffy.clear();
        self.child_node_ids.clear();
        self.taffy_children.clear();
        self.child_frames.clear();
        self.grid_columns = 0;
        self.grid_rows = 0;
        self.grid_visible_child_count = 0;
    }

    #[cfg(test)]
    pub(crate) fn retained_capacities(&self) -> (usize, usize, usize) {
        (
            self.child_node_ids.capacity(),
            self.taffy_children.capacity(),
            self.child_frames.capacity(),
        )
    }

    fn grid_dimensions(&self, container: UiContainerKind) -> Option<(usize, usize)> {
        matches!(container, UiContainerKind::GridBox(_)).then(|| {
            let columns = self.grid_columns.max(1);
            let rows = self
                .grid_rows
                .max(self.grid_visible_child_count.div_ceil(columns).max(1));
            (columns, rows)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaffyLayoutBridgeError {
    StyleUnavailable {
        tree_build: UiLayoutEngineTaffyTreeBuildStats,
    },
    TreeBuildFailed {
        tree_build: UiLayoutEngineTaffyTreeBuildStats,
    },
    ComputeFailed {
        tree_build: UiLayoutEngineTaffyTreeBuildStats,
    },
}

impl TaffyLayoutBridgeError {
    pub(crate) fn fallback_reason(self) -> UiLayoutEngineFallbackReason {
        match self {
            Self::StyleUnavailable { .. } => UiLayoutEngineFallbackReason::TaffyStyleUnavailable,
            Self::TreeBuildFailed { .. } => UiLayoutEngineFallbackReason::TaffyTreeBuildFailed,
            Self::ComputeFailed { .. } => UiLayoutEngineFallbackReason::TaffyComputeFailed,
        }
    }

    pub(crate) fn tree_build(self) -> UiLayoutEngineTaffyTreeBuildStats {
        match self {
            Self::StyleUnavailable { tree_build }
            | Self::TreeBuildFailed { tree_build }
            | Self::ComputeFailed { tree_build } => tree_build,
        }
    }
}

pub(crate) fn compute_taffy_child_frames(
    container: UiContainerKind,
    frame: UiFrame,
    scratch: &mut TaffyLayoutBridgeScratch,
) -> Result<TaffyLayoutOutcome, TaffyLayoutBridgeError> {
    if taffy_main_axis(container).is_none() {
        return Err(TaffyLayoutBridgeError::StyleUnavailable {
            tree_build: taffy_tree_stats(0),
        });
    }

    scratch.child_frames.clear();
    let parent_style = taffy_parent_style(container, frame, scratch.grid_dimensions(container));
    let Some(parent_style) = parent_style else {
        return Err(TaffyLayoutBridgeError::StyleUnavailable {
            tree_build: taffy_tree_stats(scratch.taffy_children.len()),
        });
    };

    let taffy_parent = scratch
        .taffy
        .new_with_children(parent_style, &scratch.taffy_children)
        .map_err(|_| TaffyLayoutBridgeError::TreeBuildFailed {
            tree_build: taffy_tree_stats(scratch.taffy_children.len()),
        })?;
    let complete_taffy_tree_build = complete_taffy_tree_stats(scratch.taffy_children.len());
    scratch
        .taffy
        .compute_layout(
            taffy_parent,
            TaffySize {
                width: AvailableSpace::Definite(frame.width.max(0.0)),
                height: AvailableSpace::Definite(frame.height.max(0.0)),
            },
        )
        .map_err(|_| TaffyLayoutBridgeError::ComputeFailed {
            tree_build: complete_taffy_tree_build,
        })?;

    for (child_node_id, taffy_child) in scratch
        .child_node_ids
        .iter()
        .zip(scratch.taffy_children.iter().copied())
    {
        let layout = scratch.taffy.layout(taffy_child).map_err(|_| {
            TaffyLayoutBridgeError::ComputeFailed {
                tree_build: complete_taffy_tree_build,
            }
        })?;
        scratch.child_frames.push(TaffyLayoutChildFrame {
            node_id: *child_node_id,
            frame: UiFrame::new(
                frame.x + layout.location.x,
                frame.y + layout.location.y,
                layout.size.width.max(0.0),
                layout.size.height.max(0.0),
            ),
        });
    }

    Ok(TaffyLayoutOutcome {
        tree_build: complete_taffy_tree_build,
    })
}

pub(crate) fn taffy_supports_parent_layout_values(
    container: UiContainerKind,
    frame: UiFrame,
) -> bool {
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

pub(crate) fn taffy_supports_axis_constraint_priority(
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

pub(crate) fn taffy_supports_child_layout_values(child: &UiTreeNode) -> bool {
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

pub(crate) fn taffy_supports_slot_layout_values(
    slot: &UiSlot,
    parent_container: UiContainerKind,
) -> bool {
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

pub(crate) fn taffy_supports_slot_alignment(
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

pub(crate) fn taffy_supports_slot_padding(padding: UiMargin) -> bool {
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

fn taffy_parent_style(
    container: UiContainerKind,
    frame: UiFrame,
    grid_dimensions: Option<(usize, usize)>,
) -> Option<Style> {
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
        UiContainerKind::GridBox(config) => configure_grid_parent(
            &mut style,
            grid_dimensions.unwrap_or((config.columns.max(1), config.rows.max(1))),
        ),
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

fn configure_grid_parent(style: &mut Style, dimensions: (usize, usize)) {
    let (columns, rows) = dimensions;
    style.grid_template_columns = vec![fr(1.0); columns];
    style.grid_template_rows = vec![fr(1.0); rows];
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

fn linear_slot_sizing_for_taffy(
    parent_container: UiContainerKind,
    slot: Option<&UiSlot>,
) -> Option<UiLinearSlotSizing> {
    // WrapBox uses Flow slots for order, padding, and alignment only.
    // Keep Taffy native wrap on that same contract instead of treating Flow as flex growth.
    match parent_container {
        UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => {
            slot.and_then(|slot| slot.linear_sizing)
        }
        _ => None,
    }
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
    placement: Option<UiGridSlotPlacement>,
    index: usize,
    columns: usize,
) -> UiGridSlotPlacement {
    if let Some(placement) = placement {
        return placement.with_span(placement.column_span, placement.row_span);
    }

    let columns = columns.max(1);
    UiGridSlotPlacement::new(index % columns, index / columns)
}

fn grid_line(origin_zero_index: usize) -> i16 {
    origin_zero_index.saturating_add(1).min(i16::MAX as usize) as i16
}

pub(crate) fn taffy_main_axis(container: UiContainerKind) -> Option<Option<UiAxis>> {
    match container {
        UiContainerKind::HorizontalBox(_) => Some(Some(UiAxis::Horizontal)),
        UiContainerKind::VerticalBox(_) => Some(Some(UiAxis::Vertical)),
        UiContainerKind::WrapBox(_) => Some(Some(UiAxis::Horizontal)),
        UiContainerKind::GridBox(_) => Some(None),
        UiContainerKind::BlockBox => Some(None),
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

fn taffy_tree_stats(node_count: usize) -> UiLayoutEngineTaffyTreeBuildStats {
    UiLayoutEngineTaffyTreeBuildStats::new(u64::try_from(node_count).unwrap_or(u64::MAX))
}

fn complete_taffy_tree_stats(child_count: usize) -> UiLayoutEngineTaffyTreeBuildStats {
    taffy_tree_stats(child_count.saturating_add(1))
}
