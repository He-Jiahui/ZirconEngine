use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ui::UiDesignerSelectionModel;
use zircon_runtime::ui::template::{UiAssetDocument, UiAssetKind};
use zircon_runtime::ui::template::{UiAssetHeader, UiAssetRoot};
use zircon_runtime::ui::template::{UiNodeDefinition, UiNodeDefinitionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalWidgetDraft {
    pub(crate) asset_id: String,
    pub(crate) component_name: String,
    pub(crate) document_id: String,
}

pub(super) fn can_promote_selected_component_to_external_widget(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selected_local_component_name(document, selection).is_some()
}

pub(super) fn default_external_widget_draft(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<UiAssetExternalWidgetDraft> {
    let component_name = selected_local_component_name(document, selection)?;
    let slug = widget_slug(&component_name);
    Some(UiAssetExternalWidgetDraft {
        asset_id: format!("res://ui/widgets/{slug}.ui.toml"),
        component_name,
        document_id: format!("ui.widgets.{slug}"),
    })
}

pub(super) fn selected_local_component_name(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?;
    let node = document.nodes.get(node_id)?;
    if node.kind != UiNodeDefinitionKind::Component {
        return None;
    }
    let component_name = node.component.as_deref()?;
    document
        .components
        .contains_key(component_name)
        .then(|| component_name.to_string())
}

pub(super) fn promote_selected_component_to_external_widget(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    widget_asset_id: &str,
    widget_component_name: &str,
    widget_document_id: &str,
) -> Option<UiAssetDocument> {
    let source_component_name = selected_local_component_name(document, selection)?;
    let component = document.components.get(&source_component_name)?.clone();
    let dependency_names = collect_component_dependency_names(document, &source_component_name)?;
    if widget_component_name != source_component_name
        && dependency_names.contains(widget_component_name)
    {
        return None;
    }
    let dependency_nodes = collect_component_dependency_nodes(document, &dependency_names)?;
    let target_ref = ensure_component_ref(widget_asset_id, widget_component_name);

    let components = dependency_names
        .iter()
        .filter_map(|name| {
            document.components.get(name).cloned().map(|component| {
                (
                    if name == &source_component_name {
                        widget_component_name.to_string()
                    } else {
                        name.clone()
                    },
                    component,
                )
            })
        })
        .collect();

    let nodes = dependency_nodes
        .into_iter()
        .map(|(node_id, mut node)| {
            if node.kind == UiNodeDefinitionKind::Component
                && node.component.as_deref() == Some(source_component_name.as_str())
            {
                node.component = Some(widget_component_name.to_string());
            }
            (node_id, node)
        })
        .collect();

    let widget_document = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Widget,
            id: widget_document_id.to_string(),
            version: 1,
            display_name: widget_component_name.to_string(),
        },
        imports: document.imports.clone(),
        tokens: document.tokens.clone(),
        root: Some(UiAssetRoot {
            node: component.root.clone(),
        }),
        nodes,
        components,
        stylesheets: document.stylesheets.clone(),
    };

    if !document
        .imports
        .widgets
        .iter()
        .any(|reference| reference == &target_ref)
    {
        document.imports.widgets.push(target_ref.clone());
    }

    for node in document.nodes.values_mut() {
        if node.kind == UiNodeDefinitionKind::Component
            && node.component.as_deref() == Some(source_component_name.as_str())
        {
            node.kind = UiNodeDefinitionKind::Reference;
            node.widget_type = None;
            node.component = None;
            node.component_ref = Some(target_ref.clone());
            node.slot_name = None;
            node.props.clear();
            node.layout = None;
            node.bindings.clear();
        }
    }

    let _ = document.components.remove(&source_component_name);
    for node_id in collect_subtree_node_ids(document, &component.root)? {
        let _ = document.nodes.remove(&node_id);
    }

    Some(widget_document)
}

fn ensure_component_ref(reference: &str, component_name: &str) -> String {
    if reference.contains('#') {
        reference.to_string()
    } else {
        format!("{reference}#{component_name}")
    }
}

fn collect_component_dependency_names(
    document: &UiAssetDocument,
    root_component: &str,
) -> Option<BTreeSet<String>> {
    let mut visited = BTreeSet::new();
    let mut pending = VecDeque::from([root_component.to_string()]);
    while let Some(component_name) = pending.pop_front() {
        if !visited.insert(component_name.clone()) {
            continue;
        }
        let component = document.components.get(&component_name)?;
        let subtree = collect_subtree_node_ids(document, &component.root)?;
        for node_id in subtree {
            let node = document.nodes.get(&node_id)?;
            if node.kind == UiNodeDefinitionKind::Component {
                if let Some(local_component) = node.component.as_deref() {
                    if document.components.contains_key(local_component) {
                        pending.push_back(local_component.to_string());
                    }
                }
            }
        }
    }
    Some(visited)
}

fn collect_component_dependency_nodes(
    document: &UiAssetDocument,
    dependency_names: &BTreeSet<String>,
) -> Option<BTreeMap<String, UiNodeDefinition>> {
    let mut nodes = BTreeMap::new();
    for component_name in dependency_names {
        let component = document.components.get(component_name)?;
        for node_id in collect_subtree_node_ids(document, &component.root)? {
            let node = document.nodes.get(&node_id)?.clone();
            let _ = nodes.insert(node_id, node);
        }
    }
    Some(nodes)
}

fn collect_subtree_node_ids(document: &UiAssetDocument, root_id: &str) -> Option<BTreeSet<String>> {
    let mut pending = VecDeque::from([root_id.to_string()]);
    let mut visited = BTreeSet::new();
    while let Some(node_id) = pending.pop_front() {
        if !visited.insert(node_id.clone()) {
            continue;
        }
        let node = document.nodes.get(&node_id)?;
        for child in &node.children {
            pending.push_back(child.child.clone());
        }
    }
    Some(visited)
}

fn widget_slug(component_name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = true;
    let mut previous_was_lowercase = false;
    for ch in component_name.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                if !previous_was_separator && previous_was_lowercase {
                    slug.push('_');
                }
                slug.push(ch.to_ascii_lowercase());
                previous_was_separator = false;
                previous_was_lowercase = false;
            } else {
                slug.push(ch.to_ascii_lowercase());
                previous_was_separator = false;
                previous_was_lowercase = ch.is_ascii_lowercase();
            }
        } else if !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
            previous_was_lowercase = false;
        }
    }
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "widget".to_string()
    } else {
        trimmed.to_string()
    }
}
