use crate::ui::UiDesignerSelectionModel;
use zircon_ui::{UiAssetDocument, UiTemplateNodeMetadata};

use super::preview_host::UiAssetPreviewHost;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetCanvasNodePresentation {
    pub node_id: String,
    pub label: String,
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub depth: i32,
    pub z_index: i32,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetPreviewProjection {
    pub items: Vec<String>,
    pub canvas_nodes: Vec<UiAssetCanvasNodePresentation>,
    pub selected_index: i32,
    pub surface_width: f32,
    pub surface_height: f32,
}

impl Default for UiAssetPreviewProjection {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            canvas_nodes: Vec::new(),
            selected_index: -1,
            surface_width: 0.0,
            surface_height: 0.0,
        }
    }
}

pub fn build_preview_projection(
    document: &UiAssetDocument,
    preview_host: Option<&UiAssetPreviewHost>,
    selection: &UiDesignerSelectionModel,
) -> UiAssetPreviewProjection {
    let Some(preview_host) = preview_host else {
        return UiAssetPreviewProjection {
            items: vec!["no shared preview surface".to_string()],
            ..UiAssetPreviewProjection::default()
        };
    };

    let mut projection = UiAssetPreviewProjection {
        surface_width: preview_host.preview_size().width.max(0.0),
        surface_height: preview_host.preview_size().height.max(0.0),
        ..UiAssetPreviewProjection::default()
    };
    let selected_node_id = selection.primary_node_id.as_deref();
    for command in &preview_host.surface().render_extract.list.commands {
        let Some(tree_node) = preview_host.surface().tree.node(command.node_id) else {
            continue;
        };
        let metadata = tree_node.template_metadata.as_ref();
        let document_node_id = metadata
            .and_then(|metadata| metadata.control_id.as_deref())
            .and_then(|control_id| node_id_by_control_id(document, control_id));
        let label = metadata
            .and_then(|metadata| metadata.control_id.clone())
            .or_else(|| document_node_id.clone())
            .unwrap_or_else(|| format!("#{}", command.node_id.0));
        let kind =
            preview_item_component_label(document, metadata).unwrap_or_else(|| "Node".to_string());
        let selected = document_node_id.as_deref() == selected_node_id;
        projection.items.push(format!(
            "{} [{}] {:.0},{:.0} {:.0}x{:.0}",
            label,
            kind,
            command.frame.x,
            command.frame.y,
            command.frame.width,
            command.frame.height
        ));
        projection.canvas_nodes.push(UiAssetCanvasNodePresentation {
            node_id: document_node_id.unwrap_or_else(|| label.clone()),
            label,
            kind,
            x: command.frame.x,
            y: command.frame.y,
            width: command.frame.width,
            height: command.frame.height,
            depth: tree_node.node_path.0.matches('/').count() as i32,
            z_index: command.z_index,
            selected,
        });
        if selected {
            projection.selected_index = projection.canvas_nodes.len() as i32 - 1;
        }
    }

    projection
}

pub fn preview_node_id_for_index(
    document: &UiAssetDocument,
    preview_host: &UiAssetPreviewHost,
    index: usize,
) -> Option<String> {
    preview_host
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter_map(|command| {
            let tree_node = preview_host.surface().tree.node(command.node_id)?;
            let control_id = tree_node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())?;
            node_id_by_control_id(document, control_id)
        })
        .nth(index)
}

fn preview_item_component_label(
    document: &UiAssetDocument,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<String> {
    let rendered_component = metadata
        .map(|metadata| metadata.component.clone())
        .filter(|component| !component.is_empty());
    let document_component = metadata
        .and_then(|metadata| metadata.control_id.as_deref())
        .and_then(|control_id| node_id_by_control_id(document, control_id))
        .and_then(|node_id| document.nodes.get(&node_id))
        .and_then(node_component_label);

    match (document_component, rendered_component) {
        (Some(document_component), Some(rendered_component))
            if document_component != rendered_component =>
        {
            Some(format!("{document_component}/{rendered_component}"))
        }
        (Some(document_component), _) => Some(document_component),
        (_, Some(rendered_component)) => Some(rendered_component),
        _ => None,
    }
}

fn node_component_label(node: &zircon_ui::UiNodeDefinition) -> Option<String> {
    node.component_ref
        .as_deref()
        .and_then(|reference| reference.split_once('#').map(|(_, component)| component))
        .map(str::to_string)
        .or_else(|| node.component.clone())
        .or_else(|| node.widget_type.clone())
}

fn node_id_by_control_id(document: &UiAssetDocument, control_id: &str) -> Option<String> {
    document.nodes.iter().find_map(|(node_id, node)| {
        (node.control_id.as_deref() == Some(control_id)).then(|| node_id.clone())
    })
}
