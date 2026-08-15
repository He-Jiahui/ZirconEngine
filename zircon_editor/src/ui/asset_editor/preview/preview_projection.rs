use std::collections::BTreeMap;

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiNodeDefinition};
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

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

/// Compact preview data needed by palette drag hit testing. Presentation-only
/// labels and selection state deliberately stay out of this cache.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPreviewHitIndex {
    pub(crate) canvas_nodes: Vec<UiAssetPreviewHitNode>,
    pub(crate) surface_width: f32,
    pub(crate) surface_height: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPreviewHitNode {
    pub(crate) node_id: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
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
    let control_id_index = control_id_index(document);
    for command in &preview_host.surface().render_extract.list.commands {
        let Some(tree_node) = preview_host.surface().tree.node(command.node_id) else {
            continue;
        };
        let metadata = tree_node.template_metadata.as_ref();
        let document_node = metadata
            .and_then(|metadata| metadata.control_id.as_deref())
            .and_then(|control_id| control_id_index.get(control_id).copied());
        let document_node_id = document_node.map(|node| node.node_id.as_str());
        let label = metadata
            .and_then(|metadata| metadata.control_id.as_deref())
            .or(document_node_id)
            .map(str::to_string)
            .unwrap_or_else(|| format!("#{}", command.node_id.0));
        let kind = preview_item_component_label(document_node, metadata)
            .unwrap_or_else(|| "Node".to_string());
        let selected = document_node_id == selected_node_id;
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
            node_id: document_node_id
                .map(str::to_string)
                .unwrap_or_else(|| label.clone()),
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

pub(crate) fn build_preview_hit_index(
    document: &UiAssetDocument,
    preview_host: Option<&UiAssetPreviewHost>,
) -> Option<UiAssetPreviewHitIndex> {
    let preview_host = preview_host?;
    let mut hit_index = UiAssetPreviewHitIndex {
        surface_width: preview_host.preview_size().width.max(0.0),
        surface_height: preview_host.preview_size().height.max(0.0),
        ..UiAssetPreviewHitIndex::default()
    };
    let control_id_index = control_id_index(document);
    for command in &preview_host.surface().render_extract.list.commands {
        let Some(tree_node) = preview_host.surface().tree.node(command.node_id) else {
            continue;
        };
        let control_id = tree_node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref());
        let document_node_id = control_id
            .and_then(|control_id| control_id_index.get(control_id).copied())
            .map(|node| node.node_id.as_str());
        hit_index.canvas_nodes.push(UiAssetPreviewHitNode {
            node_id: document_node_id
                .or(control_id)
                .map(str::to_string)
                .unwrap_or_else(|| format!("#{}", command.node_id.0)),
            x: command.frame.x,
            y: command.frame.y,
            width: command.frame.width,
            height: command.frame.height,
        });
    }
    Some(hit_index)
}

pub fn preview_node_id_for_index(
    document: &UiAssetDocument,
    preview_host: &UiAssetPreviewHost,
    index: usize,
) -> Option<String> {
    let control_id_index = control_id_index(document);
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
            control_id_index
                .get(control_id)
                .map(|node| node.node_id.clone())
        })
        .nth(index)
}

fn preview_item_component_label(
    document_node: Option<&UiNodeDefinition>,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> Option<String> {
    let rendered_component = metadata
        .map(|metadata| metadata.component.as_str())
        .filter(|component| !component.is_empty());
    let document_component = document_node.and_then(node_component_label);

    match (document_component, rendered_component) {
        (Some(document_component), Some(rendered_component))
            if document_component != rendered_component =>
        {
            Some(format!("{document_component}/{rendered_component}"))
        }
        (Some(document_component), _) => Some(document_component.to_string()),
        (_, Some(rendered_component)) => Some(rendered_component.to_string()),
        _ => None,
    }
}

fn node_component_label(node: &UiNodeDefinition) -> Option<&str> {
    node.component_ref
        .as_deref()
        .and_then(|reference| reference.split_once('#').map(|(_, component)| component))
        .or_else(|| node.component.as_deref())
        .or_else(|| node.widget_type.as_deref())
}

fn control_id_index(document: &UiAssetDocument) -> BTreeMap<&str, &UiNodeDefinition> {
    let mut index = BTreeMap::new();
    for node in document.iter_nodes() {
        if let Some(control_id) = node.control_id.as_deref() {
            let _ = index.entry(control_id).or_insert(node);
        }
    }
    index
}
