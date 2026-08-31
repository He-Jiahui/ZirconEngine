use std::ops::ControlFlow;

use crate::ui::asset_editor::{UiAssetEditorReflectionModel, UiDesignerSelectionModel};
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiNodeDefinition};

#[cfg(test)]
mod streaming_traversal_tests;

pub(super) fn selection_summary(selection: &UiDesignerSelectionModel) -> String {
    let primary = selection
        .primary_node_id
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let parent = selection
        .parent_node_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let mount = selection.mount.clone().unwrap_or_else(|| "-".to_string());
    format!("selected {primary} • parent {parent} • mount {mount}")
}

pub(super) fn build_hierarchy_items(
    document: &UiAssetDocument,
    selected: Option<&str>,
) -> Vec<String> {
    let mut items = Vec::new();
    let _ = visit_hierarchy_nodes(document, |_, depth, node| {
        let prefix = if selected == Some(node.node_id.as_str()) {
            "> "
        } else {
            "  "
        };
        let label = node
            .widget_type
            .as_deref()
            .or(node.component_ref.as_deref())
            .or(node.component.as_deref())
            .unwrap_or("Node");
        items.push(format!(
            "{prefix}{}{} [{label}]",
            "  ".repeat(depth),
            node.node_id.as_str()
        ));
        ControlFlow::<()>::Continue(())
    });
    items
}

pub(super) fn build_inspector_items(reflection: &UiAssetEditorReflectionModel) -> Vec<String> {
    let mut items = vec![
        format!("mode: {:?}", reflection.route.mode),
        format!("asset kind: {:?}", reflection.route.asset_kind),
        format!("dirty: {}", reflection.source_dirty),
        format!("undo: {}", reflection.can_undo),
        format!("redo: {}", reflection.can_redo),
        format!("preview: {}", reflection.preview_available),
    ];
    if let Some(node_id) = &reflection.selection.primary_node_id {
        items.push(format!("selection: {node_id}"));
    }
    if !reflection.style_inspector.classes.is_empty() {
        items.push(format!(
            "classes: {}",
            reflection.style_inspector.classes.join(", ")
        ));
    }
    items
}

pub(super) fn build_component_contract_items(root_class_policy: Option<&str>) -> Vec<String> {
    root_class_policy
        .map(|policy| vec![format!("root class policy: {policy}")])
        .unwrap_or_default()
}

pub(super) fn selected_hierarchy_index(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> i32 {
    let Some(primary) = selection.primary_node_id.as_deref() else {
        return -1;
    };
    visit_hierarchy_nodes(document, |index, _, node| {
        if node.node_id == primary {
            ControlFlow::Break(index as i32)
        } else {
            ControlFlow::Continue(())
        }
    })
    .unwrap_or(-1)
}

pub(super) fn hierarchy_node_id_at(
    document: &UiAssetDocument,
    target_index: usize,
) -> Option<&str> {
    visit_hierarchy_nodes(document, |index, _, node| {
        if index == target_index {
            ControlFlow::Break(node.node_id.as_str())
        } else {
            ControlFlow::Continue(())
        }
    })
}

fn visit_hierarchy_nodes<'a, Break>(
    document: &'a UiAssetDocument,
    mut visitor: impl FnMut(usize, usize, &'a UiNodeDefinition) -> ControlFlow<Break>,
) -> Option<Break> {
    fn visit_node<'a, Break>(
        node: &'a UiNodeDefinition,
        depth: usize,
        index: &mut usize,
        visitor: &mut impl FnMut(usize, usize, &'a UiNodeDefinition) -> ControlFlow<Break>,
    ) -> ControlFlow<Break> {
        let current_index = *index;
        *index = current_index.saturating_add(1);
        if let ControlFlow::Break(output) = visitor(current_index, depth, node) {
            return ControlFlow::Break(output);
        }
        for child in &node.children {
            if let ControlFlow::Break(output) =
                visit_node(&child.node, depth.saturating_add(1), index, visitor)
            {
                return ControlFlow::Break(output);
            }
        }
        ControlFlow::Continue(())
    }

    let mut index = 0;
    if let Some(root) = document.root.as_ref() {
        return match visit_node(root, 0, &mut index, &mut visitor) {
            ControlFlow::Break(output) => Some(output),
            ControlFlow::Continue(()) => None,
        };
    }
    for component in document.components.values() {
        if let ControlFlow::Break(output) = visit_node(&component.root, 0, &mut index, &mut visitor)
        {
            return Some(output);
        }
    }
    None
}

pub(super) fn selection_for_node(
    document: &UiAssetDocument,
    node_id: &str,
) -> UiDesignerSelectionModel {
    let mut selection = UiDesignerSelectionModel::single(node_id.to_string());
    if let Some((parent_node_id, mount)) = parent_for_node(document, node_id) {
        selection = selection.with_parent(parent_node_id);
        if let Some(mount) = mount {
            selection = selection.with_mount(mount);
        }
    }
    selection
}

pub(super) fn parent_for_node(
    document: &UiAssetDocument,
    node_id: &str,
) -> Option<(String, Option<String>)> {
    document.parent_of(node_id).map(|parent| {
        (
            parent.parent_node_id.to_string(),
            parent.mount.map(str::to_string),
        )
    })
}
