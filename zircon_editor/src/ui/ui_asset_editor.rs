use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_ui::UiAssetKind;

use crate::ActivityWindowDescriptor;

pub const UI_ASSET_EDITOR_WINDOW_ID: &str = "editor.ui_asset";
pub const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID: &str =
    "res://ui/editor/ui_asset_editor.ui.toml";
pub const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID: &str = "editor.ui_asset_editor";
pub const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID: &str =
    "res://ui/editor/editor_widgets.ui.toml";
pub const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorToolbarButton";
pub const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorSectionCard";
pub const UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID: &str =
    "res://ui/theme/editor_base.ui.toml";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetEditorMode {
    #[default]
    Design,
    Split,
    Source,
    Preview,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAssetPreviewPreset {
    #[default]
    EditorDocked,
    EditorFloating,
    GameHud,
    Dialog,
}

impl UiAssetPreviewPreset {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EditorDocked => "Editor Docked",
            Self::EditorFloating => "Editor Floating",
            Self::GameHud => "Game HUD",
            Self::Dialog => "Dialog",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetEditorRoute {
    pub asset_id: String,
    pub asset_kind: UiAssetKind,
    pub mode: UiAssetEditorMode,
    #[serde(default)]
    pub preview_preset: UiAssetPreviewPreset,
}

impl UiAssetEditorRoute {
    pub fn new(
        asset_id: impl Into<String>,
        asset_kind: UiAssetKind,
        mode: UiAssetEditorMode,
    ) -> Self {
        Self {
            asset_id: asset_id.into(),
            asset_kind,
            mode,
            preview_preset: UiAssetPreviewPreset::default(),
        }
    }

    pub const fn window_id(&self) -> &'static str {
        UI_ASSET_EDITOR_WINDOW_ID
    }
}

pub fn ui_asset_editor_window_descriptor() -> ActivityWindowDescriptor {
    ActivityWindowDescriptor::new(
        UI_ASSET_EDITOR_WINDOW_ID,
        "UI Asset Editor",
        "albums-outline",
    )
    .with_multi_instance(true)
    .with_supports_document_tab(true)
    .with_supports_exclusive_page(true)
    .with_supports_floating_window(true)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDesignerSelectionModel {
    pub primary_node_id: Option<String>,
    #[serde(default)]
    pub sibling_node_ids: Vec<String>,
    pub parent_node_id: Option<String>,
    pub mount: Option<String>,
}

impl UiDesignerSelectionModel {
    pub fn single(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        Self {
            primary_node_id: Some(node_id.clone()),
            sibling_node_ids: vec![node_id],
            parent_node_id: None,
            mount: None,
        }
    }

    pub fn with_parent(mut self, node_id: impl Into<String>) -> Self {
        self.parent_node_id = Some(node_id.into());
        self
    }

    pub fn with_mount(mut self, mount: impl Into<String>) -> Self {
        self.mount = Some(mount.into());
        self
    }

    pub fn with_sibling(mut self, node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        if !self
            .sibling_node_ids
            .iter()
            .any(|existing| existing == &node_id)
        {
            self.sibling_node_ids.push(node_id);
        }
        self
    }

    pub fn is_multi_select(&self) -> bool {
        self.sibling_node_ids.len() > 1
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMatchedStyleRuleReflection {
    pub origin_id: String,
    pub selector: String,
    pub specificity: usize,
    pub source_order: usize,
}

impl UiMatchedStyleRuleReflection {
    pub fn new(
        origin_id: impl Into<String>,
        selector: impl Into<String>,
        specificity: usize,
        source_order: usize,
    ) -> Self {
        Self {
            origin_id: origin_id.into(),
            selector: selector.into(),
            specificity,
            source_order,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStyleInspectorReflectionModel {
    pub selected_node_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub active_pseudo_states: Vec<String>,
    #[serde(default)]
    pub inline_overrides: BTreeMap<String, Value>,
    #[serde(default)]
    pub matched_rules: Vec<UiMatchedStyleRuleReflection>,
}

impl UiStyleInspectorReflectionModel {
    pub fn for_node(node_id: impl Into<String>) -> Self {
        Self {
            selected_node_id: Some(node_id.into()),
            classes: Vec::new(),
            active_pseudo_states: Vec::new(),
            inline_overrides: BTreeMap::new(),
            matched_rules: Vec::new(),
        }
    }

    pub fn with_class(mut self, class_name: impl Into<String>) -> Self {
        let class_name = class_name.into();
        if !self.classes.iter().any(|existing| existing == &class_name) {
            self.classes.push(class_name);
        }
        self
    }

    pub fn with_active_pseudo_state(mut self, state: impl Into<String>) -> Self {
        let state = state.into();
        if !self
            .active_pseudo_states
            .iter()
            .any(|existing| existing == &state)
        {
            self.active_pseudo_states.push(state);
        }
        self
    }

    pub fn with_inline_override(mut self, path: impl Into<String>, value: Value) -> Self {
        let _ = self.inline_overrides.insert(path.into(), value);
        self
    }

    pub fn with_matched_rule(mut self, rule: UiMatchedStyleRuleReflection) -> Self {
        self.matched_rules.push(rule);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorReflectionModel {
    pub route: UiAssetEditorRoute,
    pub display_name: String,
    pub source_dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub preview_available: bool,
    pub last_error: Option<String>,
    pub selection: UiDesignerSelectionModel,
    pub style_inspector: UiStyleInspectorReflectionModel,
}

impl UiAssetEditorReflectionModel {
    pub fn new(route: UiAssetEditorRoute, display_name: impl Into<String>) -> Self {
        Self {
            route,
            display_name: display_name.into(),
            source_dirty: false,
            can_undo: false,
            can_redo: false,
            preview_available: false,
            last_error: None,
            selection: UiDesignerSelectionModel::default(),
            style_inspector: UiStyleInspectorReflectionModel::default(),
        }
    }

    pub fn with_source_dirty(mut self, dirty: bool) -> Self {
        self.source_dirty = dirty;
        self
    }

    pub fn with_undo_state(mut self, can_undo: bool, can_redo: bool) -> Self {
        self.can_undo = can_undo;
        self.can_redo = can_redo;
        self
    }

    pub fn with_preview_available(mut self, available: bool) -> Self {
        self.preview_available = available;
        self
    }

    pub fn with_last_error(mut self, error: impl Into<String>) -> Self {
        self.last_error = Some(error.into());
        self
    }

    pub fn with_selection(mut self, selection: UiDesignerSelectionModel) -> Self {
        self.selection = selection;
        self
    }

    pub fn with_style_inspector(mut self, inspector: UiStyleInspectorReflectionModel) -> Self {
        self.style_inspector = inspector;
        self
    }
}
