use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime::ui::template::UiAssetLoader;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::accessibility::UiAccessibilityContract;
use zircon_runtime_interface::ui::focus::UiFocusContract;
use zircon_runtime_interface::ui::navigation::UiNavigationContract;
use zircon_runtime_interface::ui::picking::UiPickPolicy;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiBindingRef, UiChildMount,
    UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiNodeDefinition,
    UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};
use zircon_runtime_interface::ui::widget::UiWidgetContract;

/// Serializes process-global test configuration without cascading mutex poison.
pub(crate) struct TestEnvironmentLock {
    inner: Mutex<()>,
}

impl TestEnvironmentLock {
    fn new() -> Self {
        Self {
            inner: Mutex::new(()),
        }
    }

    pub(crate) fn lock(&self) -> std::sync::LockResult<MutexGuard<'_, ()>> {
        Ok(match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.inner.clear_poison();
                poisoned.into_inner()
            }
        })
    }
}

pub(crate) fn env_lock() -> &'static TestEnvironmentLock {
    static LOCK: OnceLock<TestEnvironmentLock> = OnceLock::new();
    LOCK.get_or_init(TestEnvironmentLock::new)
}

pub(crate) fn load_test_ui_asset(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    UiAssetLoader::load_toml_str(source).or_else(|error| {
        if let Ok(document) = UiZuiAssetLoader::load_zui_str(source) {
            return crate::ui::asset_editor::project_v2_document_to_authoring(&document).map_err(
                |projection_error| UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: projection_error.to_string(),
                },
            );
        }

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
    let path = path.as_ref();
    let tree_source = if path.extension().and_then(|extension| extension.to_str()) == Some("zui") {
        let v2_document = crate::ui::asset_editor::project_authoring_document_to_v2(&document)
            .map_err(|error| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: error.to_string(),
            })?;
        toml::to_string_pretty(&v2_document)
    } else {
        toml::to_string_pretty(&document)
    }
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

#[cfg(test)]
mod env_lock_tests {
    use super::env_lock;

    #[test]
    fn shared_test_environment_lock_recovers_after_poison() {
        let lock = env_lock();
        let poisoned = std::panic::catch_unwind(|| {
            let _guard = lock.lock().expect("initial environment lock");
            panic!("poison shared test environment lock");
        });
        assert!(poisoned.is_err());

        let recovered = lock.lock();
        assert!(recovered.is_ok(), "environment lock must recover poison");
    }
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
    focus: Option<UiFocusContract>,
    #[serde(default)]
    navigation: Option<UiNavigationContract>,
    #[serde(default)]
    picking: Option<UiPickPolicy>,
    #[serde(default)]
    a11y: Option<UiAccessibilityContract>,
    #[serde(default)]
    widget: Option<UiWidgetContract>,
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
