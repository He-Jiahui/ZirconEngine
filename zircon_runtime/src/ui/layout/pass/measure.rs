use crate::ui::{surface::measure_text, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{DesiredSize, UiAxis, UiContainerKind, UiSize},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeError},
};

use super::{axis::desired_axis, material::measure_material_content};

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
    for child_id in children {
        let desired = measure_node(tree, child_id)?;
        if tree
            .node(child_id)
            .is_some_and(|child| child.effective_visibility().occupies_layout())
        {
            child_desired.push(desired);
        }
    }

    let content_size = measure_content_size(container, &child_desired, template_metadata.as_ref());
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
    container: UiContainerKind,
    child_desired: &[DesiredSize],
    metadata: Option<&UiTemplateNodeMetadata>,
) -> UiSize {
    if child_desired.is_empty() {
        return measure_leaf_content_size(metadata);
    }

    let content_size = match container {
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
