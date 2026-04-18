use zircon_editor_ui::{UiAssetEditorReflectionModel, UiDesignerSelectionModel};
use zircon_ui::UiAssetDocument;

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
    fn visit(
        output: &mut Vec<String>,
        document: &UiAssetDocument,
        node_id: &str,
        depth: usize,
        selected: Option<&str>,
    ) {
        let Some(node) = document.nodes.get(node_id) else {
            return;
        };
        let prefix = if selected == Some(node_id) {
            "> "
        } else {
            "  "
        };
        let label = node
            .widget_type
            .clone()
            .or_else(|| node.component_ref.clone())
            .unwrap_or_else(|| "Node".to_string());
        output.push(format!("{prefix}{}{node_id} [{label}]", "  ".repeat(depth)));
        for child in &node.children {
            visit(output, document, &child.child, depth + 1, selected);
        }
    }

    let mut items = Vec::new();
    if let Some(root) = &document.root {
        visit(&mut items, document, &root.node, 0, selected);
    }
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

pub(super) fn selected_hierarchy_index(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> i32 {
    let Some(primary) = selection.primary_node_id.as_deref() else {
        return -1;
    };
    hierarchy_node_ids(document)
        .iter()
        .position(|id| id == primary)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

pub(super) fn hierarchy_node_ids(document: &UiAssetDocument) -> Vec<String> {
    fn visit(output: &mut Vec<String>, document: &UiAssetDocument, node_id: &str) {
        output.push(node_id.to_string());
        let Some(node) = document.nodes.get(node_id) else {
            return;
        };
        for child in &node.children {
            visit(output, document, &child.child);
        }
    }

    let mut items = Vec::new();
    if let Some(root) = &document.root {
        visit(&mut items, document, &root.node);
    }
    items
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
    for (parent_id, node) in &document.nodes {
        for child in &node.children {
            if child.child == node_id {
                return Some((parent_id.clone(), child.mount.clone()));
            }
        }
    }
    None
}
