use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use toml::Value;

use zircon_runtime_interface::ui::accessibility::UiAccessibilityContract;
use zircon_runtime_interface::ui::focus::UiFocusContract;
use zircon_runtime_interface::ui::navigation::UiNavigationContract;
use zircon_runtime_interface::ui::picking::UiPickPolicy;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiBindingRef, UiChildMount,
    UiComponentDefinition, UiComponentParamSchema, UiComponentPrototype, UiDocumentPrototype,
    UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind, UiNodePrototype,
    UiPrototypeChildMount, UiPrototypeNodeHandle, UiRawAssetPrototype, UiStyleDeclarationBlock,
    UiStylePrototype, UiStyleScope, UiStyleSheet,
};
use zircon_runtime_interface::ui::widget::UiWidgetContract;

pub(super) fn migrate_flat_toml_str(input: &str) -> Result<UiAssetDocument, UiAssetError> {
    let legacy: FlatUiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    legacy.into_tree_document()
}

pub(crate) fn load_flat_prototype_toml_str(
    input: &str,
) -> Result<UiRawAssetPrototype, UiAssetError> {
    let flat: FlatUiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    flat.into_raw_prototype()
}

impl FlatUiAssetDocument {
    fn into_tree_document(self) -> Result<UiAssetDocument, UiAssetError> {
        let root = self
            .root
            .map(|root| build_tree_node(&self.asset.id, &self.nodes, &root.node, &mut Vec::new()))
            .transpose()?;
        let components = self
            .components
            .into_iter()
            .map(|(name, component)| {
                Ok((
                    name,
                    UiComponentDefinition {
                        root: build_tree_node(
                            &self.asset.id,
                            &self.nodes,
                            &component.root,
                            &mut Vec::new(),
                        )?,
                        style_scope: component.style_scope,
                        contract: Default::default(),
                        params: component.params,
                        slots: component.slots,
                    },
                ))
            })
            .collect::<Result<_, UiAssetError>>()?;

        Ok(UiAssetDocument {
            asset: self.asset,
            imports: self.imports,
            tokens: self.tokens,
            root,
            components,
            stylesheets: self.stylesheets,
        })
    }

    fn into_raw_prototype(self) -> Result<UiRawAssetPrototype, UiAssetError> {
        let node_handles = prototype_node_handles(&self.asset.id, &self.nodes)?;
        validate_reachable_prototype_root(
            &self.asset.id,
            &self.nodes,
            &node_handles,
            self.root.as_ref().map(|root| root.node.as_str()),
        )?;
        for (component_name, component) in &self.components {
            validate_reachable_prototype_root(
                &self.asset.id,
                &self.nodes,
                &node_handles,
                Some(component.root.as_str()),
            )
            .map_err(|error| match error {
                UiAssetError::InvalidDocument { asset_id, detail } => {
                    UiAssetError::InvalidDocument {
                        asset_id,
                        detail: format!("component {component_name}: {detail}"),
                    }
                }
                other => other,
            })?;
        }

        let mut nodes = vec![UiNodePrototype::default(); node_handles.len()];
        for (node_id, flat_node) in &self.nodes {
            let handle = node_handles[node_id];
            nodes[handle.index()] =
                flat_node.to_node_prototype(&self.asset.id, node_id, &node_handles)?;
        }

        let root = self.root.as_ref().map(|root| node_handles[&root.node]);
        let components = self
            .components
            .into_iter()
            .map(|(name, component)| {
                Ok((
                    name,
                    UiComponentPrototype {
                        root: node_handles[&component.root],
                        style_scope: component.style_scope,
                        contract: Default::default(),
                        params: component.params,
                        slots: component.slots,
                    },
                ))
            })
            .collect::<Result<_, UiAssetError>>()?;

        Ok(UiRawAssetPrototype {
            asset: self.asset,
            imports: self.imports,
            tokens: self.tokens,
            document: UiDocumentPrototype { root, nodes },
            components,
            styles: self
                .stylesheets
                .into_iter()
                .map(|stylesheet| UiStylePrototype { stylesheet })
                .collect(),
        })
    }
}

fn prototype_node_handles(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
) -> Result<BTreeMap<String, UiPrototypeNodeHandle>, UiAssetError> {
    if nodes.len() > u32::MAX as usize {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: "flat prototype node table exceeds u32 handle capacity".to_string(),
        });
    }

    Ok(nodes
        .keys()
        .enumerate()
        .map(|(index, node_id)| (node_id.clone(), UiPrototypeNodeHandle::new(index as u32)))
        .collect())
}

fn validate_reachable_prototype_root(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_handles: &BTreeMap<String, UiPrototypeNodeHandle>,
    root: Option<&str>,
) -> Result<(), UiAssetError> {
    let Some(root) = root else {
        return Ok(());
    };
    if !node_handles.contains_key(root) {
        return Err(UiAssetError::MissingNode {
            asset_id: asset_id.to_string(),
            node_id: root.to_string(),
        });
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = vec![PrototypeVisitFrame::Enter(root.to_string())];
    while let Some(frame) = stack.pop() {
        match frame {
            PrototypeVisitFrame::Enter(node_id) => {
                if visited.contains(&node_id) {
                    continue;
                }
                if !visiting.insert(node_id.clone()) {
                    return Err(UiAssetError::InvalidDocument {
                        asset_id: asset_id.to_string(),
                        detail: format!("ui asset prototype contains a cycle at {node_id}"),
                    });
                }
                let node = nodes
                    .get(&node_id)
                    .ok_or_else(|| UiAssetError::MissingNode {
                        asset_id: asset_id.to_string(),
                        node_id: node_id.clone(),
                    })?;
                stack.push(PrototypeVisitFrame::Exit(node_id));
                for child in node.children.iter().rev() {
                    if !node_handles.contains_key(&child.child) {
                        return Err(UiAssetError::MissingNode {
                            asset_id: asset_id.to_string(),
                            node_id: child.child.clone(),
                        });
                    }
                    stack.push(PrototypeVisitFrame::Enter(child.child.clone()));
                }
            }
            PrototypeVisitFrame::Exit(node_id) => {
                let _ = visiting.remove(&node_id);
                let _ = visited.insert(node_id);
            }
        }
    }
    Ok(())
}

enum PrototypeVisitFrame {
    Enter(String),
    Exit(String),
}

fn build_tree_node(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_id: &str,
    visiting: &mut Vec<String>,
) -> Result<UiNodeDefinition, UiAssetError> {
    if visiting.iter().any(|current| current == node_id) {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("ui asset tree contains a cycle at {node_id}"),
        });
    }
    let node = nodes
        .get(node_id)
        .ok_or_else(|| UiAssetError::MissingNode {
            asset_id: asset_id.to_string(),
            node_id: node_id.to_string(),
        })?;

    visiting.push(node_id.to_string());
    let children = node
        .children
        .iter()
        .map(|child| {
            Ok(UiChildMount {
                mount: child.mount.clone(),
                slot: child.slot.clone(),
                node: build_tree_node(asset_id, nodes, &child.child, visiting)?,
            })
        })
        .collect::<Result<Vec<_>, UiAssetError>>()?;
    let _ = visiting.pop();

    Ok(UiNodeDefinition {
        node_id: node_id.to_string(),
        kind: node.kind,
        widget_type: node.widget_type.clone(),
        component: node.component.clone(),
        component_ref: node.component_ref.clone(),
        component_api_version: None,
        slot_name: node.slot_name.clone(),
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: node.params.clone(),
        props: node.props.clone(),
        layout: node.layout.clone(),
        bindings: node.bindings.clone(),
        style_overrides: node.style_overrides.clone(),
        children,
        focus: node.focus.clone(),
        navigation: node.navigation.clone(),
        picking: node.picking,
        a11y: node.a11y.clone(),
        widget: node.widget.clone(),
    })
}

impl FlatUiNodeDefinition {
    fn to_node_prototype(
        &self,
        asset_id: &str,
        node_id: &str,
        node_handles: &BTreeMap<String, UiPrototypeNodeHandle>,
    ) -> Result<UiNodePrototype, UiAssetError> {
        let children = self
            .children
            .iter()
            .map(|child| {
                let child_handle = node_handles.get(&child.child).copied().ok_or_else(|| {
                    UiAssetError::MissingNode {
                        asset_id: asset_id.to_string(),
                        node_id: child.child.clone(),
                    }
                })?;
                Ok(UiPrototypeChildMount {
                    mount: child.mount.clone(),
                    slot: child.slot.clone(),
                    child: child_handle,
                })
            })
            .collect::<Result<_, UiAssetError>>()?;

        Ok(UiNodePrototype {
            node_id: node_id.to_string(),
            kind: self.kind,
            widget_type: self.widget_type.clone(),
            component: self.component.clone(),
            component_ref: self.component_ref.clone(),
            slot_name: self.slot_name.clone(),
            control_id: self.control_id.clone(),
            classes: self.classes.clone(),
            params: self.params.clone(),
            props: self.props.clone(),
            layout: self.layout.clone(),
            bindings: self.bindings.clone(),
            style_overrides: self.style_overrides.clone(),
            focus: self.focus.clone(),
            navigation: self.navigation.clone(),
            picking: self.picking,
            a11y: self.a11y.clone(),
            widget: self.widget.clone(),
            children,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FlatUiAssetDocument {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    #[serde(default)]
    pub root: Option<FlatUiAssetRoot>,
    #[serde(default)]
    pub nodes: BTreeMap<String, FlatUiNodeDefinition>,
    #[serde(default)]
    pub components: BTreeMap<String, FlatUiComponentDefinition>,
    #[serde(default)]
    pub stylesheets: Vec<UiStyleSheet>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FlatUiAssetRoot {
    pub node: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiNodeDefinition {
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub component_ref: Option<String>,
    #[serde(default)]
    pub slot_name: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    pub focus: Option<UiFocusContract>,
    #[serde(default)]
    pub navigation: Option<UiNavigationContract>,
    #[serde(default)]
    pub picking: Option<UiPickPolicy>,
    #[serde(default)]
    pub a11y: Option<UiAccessibilityContract>,
    #[serde(default)]
    pub widget: Option<UiWidgetContract>,
    #[serde(default)]
    pub children: Vec<FlatUiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiChildMount {
    pub child: String,
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiComponentDefinition {
    pub root: String,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}
