use std::collections::BTreeMap;

use crate::ui::UiDesignerSelectionModel;
use toml::Value;
use zircon_runtime::ui::template::{
    UiAssetRoot, UiBindingRef, UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind,
};
use zircon_runtime::ui::{template::UiStyleRule, template::UiStyleSheet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAssetEditorTreeEditKind {
    DocumentEdit,
    InsertPaletteItem,
    MoveNode,
    ReparentNode,
    WrapNode,
    UnwrapNode,
    ConvertToReference,
    ExtractComponent,
    PromoteToExternalWidget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAssetEditorTreeEdit {
    Generic {
        kind: UiAssetEditorTreeEditKind,
    },
    InsertPaletteItem {
        node_id: String,
        parent_node_id: Option<String>,
        palette_item_label: String,
        insert_mode: String,
    },
    MoveNode {
        node_id: String,
        direction: String,
    },
    ReparentNode {
        node_id: String,
        parent_node_id: Option<String>,
        direction: String,
    },
    WrapNode {
        node_id: String,
        wrapper_node_id: String,
        wrapper_widget_type: String,
    },
    UnwrapNode {
        wrapper_node_id: String,
        child_node_id: String,
    },
    ConvertToReference {
        node_id: String,
        component_ref: String,
    },
    ExtractComponent {
        node_id: String,
        component_name: String,
        component_root_id: String,
    },
    PromoteToExternalWidget {
        source_component_name: String,
        asset_id: String,
        component_name: String,
        document_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAssetEditorInverseTreeEdit {
    RemoveNode {
        node_id: String,
        parent_node_id: Option<String>,
    },
    MoveNode {
        node_id: String,
        direction: String,
    },
    ReparentNode {
        node_id: String,
        parent_node_id: Option<String>,
        direction: String,
    },
    WrapNode {
        node_id: String,
        wrapper_node_id: String,
        wrapper_widget_type: String,
    },
    UnwrapNode {
        wrapper_node_id: String,
        child_node_id: String,
    },
    RestoreNodeDefinition {
        node_id: String,
        kind: UiNodeDefinitionKind,
        widget_type: Option<String>,
        component: Option<String>,
        component_ref: Option<String>,
    },
    InlineExtractedComponent {
        node_id: String,
        component_name: String,
        component_root_id: String,
    },
    RestorePromotedComponent {
        source_component_name: String,
        asset_id: String,
        component_name: String,
        document_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiAssetEditorDocumentReplayCommand {
    SetWidgetImports {
        references: Vec<String>,
    },
    InsertWidgetImport {
        index: usize,
        reference: String,
    },
    RemoveWidgetImport {
        index: usize,
        reference: String,
    },
    MoveWidgetImport {
        from_index: usize,
        to_index: usize,
        reference: String,
    },
    SetRoot {
        root: Option<UiAssetRoot>,
    },
    UpsertNode {
        node_id: String,
        node: UiNodeDefinition,
    },
    RemoveNode {
        node_id: String,
    },
    UpsertComponent {
        component_name: String,
        component: UiComponentDefinition,
    },
    RemoveComponent {
        component_name: String,
    },
    SetNodeBindings {
        node_id: String,
        bindings: Vec<UiBindingRef>,
    },
    SetStyleImports {
        references: Vec<String>,
    },
    InsertStyleImport {
        index: usize,
        reference: String,
    },
    RemoveStyleImport {
        index: usize,
        reference: String,
    },
    MoveStyleImport {
        from_index: usize,
        to_index: usize,
        reference: String,
    },
    SetStyleTokens {
        tokens: BTreeMap<String, Value>,
    },
    UpsertStyleToken {
        token_name: String,
        value: Value,
    },
    RemoveStyleToken {
        token_name: String,
    },
    SetStyleSheets {
        stylesheets: Vec<UiStyleSheet>,
    },
    InsertStyleSheet {
        index: usize,
        stylesheet_id: String,
        stylesheet: Option<UiStyleSheet>,
    },
    RemoveStyleSheet {
        index: usize,
        stylesheet_id: String,
    },
    ReplaceStyleSheet {
        index: usize,
        stylesheet_id: String,
        stylesheet: UiStyleSheet,
    },
    MoveStyleSheet {
        from_index: usize,
        to_index: usize,
        stylesheet_id: String,
    },
    InsertStyleRule {
        stylesheet_index: usize,
        index: usize,
        selector: String,
        rule: Option<UiStyleRule>,
    },
    RemoveStyleRule {
        stylesheet_index: usize,
        index: usize,
        selector: String,
    },
    MoveStyleRule {
        stylesheet_index: usize,
        from_index: usize,
        to_index: usize,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetEditorDocumentReplayBundle {
    pub undo: Vec<UiAssetEditorDocumentReplayCommand>,
    pub redo: Vec<UiAssetEditorDocumentReplayCommand>,
}

impl UiAssetEditorTreeEdit {
    pub fn generic(kind: UiAssetEditorTreeEditKind) -> Self {
        Self::Generic { kind }
    }

    pub fn kind(&self) -> UiAssetEditorTreeEditKind {
        match self {
            Self::Generic { kind } => *kind,
            Self::InsertPaletteItem { .. } => UiAssetEditorTreeEditKind::InsertPaletteItem,
            Self::MoveNode { .. } => UiAssetEditorTreeEditKind::MoveNode,
            Self::ReparentNode { .. } => UiAssetEditorTreeEditKind::ReparentNode,
            Self::WrapNode { .. } => UiAssetEditorTreeEditKind::WrapNode,
            Self::UnwrapNode { .. } => UiAssetEditorTreeEditKind::UnwrapNode,
            Self::ConvertToReference { .. } => UiAssetEditorTreeEditKind::ConvertToReference,
            Self::ExtractComponent { .. } => UiAssetEditorTreeEditKind::ExtractComponent,
            Self::PromoteToExternalWidget { .. } => {
                UiAssetEditorTreeEditKind::PromoteToExternalWidget
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiAssetEditorCommand {
    EditSource {
        next_source: String,
    },
    TreeEdit {
        edit: UiAssetEditorTreeEdit,
        label: String,
        next_source: String,
        next_selection: Option<UiDesignerSelectionModel>,
        document_replay: Option<UiAssetEditorDocumentReplayBundle>,
    },
}

impl UiAssetEditorCommand {
    pub fn edit_source(next_source: impl Into<String>) -> Self {
        Self::EditSource {
            next_source: next_source.into(),
        }
    }

    pub fn tree_edit(
        kind: UiAssetEditorTreeEditKind,
        label: impl Into<String>,
        next_source: impl Into<String>,
    ) -> Self {
        Self::tree_edit_structured(UiAssetEditorTreeEdit::generic(kind), label, next_source)
    }

    pub fn tree_edit_structured(
        edit: UiAssetEditorTreeEdit,
        label: impl Into<String>,
        next_source: impl Into<String>,
    ) -> Self {
        Self::TreeEdit {
            edit,
            label: label.into(),
            next_source: next_source.into(),
            next_selection: None,
            document_replay: None,
        }
    }

    pub fn tree_edit_with_selection(
        kind: UiAssetEditorTreeEditKind,
        label: impl Into<String>,
        next_source: impl Into<String>,
        next_selection: UiDesignerSelectionModel,
    ) -> Self {
        Self::tree_edit_structured_with_selection(
            UiAssetEditorTreeEdit::generic(kind),
            label,
            next_source,
            next_selection,
        )
    }

    pub fn tree_edit_structured_with_selection(
        edit: UiAssetEditorTreeEdit,
        label: impl Into<String>,
        next_source: impl Into<String>,
        next_selection: UiDesignerSelectionModel,
    ) -> Self {
        Self::TreeEdit {
            edit,
            label: label.into(),
            next_source: next_source.into(),
            next_selection: Some(next_selection),
            document_replay: None,
        }
    }

    pub fn with_document_replay(mut self, replay: UiAssetEditorDocumentReplayBundle) -> Self {
        if let Self::TreeEdit {
            document_replay, ..
        } = &mut self
        {
            *document_replay = Some(replay);
        }
        self
    }

    pub fn next_source(&self) -> &str {
        match self {
            Self::EditSource { next_source } | Self::TreeEdit { next_source, .. } => next_source,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::EditSource { .. } => "Source Edit",
            Self::TreeEdit { label, .. } => label,
        }
    }

    pub fn next_selection(&self) -> Option<&UiDesignerSelectionModel> {
        match self {
            Self::EditSource { .. } => None,
            Self::TreeEdit { next_selection, .. } => next_selection.as_ref(),
        }
    }

    pub fn structured_tree_edit(&self) -> Option<&UiAssetEditorTreeEdit> {
        match self {
            Self::EditSource { .. } => None,
            Self::TreeEdit { edit, .. } => Some(edit),
        }
    }

    pub fn document_replay(&self) -> Option<&UiAssetEditorDocumentReplayBundle> {
        match self {
            Self::EditSource { .. } => None,
            Self::TreeEdit {
                document_replay, ..
            } => document_replay.as_ref(),
        }
    }

    pub fn tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.structured_tree_edit().map(UiAssetEditorTreeEdit::kind)
    }
}
