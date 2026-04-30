use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetLoader, UiBindingRef,
    UiChildMount, UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema,
    UiNodeDefinition, UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};

pub(crate) fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) fn load_test_ui_asset(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    UiAssetLoader::load_toml_str(source).or_else(|error| {
        if looks_like_flat_ui_asset_source(source) {
            // Keep flat fixture migration isolated to editor tests.
            let migrated = migrate_flat_ui_asset_fixture_toml_str(source)?;
            return UiAssetLoader::load_toml_str(&migrated);
        }

        Err(error)
    })
}

pub(crate) fn write_test_ui_asset(
    path: impl AsRef<Path>,
    source: &str,
) -> Result<(), UiAssetError> {
    let document = load_test_ui_asset(source)?;
    let tree_source = toml::to_string_pretty(&document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    fs::write(path, tree_source).map_err(|error| UiAssetError::Io(error.to_string()))
}

fn looks_like_flat_ui_asset_source(source: &str) -> bool {
    source.contains("\n[nodes.")
        || source.contains("\n[root]\nnode =")
        || source.contains("\r\n[root]\r\nnode =")
        || source.contains("\n[components.")
            && (source.contains("\nroot = \"") || source.contains("\r\nroot = \""))
}

fn migrate_flat_ui_asset_fixture_toml_str(input: &str) -> Result<String, UiAssetError> {
    let flat_fixture: FlatUiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    let migrated = flat_fixture.into_tree_document()?;
    toml::to_string_pretty(&migrated).map_err(|error| UiAssetError::ParseToml(error.to_string()))
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
        slot_name: node.slot_name.clone(),
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: node.params.clone(),
        props: node.props.clone(),
        layout: node.layout.clone(),
        bindings: node.bindings.clone(),
        style_overrides: node.style_overrides.clone(),
        children,
    })
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
struct FlatUiAssetDocument {
    asset: UiAssetHeader,
    #[serde(default)]
    imports: UiAssetImports,
    #[serde(default)]
    tokens: BTreeMap<String, Value>,
    #[serde(default)]
    root: Option<FlatUiAssetRoot>,
    #[serde(default)]
    nodes: BTreeMap<String, FlatUiNodeDefinition>,
    #[serde(default)]
    components: BTreeMap<String, FlatUiComponentDefinition>,
    #[serde(default)]
    stylesheets: Vec<UiStyleSheet>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
struct FlatUiAssetRoot {
    node: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiNodeDefinition {
    #[serde(default)]
    kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    widget_type: Option<String>,
    #[serde(default)]
    component: Option<String>,
    #[serde(default)]
    component_ref: Option<String>,
    #[serde(default)]
    slot_name: Option<String>,
    #[serde(default)]
    control_id: Option<String>,
    #[serde(default)]
    classes: Vec<String>,
    #[serde(default)]
    params: BTreeMap<String, Value>,
    #[serde(default)]
    props: BTreeMap<String, Value>,
    #[serde(default)]
    layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    bindings: Vec<UiBindingRef>,
    #[serde(default)]
    style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    children: Vec<FlatUiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiChildMount {
    child: String,
    #[serde(default)]
    mount: Option<String>,
    #[serde(default)]
    slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
struct FlatUiComponentDefinition {
    root: String,
    #[serde(default)]
    style_scope: UiStyleScope,
    #[serde(default)]
    params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    slots: BTreeMap<String, UiNamedSlotSchema>,
}
