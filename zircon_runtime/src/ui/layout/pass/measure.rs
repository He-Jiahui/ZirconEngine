use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{DesiredSize, UiAxis, UiContainerKind, UiSize},
    tree::{UiTree, UiTreeError},
};

use super::axis::desired_axis;

pub(crate) fn measure_node(
    tree: &mut UiTree,
    node_id: UiNodeId,
) -> Result<DesiredSize, UiTreeError> {
    let (children, layout_boundary, constraints, container) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (
            node.children.clone(),
            node.layout_boundary,
            node.constraints,
            node.container,
        )
    };

    let mut child_desired = Vec::with_capacity(children.len());
    for child_id in children {
        child_desired.push(measure_node(tree, child_id)?);
    }

    let content_size = measure_content_size(container, &child_desired);
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

fn measure_content_size(container: UiContainerKind, child_desired: &[DesiredSize]) -> UiSize {
    match container {
        UiContainerKind::Free | UiContainerKind::Container | UiContainerKind::Overlay => {
            UiSize::new(
                child_desired
                    .iter()
                    .map(|size| size.width)
                    .fold(0.0_f32, f32::max),
                child_desired
                    .iter()
                    .map(|size| size.height)
                    .fold(0.0_f32, f32::max),
            )
        }
        UiContainerKind::Space => UiSize::default(),
        UiContainerKind::HorizontalBox(config) => {
            measure_linear_content_size(UiAxis::Horizontal, config.gap, child_desired)
        }
        UiContainerKind::VerticalBox(config) => {
            measure_linear_content_size(UiAxis::Vertical, config.gap, child_desired)
        }
        UiContainerKind::ScrollableBox(config) => {
            measure_linear_content_size(config.axis, config.gap, child_desired)
        }
    }
}

fn measure_linear_content_size(axis: UiAxis, gap: f32, child_desired: &[DesiredSize]) -> UiSize {
    let gap = gap.max(0.0);
    let count = child_desired.len() as f32;
    match axis {
        UiAxis::Vertical => UiSize::new(
            child_desired
                .iter()
                .map(|size| size.width)
                .fold(0.0_f32, f32::max),
            child_desired.iter().map(|size| size.height).sum::<f32>()
                + gap * (count - 1.0).max(0.0),
        ),
        UiAxis::Horizontal => UiSize::new(
            child_desired.iter().map(|size| size.width).sum::<f32>() + gap * (count - 1.0).max(0.0),
            child_desired
                .iter()
                .map(|size| size.height)
                .fold(0.0_f32, f32::max),
        ),
    }
}
