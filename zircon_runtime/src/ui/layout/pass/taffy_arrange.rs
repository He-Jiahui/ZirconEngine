use crate::ui::{layout::taffy_style_for_container, tree::UiRuntimeTreeAccessExt};
use taffy::prelude::{
    fr, AlignContent, AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap,
    Size as TaffySize, Style, TaffyTree,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        AxisConstraint, BoxConstraints, Pivot, Position, StretchMode, UiAxis, UiContainerKind,
        UiFrame, UiGridBoxConfig, UiSize,
    },
    tree::{UiTree, UiTreeError, UiTreeNode},
};

use super::arrange::arrange_node;
use super::slot::{
    has_slot_frame_policy, ordered_children_for_container, slot_for_container_child,
};

pub(super) fn try_arrange_taffy_owned_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
) -> Result<bool, UiTreeError> {
    let container = tree
        .node(parent_id)
        .ok_or(UiTreeError::MissingNode(parent_id))?
        .container;
    let Some(axis) = taffy_main_axis(container) else {
        return Ok(false);
    };
    if !taffy_child_contracts_supported(tree, parent_id, children, container)? {
        return Ok(false);
    }

    let ordered_children = ordered_children_for_container(tree, parent_id, children, container);
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut taffy_children = Vec::with_capacity(ordered_children.len());
    for child_id in &ordered_children {
        let child = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        let Ok(taffy_child) = taffy.new_leaf(taffy_child_style(child, axis, container)) else {
            return Ok(false);
        };
        taffy_children.push(taffy_child);
    }

    let Some(parent_style) = taffy_parent_style(tree, parent_id, container, frame) else {
        return Ok(false);
    };
    let Ok(taffy_parent) = taffy.new_with_children(parent_style, &taffy_children) else {
        return Ok(false);
    };
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
        return Ok(false);
    }

    for (child_id, taffy_child) in ordered_children.iter().copied().zip(taffy_children) {
        let Ok(layout) = taffy.layout(taffy_child) else {
            return Ok(false);
        };
        let child_frame = UiFrame::new(
            frame.x + layout.location.x,
            frame.y + layout.location.y,
            layout.size.width.max(0.0),
            layout.size.height.max(0.0),
        );
        arrange_node(tree, child_id, child_frame, inherited_clip)?;
    }

    Ok(true)
}

fn taffy_child_contracts_supported(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
) -> Result<bool, UiTreeError> {
    for child_id in children {
        let child = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        if !child.effective_visibility().occupies_layout() {
            return Ok(false);
        }
        // Template metadata carries render/event descriptors only; Taffy eligibility is decided by
        // authored placement and slot policies so v2 template assets can use the shared layout pass.
        if child.anchor != Default::default()
            || child.pivot != Pivot::default()
            || child.position != Position::default()
        {
            return Ok(false);
        }

        let slot = slot_for_container_child(tree, parent_id, *child_id, container);
        if has_slot_frame_policy(slot) {
            return Ok(false);
        }
        if slot.is_some_and(|slot| {
            slot.linear_sizing.is_some()
                || slot.canvas_placement.is_some()
                || slot.grid_placement.is_some()
        }) {
            return Ok(false);
        }
    }

    Ok(true)
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
    let columns = config.columns.max(1);
    let visible_child_count = parent
        .children
        .iter()
        .filter(|child_id| {
            tree.node(**child_id)
                .is_some_and(|node| node.effective_visibility().occupies_layout())
        })
        .count();
    let rows = config
        .rows
        .max(1)
        .max(visible_child_count.div_ceil(columns).max(1));
    style.grid_template_columns = vec![fr(1.0); columns];
    style.grid_template_rows = vec![fr(1.0); rows];
}

fn taffy_child_style(
    child: &UiTreeNode,
    parent_axis: Option<UiAxis>,
    parent_container: UiContainerKind,
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

    style
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
