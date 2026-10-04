use super::*;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

pub(in crate::ui::asset_editor::session) fn parse_ui_asset_source(
    source: &str,
) -> Result<UiAssetDocument, UiAssetError> {
    EditorTemplateRuntimeService.parse_document_source(source)
}

pub(crate) fn v2_document_to_legacy_projection_document(
    document: &UiV2AssetDocument,
) -> Result<UiAssetDocument, UiAssetEditorSessionError> {
    let root = document
        .root_node_id()
        .map(|root| v2_node_tree_to_legacy_projection_node(document, root))
        .transpose()?;
    let components = document
        .components
        .iter()
        .map(|(component_name, component)| {
            let root = v2_node_tree_to_legacy_projection_node(document, &component.root)?;
            Ok((
                component_name.clone(),
                UiComponentDefinition {
                    root,
                    style_scope: component.style_scope,
                    contract: component.contract.clone(),
                    params: component.params.clone(),
                    slots: component.slots.clone(),
                },
            ))
        })
        .collect::<Result<_, UiAssetEditorSessionError>>()?;

    Ok(UiAssetDocument {
        asset: UiAssetHeader {
            kind: legacy_asset_kind(document.asset.kind),
            id: document.asset.id.clone(),
            version: document.asset.version,
            display_name: document.asset.display_name.clone(),
        },
        imports: UiAssetImports {
            widgets: document.imports.widgets.clone(),
            styles: document.imports.styles.clone(),
            resources: document.imports.resources.clone(),
        },
        tokens: document.tokens.clone(),
        root,
        components,
        stylesheets: document
            .stylesheets
            .iter()
            .map(|stylesheet| UiStyleSheet {
                id: stylesheet.id.clone(),
                rules: stylesheet
                    .rules
                    .iter()
                    .map(|rule| UiStyleRule {
                        id: rule.id.clone(),
                        selector: rule.selector.clone(),
                        set: legacy_style_block(&rule.set),
                    })
                    .collect(),
            })
            .collect(),
    })
}

pub(crate) fn legacy_projection_document_to_v2_document(
    document: &UiAssetDocument,
    previous: Option<&UiV2AssetDocument>,
) -> Result<UiV2AssetDocument, UiAssetEditorSessionError> {
    let mut nodes = BTreeMap::new();
    if let Some(root) = &document.root {
        flatten_legacy_projection_nodes_into(&mut nodes, root, previous)?;
    }
    let components = document
        .components
        .iter()
        .map(|(component_name, component)| {
            flatten_legacy_projection_nodes_into(&mut nodes, &component.root, previous)?;
            let previous_component =
                previous.and_then(|document| document.components.get(component_name));
            Ok((
                component_name.clone(),
                zircon_runtime_interface::ui::v2::UiV2ComponentDefinition {
                    root: component.root.node_id.clone(),
                    style_scope: component.style_scope,
                    contract: component.contract.clone(),
                    params: component.params.clone(),
                    slots: component.slots.clone(),
                    default_classes: previous_component
                        .map(|component| component.default_classes.clone())
                        .unwrap_or_default(),
                },
            ))
        })
        .collect::<Result<_, UiAssetEditorSessionError>>()?;

    Ok(UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: v2_asset_kind(document.asset.kind),
            id: document.asset.id.clone(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: document.asset.display_name.clone(),
        },
        imports: document.imports.clone(),
        tokens: document.tokens.clone(),
        root: (document.asset.kind == UiAssetKind::Layout)
            .then(|| document.root.as_ref())
            .flatten()
            .map(|root| UiV2Root {
                node: root.node_id.clone(),
            }),
        nodes,
        components,
        stylesheets: document
            .stylesheets
            .iter()
            .map(|stylesheet| UiV2StyleSheet {
                id: stylesheet.id.clone(),
                rules: stylesheet
                    .rules
                    .iter()
                    .map(|rule| UiV2StyleRule {
                        id: rule.id.clone(),
                        selector: rule.selector.clone(),
                        set: v2_style_block(&rule.set),
                    })
                    .collect(),
            })
            .collect(),
    })
}

pub(in crate::ui::asset_editor) fn serialize_v2_projection_document(
    document: &UiAssetDocument,
    previous: Option<&UiV2AssetDocument>,
) -> Result<String, UiAssetEditorSessionError> {
    let document = legacy_projection_document_to_v2_document(document, previous)?;
    let source = toml::to_string_pretty(&document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    UiZuiAssetLoader::load_zui_str(&source)?;
    Ok(source)
}

pub(super) fn flatten_legacy_projection_nodes_into(
    nodes: &mut BTreeMap<String, UiV2NodeDefinition>,
    root: &UiNodeDefinition,
    previous: Option<&UiV2AssetDocument>,
) -> Result<(), UiAssetEditorSessionError> {
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        if node.node_id.trim().is_empty() {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        let children = node
            .children
            .iter()
            .map(|child| {
                let mut slot = child.slot.clone();
                if let Some(mount) = child.mount.as_ref() {
                    slot.entry("name".to_string())
                        .or_insert_with(|| toml::Value::String(mount.clone()));
                }
                UiV2ChildMount {
                    node: child.node.node_id.clone(),
                    slot,
                }
            })
            .collect::<Vec<_>>();
        let component = legacy_projection_component_name(&node);
        let mut props = node.props;
        if node.kind == UiNodeDefinitionKind::Slot {
            if let Some(slot_name) = node.slot_name.as_ref() {
                props
                    .entry("name".to_string())
                    .or_insert_with(|| toml::Value::String(slot_name.clone()));
            }
        }
        let previous_node = previous.and_then(|document| document.nodes.get(&node.node_id));
        let next = UiV2NodeDefinition {
            component,
            control_id: node.control_id,
            pixel_snapping: previous_node.and_then(|node| node.pixel_snapping),
            params: node.params,
            classes: node.classes,
            props,
            state: previous_node
                .map(|node| node.state.clone())
                .unwrap_or_default(),
            layout: node.layout,
            repeat: previous_node.and_then(|node| node.repeat.clone()),
            style: v2_style_block(&node.style_overrides),
            slots: previous_node
                .map(|node| node.slots.clone())
                .unwrap_or_default(),
            events: node.bindings,
            widget: node.widget,
            children,
        };
        if let Some(existing) = nodes.get(&node.node_id) {
            if existing != &next {
                return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
            }
            continue;
        }
        for child in node.children.iter().rev() {
            stack.push(child.node.clone());
        }
        let _ = nodes.insert(node.node_id, next);
    }
    Ok(())
}

pub(super) fn v2_node_tree_to_legacy_projection_node(
    document: &UiV2AssetDocument,
    root_node_id: &str,
) -> Result<UiNodeDefinition, UiAssetEditorSessionError> {
    let mut stack = vec![V2ProjectionBuildFrame::Enter {
        node_id: root_node_id.to_string(),
        slot: BTreeMap::new(),
    }];
    let mut built_nodes: Vec<(UiNodeDefinition, BTreeMap<String, toml::Value>)> = Vec::new();
    let mut active = BTreeSet::new();

    while let Some(frame) = stack.pop() {
        match frame {
            V2ProjectionBuildFrame::Enter { node_id, slot } => {
                let node = document
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| missing_v2_node(document, &node_id))?;
                if !active.insert(node_id.clone()) {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!("ui v2 authoring projection cycle at node {node_id}"),
                    }
                    .into());
                }
                stack.push(V2ProjectionBuildFrame::Exit {
                    node_id: node_id.clone(),
                    slot,
                    child_count: node.children.len(),
                });
                for child in node.children.iter().rev() {
                    stack.push(V2ProjectionBuildFrame::Enter {
                        node_id: child.node.clone(),
                        slot: child.slot.clone(),
                    });
                }
            }
            V2ProjectionBuildFrame::Exit {
                node_id,
                slot,
                child_count,
            } => {
                let split_at = built_nodes.len().checked_sub(child_count).ok_or_else(|| {
                    UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!(
                            "ui v2 authoring projection lost child state for node {node_id}"
                        ),
                    }
                })?;
                let children = built_nodes
                    .split_off(split_at)
                    .into_iter()
                    .map(|(node, slot)| UiChildMount {
                        mount: child_mount_name(&slot).map(str::to_string),
                        slot,
                        node,
                    })
                    .collect::<Vec<_>>();
                let source = document
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| missing_v2_node(document, &node_id))?;
                let node = legacy_projection_node(document, &node_id, source, children);
                active.remove(&node_id);
                built_nodes.push((node, slot));
            }
        }
    }

    built_nodes
        .pop()
        .map(|(node, _)| node)
        .ok_or_else(|| missing_v2_node(document, root_node_id).into())
}

fn legacy_projection_component_name(node: &UiNodeDefinition) -> String {
    match node.kind {
        UiNodeDefinitionKind::Reference => node.component_ref.clone(),
        UiNodeDefinitionKind::Component => node.component.clone(),
        UiNodeDefinitionKind::Slot => Some("Slot".to_string()),
        UiNodeDefinitionKind::Native => node.widget_type.clone().or_else(|| node.component.clone()),
    }
    .unwrap_or_else(|| "Space".to_string())
}

pub(super) fn missing_v2_node(
    document: &UiV2AssetDocument,
    node_id: &str,
) -> UiAssetEditorSessionError {
    UiV2AssetError::MissingNode {
        asset_id: document.asset.id.clone(),
        node_id: node_id.to_string(),
    }
    .into()
}

pub(super) fn legacy_projection_node(
    document: &UiV2AssetDocument,
    node_id: &str,
    source: &UiV2NodeDefinition,
    children: Vec<UiChildMount>,
) -> UiNodeDefinition {
    let is_reference = source.component.contains("#") || source.component.contains("://");
    let is_local_component = document.components.contains_key(&source.component);
    let is_slot = source.component == "Slot";
    let (kind, widget_type, component, component_ref, slot_name) = if is_reference {
        (
            UiNodeDefinitionKind::Reference,
            None,
            None,
            Some(source.component.clone()),
            None,
        )
    } else if is_local_component {
        (
            UiNodeDefinitionKind::Component,
            None,
            Some(source.component.clone()),
            None,
            None,
        )
    } else if is_slot {
        (
            UiNodeDefinitionKind::Slot,
            None,
            Some(source.component.clone()),
            None,
            source
                .props
                .get("name")
                .or_else(|| source.props.get("slot_name"))
                .and_then(toml::Value::as_str)
                .map(str::to_string),
        )
    } else {
        (
            UiNodeDefinitionKind::Native,
            Some(source.component.clone()),
            Some(source.component.clone()),
            None,
            None,
        )
    };
    UiNodeDefinition {
        node_id: node_id.to_string(),
        kind,
        widget_type,
        component,
        component_ref,
        component_api_version: None,
        slot_name,
        control_id: source.control_id.clone(),
        classes: source.classes.clone(),
        params: source.params.clone(),
        props: source.props.clone(),
        layout: source.layout.clone(),
        bindings: source.events.clone(),
        style_overrides: legacy_style_block(&source.style),
        focus: None,
        navigation: None,
        picking: None,
        a11y: None,
        widget: source.widget.clone(),
        children,
    }
}

fn child_mount_name(slot: &BTreeMap<String, toml::Value>) -> Option<&str> {
    slot.get("name")
        .or_else(|| slot.get("slot_name"))
        .and_then(toml::Value::as_str)
}

pub(super) fn legacy_style_block(source: &UiV2StyleDeclarationBlock) -> UiStyleDeclarationBlock {
    UiStyleDeclarationBlock {
        self_values: source.self_values.clone(),
        slot: source.slot.clone(),
    }
}

pub(super) fn v2_style_block(source: &UiStyleDeclarationBlock) -> UiV2StyleDeclarationBlock {
    UiV2StyleDeclarationBlock {
        self_values: source.self_values.clone(),
        slot: source.slot.clone(),
    }
}

pub(super) fn legacy_asset_kind(kind: UiV2AssetKind) -> UiAssetKind {
    match kind {
        UiV2AssetKind::View => UiAssetKind::Layout,
        UiV2AssetKind::Component => UiAssetKind::Widget,
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => UiAssetKind::Style,
    }
}

pub(super) fn v2_asset_kind(kind: UiAssetKind) -> UiV2AssetKind {
    match kind {
        UiAssetKind::Layout => UiV2AssetKind::View,
        UiAssetKind::Widget => UiV2AssetKind::Component,
        UiAssetKind::Style => UiV2AssetKind::Style,
    }
}

enum V2ProjectionBuildFrame {
    Enter {
        node_id: String,
        slot: BTreeMap<String, toml::Value>,
    },
    Exit {
        node_id: String,
        slot: BTreeMap<String, toml::Value>,
        child_count: usize,
    },
}

pub(in crate::ui::asset_editor::session) fn compiled_resource_report(
    compiled: Option<&zircon_runtime::ui::template::UiCompiledDocument>,
) -> (Vec<UiResourceDependency>, Vec<UiResourceDiagnostic>) {
    compiled
        .map(|compiled| {
            (
                compiled.resource_dependencies().to_vec(),
                compiled.resource_diagnostics().to_vec(),
            )
        })
        .unwrap_or_default()
}

pub(super) fn reconcile_selected_palette_index<T>(
    items: &[T],
    current: Option<usize>,
) -> Option<usize> {
    match (current, items.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

#[cfg(test)]
#[path = "tests/v2_projection.rs"]
mod tests;

pub(in crate::ui::asset_editor::session) fn structured_compile_diagnostics(
    document: &UiAssetDocument,
    imports: &UiAssetCompilerImports,
) -> Vec<crate::ui::asset_editor::UiAssetEditorDiagnostic> {
    let mut diagnostics = structured_contract_diagnostics(document, imports);
    diagnostics.extend(
        EditorTemplateRuntimeService
            .collect_binding_report(document)
            .diagnostics
            .into_iter()
            .map(map_binding_diagnostic),
    );
    diagnostics.extend(
        EditorTemplateRuntimeService
            .collect_localization_report(document)
            .diagnostics
            .into_iter()
            .map(map_localization_diagnostic),
    );
    diagnostics
}

pub(super) fn structured_contract_diagnostics(
    document: &UiAssetDocument,
    imports: &UiAssetCompilerImports,
) -> Vec<crate::ui::asset_editor::UiAssetEditorDiagnostic> {
    match component_contract_diagnostic(document, &imports.widgets, &imports.styles) {
        Ok(diagnostic) => diagnostic
            .into_iter()
            .map(map_component_contract_diagnostic)
            .collect(),
        Err(_) => Vec::new(),
    }
}
