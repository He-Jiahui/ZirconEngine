use crate::ui::layout::solve_axis_constraints;
use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, LayoutBoundary, StretchMode, UiAxis, UiFrame, UiSize},
    tree::{UiTree, UiTreeError},
};

use super::slot::{slot_for_container_child, slot_padding};

pub(crate) fn resolve_linear_child_main_extents(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    axis: UiAxis,
    available_extent: f32,
    gap: f32,
) -> Result<Vec<f32>, UiTreeError> {
    let layout_child_count = children
        .iter()
        .filter(|child_id| {
            tree.node(**child_id)
                .is_some_and(|node| node.effective_visibility().occupies_layout())
        })
        .count();
    let gap_total = gap.max(0.0) * layout_child_count.saturating_sub(1) as f32;
    let available_extent = (available_extent - gap_total).max(0.0);
    let mut constraints = Vec::with_capacity(children.len());

    for child_id in children {
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        if !node.effective_visibility().occupies_layout() {
            constraints.push(collapsed_axis_constraint());
            continue;
        }
        let slot = slot_for_container_child(tree, parent_id, *child_id, linear_container(axis));
        let padding = slot_padding(slot);
        let padding_extent = match axis {
            UiAxis::Horizontal => padding.horizontal(),
            UiAxis::Vertical => padding.vertical(),
        };
        let desired_extent = size_axis_extent(
            UiSize::new(
                node.layout_cache.desired_size.width,
                node.layout_cache.desired_size.height,
            ),
            axis,
        );
        let preserve_stretch = match axis {
            UiAxis::Horizontal => node.layout_stretch_width,
            UiAxis::Vertical => node.layout_stretch_height,
        };
        constraints.push(linear_main_axis_constraint(
            match axis {
                UiAxis::Horizontal => node.constraints.width,
                UiAxis::Vertical => node.constraints.height,
            },
            desired_extent,
            padding_extent,
            preserve_stretch,
        ));
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

fn linear_main_axis_constraint(
    mut constraint: AxisConstraint,
    desired_extent: f32,
    padding_extent: f32,
    preserve_stretch: bool,
) -> AxisConstraint {
    let was_default = constraint == AxisConstraint::default();
    let padding_extent = padding_extent.max(0.0);
    constraint = inflate_axis_constraint(constraint, padding_extent);

    let desired_extent = desired_extent.max(0.0) + padding_extent;
    if desired_extent > 0.0 && was_default && !preserve_stretch {
        constraint.preferred = desired_extent;
        constraint.stretch_mode = StretchMode::Fixed;
    }
    constraint
}

fn inflate_axis_constraint(mut constraint: AxisConstraint, padding_extent: f32) -> AxisConstraint {
    if padding_extent <= 0.0 {
        return constraint;
    }
    constraint.min += padding_extent;
    if constraint.max >= 0.0 {
        constraint.max += padding_extent;
    }
    constraint.preferred += padding_extent;
    constraint
}

fn linear_container(axis: UiAxis) -> zircon_runtime_interface::ui::layout::UiContainerKind {
    match axis {
        UiAxis::Horizontal => {
            zircon_runtime_interface::ui::layout::UiContainerKind::HorizontalBox(Default::default())
        }
        UiAxis::Vertical => {
            zircon_runtime_interface::ui::layout::UiContainerKind::VerticalBox(Default::default())
        }
    }
}

fn collapsed_axis_constraint() -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: 0.0,
        preferred: 0.0,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

pub(crate) fn stacked_axis_extent(constraint: AxisConstraint, desired: f32) -> f32 {
    let resolved = constraint.resolved();
    clamp_axis(desired, resolved.min, resolved.max)
}

fn clamp_axis(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}
