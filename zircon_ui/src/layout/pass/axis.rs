use crate::layout::solve_axis_constraints;
use crate::tree::UiTreeError;
use crate::{AxisConstraint, LayoutBoundary, StretchMode, UiAxis, UiFrame, UiSize, UiTree};

pub(crate) fn resolve_linear_child_main_extents(
    tree: &UiTree,
    children: &[crate::event_ui::UiNodeId],
    axis: UiAxis,
    available_extent: f32,
    gap: f32,
) -> Result<Vec<f32>, UiTreeError> {
    let gap_total = gap.max(0.0) * children.len().saturating_sub(1) as f32;
    let available_extent = (available_extent - gap_total).max(0.0);
    let mut constraints = Vec::with_capacity(children.len());

    for child_id in children {
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        constraints.push(match axis {
            UiAxis::Horizontal => node.constraints.width,
            UiAxis::Vertical => node.constraints.height,
        });
    }

    Ok(solve_axis_constraints(available_extent, &constraints)
        .into_iter()
        .map(|constraint| constraint.resolved)
        .collect())
}

pub(crate) fn size_axis_extent(size: UiSize, axis: UiAxis) -> f32 {
    match axis {
        UiAxis::Horizontal => size.width,
        UiAxis::Vertical => size.height,
    }
}

pub(crate) fn frame_axis_extent(frame: UiFrame, axis: UiAxis) -> f32 {
    match axis {
        UiAxis::Horizontal => frame.width,
        UiAxis::Vertical => frame.height,
    }
}

pub(crate) fn desired_axis(
    layout_boundary: LayoutBoundary,
    constraint: AxisConstraint,
    content_value: f32,
) -> f32 {
    let resolved = constraint.resolved();
    let preferred = match layout_boundary {
        LayoutBoundary::ContentDriven => content_value.max(resolved.preferred),
        LayoutBoundary::ParentDirected | LayoutBoundary::Fixed => resolved.preferred,
    };
    clamp_axis(preferred, resolved.min, resolved.max)
}

pub(crate) fn arranged_axis_extent(
    constraint: AxisConstraint,
    desired: f32,
    available: f32,
) -> f32 {
    let resolved = constraint.resolved();
    let base = match constraint.stretch_mode {
        StretchMode::Fixed => desired,
        StretchMode::Stretch => available,
    };
    clamp_axis(base, resolved.min, resolved.max)
}

pub(crate) fn stacked_axis_extent(constraint: AxisConstraint, desired: f32) -> f32 {
    let resolved = constraint.resolved();
    clamp_axis(desired, resolved.min, resolved.max)
}

fn clamp_axis(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}
