use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use toml::Value as TomlValue;
use zircon_runtime_interface::ui::template::{
    UiComponentDefinition, UiNodeDefinition, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
};

use super::ui_asset_editor_session::UiAssetEditorSession;
use super::{
    command::{
        UiAssetEditorDocumentReplayCommand, UiAssetEditorInverseTreeEdit, UiAssetEditorTreeEdit,
    },
    undo_stack::{UiAssetEditorExternalEffect, UiAssetEditorUndoStackReplayRecord},
};

pub const UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION: u32 = 1;

const FNV_1A_64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_1A_64_PRIME: u64 = 0x100000001b3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorBugReportReplayArtifact {
    pub schema_version: u32,
    pub route: UiAssetEditorReplayArtifactRoute,
    pub initial_source: UiAssetEditorReplaySourceSummary,
    pub current_source: UiAssetEditorReplaySourceSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_source_reconstruction_error: Option<String>,
    pub selection: UiAssetEditorReplaySelectionSummary,
    pub diagnostics: Vec<String>,
    pub structured_diagnostic_count: usize,
    pub records: Vec<UiAssetEditorReplayArtifactRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorReplayArtifactRoute {
    pub asset_id: String,
    pub asset_kind: String,
    pub mode: String,
    pub preview_preset: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetEditorReplaySourceSummary {
    pub redacted: bool,
    pub byte_len: usize,
    pub line_count: usize,
    pub stable_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetEditorReplaySelectionSummary {
    pub primary_node_id: Option<String>,
    pub sibling_node_ids: Vec<String>,
    pub parent_node_id: Option<String>,
    pub mount: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorReplayArtifactRecord {
    pub sequence: usize,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_edit: Option<UiAssetEditorReplayCommandSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse_tree_edit: Option<UiAssetEditorReplayCommandSummary>,
    pub undo_document_commands: Vec<UiAssetEditorReplayCommandSummary>,
    pub redo_document_commands: Vec<UiAssetEditorReplayCommandSummary>,
    pub undo_external_effects: Vec<UiAssetEditorReplayExternalEffectSummary>,
    pub redo_external_effects: Vec<UiAssetEditorReplayExternalEffectSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorReplayCommandSummary {
    pub command_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default)]
    pub payload: BTreeMap<String, JsonValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetEditorReplayExternalEffectSummary {
    pub effect_id: String,
    pub asset_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<UiAssetEditorReplaySourceSummary>,
}

impl UiAssetEditorSession {
    pub fn export_bug_report_replay_artifact(&self) -> UiAssetEditorBugReportReplayArtifact {
        let (initial_source, initial_source_reconstruction_error) =
            reconstruct_initial_source(self);
        UiAssetEditorBugReportReplayArtifact {
            schema_version: UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION,
            route: UiAssetEditorReplayArtifactRoute {
                asset_id: self.route.asset_id.clone(),
                asset_kind: format!("{:?}", self.route.asset_kind),
                mode: format!("{:?}", self.route.mode),
                preview_preset: format!("{:?}", self.route.preview_preset),
            },
            initial_source: source_summary(&initial_source),
            current_source: source_summary(self.source_buffer.text()),
            initial_source_reconstruction_error,
            selection: selection_summary(&self.selection),
            diagnostics: self
                .diagnostics
                .iter()
                .map(|diagnostic| sanitize_text_for_artifact(diagnostic))
                .collect(),
            structured_diagnostic_count: self.structured_diagnostics.len(),
            records: self
                .undo_stack
                .replay_records()
                .into_iter()
                .map(replay_record_summary)
                .collect(),
        }
    }
}

fn reconstruct_initial_source(session: &UiAssetEditorSession) -> (String, Option<String>) {
    let mut source = session.source_buffer.text().to_string();
    let mut undo_stack = session.undo_stack.clone();
    while let Some(record) = undo_stack.undo_record() {
        if let Err(error) = record.transition.apply_to_source(&mut source) {
            return (source, Some(error.to_string()));
        }
    }
    (source, None)
}

fn replay_record_summary(
    record: UiAssetEditorUndoStackReplayRecord,
) -> UiAssetEditorReplayArtifactRecord {
    UiAssetEditorReplayArtifactRecord {
        sequence: record.sequence,
        label: record.label,
        tree_edit: record.tree_edit.as_ref().map(tree_edit_summary),
        inverse_tree_edit: record
            .inverse_tree_edit
            .as_ref()
            .map(inverse_tree_edit_summary),
        undo_document_commands: record
            .undo_document_commands
            .iter()
            .map(document_replay_command_summary)
            .collect(),
        redo_document_commands: record
            .redo_document_commands
            .iter()
            .map(document_replay_command_summary)
            .collect(),
        undo_external_effects: record
            .undo_external_effects
            .iter()
            .map(external_effect_summary)
            .collect(),
        redo_external_effects: record
            .redo_external_effects
            .iter()
            .map(external_effect_summary)
            .collect(),
    }
}

fn source_summary(source: &str) -> UiAssetEditorReplaySourceSummary {
    UiAssetEditorReplaySourceSummary {
        redacted: true,
        byte_len: source.len(),
        line_count: source.lines().count(),
        stable_hash: stable_text_hash(source),
    }
}

fn selection_summary(
    selection: &crate::ui::asset_editor::UiDesignerSelectionModel,
) -> UiAssetEditorReplaySelectionSummary {
    UiAssetEditorReplaySelectionSummary {
        primary_node_id: selection.primary_node_id.clone(),
        sibling_node_ids: selection.sibling_node_ids.clone(),
        parent_node_id: selection.parent_node_id.clone(),
        mount: selection.mount.clone(),
    }
}

fn tree_edit_summary(edit: &UiAssetEditorTreeEdit) -> UiAssetEditorReplayCommandSummary {
    match edit {
        UiAssetEditorTreeEdit::Generic { kind } => {
            command_summary("tree.generic", None, [("kind", json!(format!("{kind:?}")))])
        }
        UiAssetEditorTreeEdit::InsertPaletteItem {
            node_id,
            parent_node_id,
            palette_item_label,
            insert_mode,
        } => command_summary(
            "tree.insert_palette_item",
            Some(node_id.clone()),
            [
                ("parent_node_id", json!(parent_node_id)),
                ("palette_item_label", json!(palette_item_label)),
                ("insert_mode", json!(insert_mode)),
            ],
        ),
        UiAssetEditorTreeEdit::MoveNode { node_id, direction } => command_summary(
            "tree.move_node",
            Some(node_id.clone()),
            [("direction", json!(direction))],
        ),
        UiAssetEditorTreeEdit::ReparentNode {
            node_id,
            parent_node_id,
            direction,
        } => command_summary(
            "tree.reparent_node",
            Some(node_id.clone()),
            [
                ("parent_node_id", json!(parent_node_id)),
                ("direction", json!(direction)),
            ],
        ),
        UiAssetEditorTreeEdit::WrapNode {
            node_id,
            wrapper_node_id,
            wrapper_widget_type,
        } => command_summary(
            "tree.wrap_node",
            Some(node_id.clone()),
            [
                ("wrapper_node_id", json!(wrapper_node_id)),
                ("wrapper_widget_type", json!(wrapper_widget_type)),
            ],
        ),
        UiAssetEditorTreeEdit::UnwrapNode {
            wrapper_node_id,
            child_node_id,
        } => command_summary(
            "tree.unwrap_node",
            Some(wrapper_node_id.clone()),
            [("child_node_id", json!(child_node_id))],
        ),
        UiAssetEditorTreeEdit::ConvertToReference {
            node_id,
            component_ref,
        } => command_summary(
            "tree.convert_to_reference",
            Some(node_id.clone()),
            [("component_ref", json!(component_ref))],
        ),
        UiAssetEditorTreeEdit::ExtractComponent {
            node_id,
            component_name,
            component_root_id,
        } => command_summary(
            "tree.extract_component",
            Some(node_id.clone()),
            [
                ("component_name", json!(component_name)),
                ("component_root_id", json!(component_root_id)),
            ],
        ),
        UiAssetEditorTreeEdit::PromoteToExternalWidget {
            source_component_name,
            asset_id,
            component_name,
            document_id,
        } => command_summary(
            "tree.promote_to_external_widget",
            Some(source_component_name.clone()),
            [
                ("asset_id", json!(asset_id)),
                ("component_name", json!(component_name)),
                ("document_id", json!(document_id)),
            ],
        ),
    }
}

fn inverse_tree_edit_summary(
    edit: &UiAssetEditorInverseTreeEdit,
) -> UiAssetEditorReplayCommandSummary {
    match edit {
        UiAssetEditorInverseTreeEdit::RemoveNode {
            node_id,
            parent_node_id,
        } => command_summary(
            "inverse.remove_node",
            Some(node_id.clone()),
            [("parent_node_id", json!(parent_node_id))],
        ),
        UiAssetEditorInverseTreeEdit::MoveNode { node_id, direction } => command_summary(
            "inverse.move_node",
            Some(node_id.clone()),
            [("direction", json!(direction))],
        ),
        UiAssetEditorInverseTreeEdit::ReparentNode {
            node_id,
            parent_node_id,
            direction,
        } => command_summary(
            "inverse.reparent_node",
            Some(node_id.clone()),
            [
                ("parent_node_id", json!(parent_node_id)),
                ("direction", json!(direction)),
            ],
        ),
        UiAssetEditorInverseTreeEdit::WrapNode {
            node_id,
            wrapper_node_id,
            wrapper_widget_type,
        } => command_summary(
            "inverse.wrap_node",
            Some(node_id.clone()),
            [
                ("wrapper_node_id", json!(wrapper_node_id)),
                ("wrapper_widget_type", json!(wrapper_widget_type)),
            ],
        ),
        UiAssetEditorInverseTreeEdit::UnwrapNode {
            wrapper_node_id,
            child_node_id,
        } => command_summary(
            "inverse.unwrap_node",
            Some(wrapper_node_id.clone()),
            [("child_node_id", json!(child_node_id))],
        ),
        UiAssetEditorInverseTreeEdit::RestoreNodeDefinition {
            node_id,
            kind,
            widget_type,
            component,
            component_ref,
        } => command_summary(
            "inverse.restore_node_definition",
            Some(node_id.clone()),
            [
                ("kind", json!(format!("{kind:?}"))),
                ("widget_type", json!(widget_type)),
                ("component", json!(component)),
                ("component_ref", json!(component_ref)),
            ],
        ),
        UiAssetEditorInverseTreeEdit::InlineExtractedComponent {
            node_id,
            component_name,
            component_root_id,
        } => command_summary(
            "inverse.inline_extracted_component",
            Some(node_id.clone()),
            [
                ("component_name", json!(component_name)),
                ("component_root_id", json!(component_root_id)),
            ],
        ),
        UiAssetEditorInverseTreeEdit::RestorePromotedComponent {
            source_component_name,
            asset_id,
            component_name,
            document_id,
        } => command_summary(
            "inverse.restore_promoted_component",
            Some(source_component_name.clone()),
            [
                ("asset_id", json!(asset_id)),
                ("component_name", json!(component_name)),
                ("document_id", json!(document_id)),
            ],
        ),
    }
}

fn document_replay_command_summary(
    command: &UiAssetEditorDocumentReplayCommand,
) -> UiAssetEditorReplayCommandSummary {
    match command {
        UiAssetEditorDocumentReplayCommand::SetWidgetImports { references } => command_summary(
            "document.set_widget_imports",
            None,
            [("references", json!(references))],
        ),
        UiAssetEditorDocumentReplayCommand::InsertWidgetImport { index, reference } => {
            command_summary(
                "document.insert_widget_import",
                Some(reference.clone()),
                [("index", json!(index))],
            )
        }
        UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index, reference } => {
            command_summary(
                "document.remove_widget_import",
                Some(reference.clone()),
                [("index", json!(index))],
            )
        }
        UiAssetEditorDocumentReplayCommand::MoveWidgetImport {
            from_index,
            to_index,
            reference,
        } => command_summary(
            "document.move_widget_import",
            Some(reference.clone()),
            [
                ("from_index", json!(from_index)),
                ("to_index", json!(to_index)),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::SetRoot { root } => command_summary(
            "document.set_root",
            root.as_ref().map(|root| root.node_id.clone()),
            [("root_present", json!(root.is_some()))],
        ),
        UiAssetEditorDocumentReplayCommand::UpsertNode { node_id, node } => command_summary(
            "document.upsert_node",
            Some(node_id.clone()),
            [("node", node_summary(node))],
        ),
        UiAssetEditorDocumentReplayCommand::RemoveNode { node_id } => {
            command_summary("document.remove_node", Some(node_id.clone()), [])
        }
        UiAssetEditorDocumentReplayCommand::UpsertComponent {
            component_name,
            component,
        } => command_summary(
            "document.upsert_component",
            Some(component_name.clone()),
            [("component", component_summary(component))],
        ),
        UiAssetEditorDocumentReplayCommand::RemoveComponent { component_name } => command_summary(
            "document.remove_component",
            Some(component_name.clone()),
            [],
        ),
        UiAssetEditorDocumentReplayCommand::SetNodeBindings { node_id, bindings } => {
            command_summary(
                "document.set_node_bindings",
                Some(node_id.clone()),
                [(
                    "binding_ids",
                    json!(bindings
                        .iter()
                        .map(|binding| &binding.id)
                        .collect::<Vec<_>>()),
                )],
            )
        }
        UiAssetEditorDocumentReplayCommand::SetStyleImports { references } => command_summary(
            "document.set_style_imports",
            None,
            [("references", json!(references))],
        ),
        UiAssetEditorDocumentReplayCommand::InsertStyleImport { index, reference } => {
            command_summary(
                "document.insert_style_import",
                Some(reference.clone()),
                [("index", json!(index))],
            )
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleImport { index, reference } => {
            command_summary(
                "document.remove_style_import",
                Some(reference.clone()),
                [("index", json!(index))],
            )
        }
        UiAssetEditorDocumentReplayCommand::MoveStyleImport {
            from_index,
            to_index,
            reference,
        } => command_summary(
            "document.move_style_import",
            Some(reference.clone()),
            [
                ("from_index", json!(from_index)),
                ("to_index", json!(to_index)),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::SetStyleTokens { tokens } => command_summary(
            "document.set_style_tokens",
            None,
            [
                ("token_count", json!(tokens.len())),
                ("token_names", json!(tokens.keys().collect::<Vec<_>>())),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::UpsertStyleToken { token_name, value } => {
            command_summary(
                "document.upsert_style_token",
                Some(token_name.clone()),
                [("value_kind", json!(toml_value_kind(value)))],
            )
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleToken { token_name } => {
            command_summary("document.remove_style_token", Some(token_name.clone()), [])
        }
        UiAssetEditorDocumentReplayCommand::SetStyleSheets { stylesheets } => command_summary(
            "document.set_stylesheets",
            None,
            [("stylesheets", style_sheets_summary(stylesheets))],
        ),
        UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
            index,
            stylesheet_id,
            stylesheet,
        } => command_summary(
            "document.insert_stylesheet",
            Some(stylesheet_id.clone()),
            [
                ("index", json!(index)),
                (
                    "stylesheet",
                    stylesheet
                        .as_ref()
                        .map(style_sheet_summary)
                        .unwrap_or_else(|| json!({ "id": stylesheet_id, "rule_count": 0 })),
                ),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index,
            stylesheet_id,
        } => command_summary(
            "document.remove_stylesheet",
            Some(stylesheet_id.clone()),
            [("index", json!(index))],
        ),
        UiAssetEditorDocumentReplayCommand::ReplaceStyleSheet {
            index,
            stylesheet_id,
            stylesheet,
        } => command_summary(
            "document.replace_stylesheet",
            Some(stylesheet_id.clone()),
            [
                ("index", json!(index)),
                ("stylesheet", style_sheet_summary(stylesheet)),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
            from_index,
            to_index,
            stylesheet_id,
        } => command_summary(
            "document.move_stylesheet",
            Some(stylesheet_id.clone()),
            [
                ("from_index", json!(from_index)),
                ("to_index", json!(to_index)),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index,
            selector,
            rule,
        } => command_summary(
            "document.insert_style_rule",
            Some(selector.clone()),
            [
                ("stylesheet_index", json!(stylesheet_index)),
                ("index", json!(index)),
                (
                    "rule",
                    rule.as_ref()
                        .map(style_rule_summary)
                        .unwrap_or_else(|| json!({ "selector": selector })),
                ),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index,
            selector,
        } => command_summary(
            "document.remove_style_rule",
            Some(selector.clone()),
            [
                ("stylesheet_index", json!(stylesheet_index)),
                ("index", json!(index)),
            ],
        ),
        UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index,
            to_index,
        } => command_summary(
            "document.move_style_rule",
            None,
            [
                ("stylesheet_index", json!(stylesheet_index)),
                ("from_index", json!(from_index)),
                ("to_index", json!(to_index)),
            ],
        ),
    }
}

fn external_effect_summary(
    effect: &UiAssetEditorExternalEffect,
) -> UiAssetEditorReplayExternalEffectSummary {
    match effect {
        UiAssetEditorExternalEffect::UpsertAssetSource { asset_id, source } => {
            UiAssetEditorReplayExternalEffectSummary {
                effect_id: "upsert_asset_source".to_string(),
                asset_id: asset_id.clone(),
                source: Some(source_summary(source)),
            }
        }
        UiAssetEditorExternalEffect::RestoreAssetSource { asset_id, source } => {
            UiAssetEditorReplayExternalEffectSummary {
                effect_id: "restore_asset_source".to_string(),
                asset_id: asset_id.clone(),
                source: Some(source_summary(source)),
            }
        }
        UiAssetEditorExternalEffect::RemoveAssetSource { asset_id } => {
            UiAssetEditorReplayExternalEffectSummary {
                effect_id: "remove_asset_source".to_string(),
                asset_id: asset_id.clone(),
                source: None,
            }
        }
    }
}

fn command_summary<const N: usize>(
    command_id: impl Into<String>,
    target: Option<String>,
    payload_entries: [(&'static str, JsonValue); N],
) -> UiAssetEditorReplayCommandSummary {
    UiAssetEditorReplayCommandSummary {
        command_id: command_id.into(),
        target,
        payload: payload_entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    }
}

fn node_summary(node: &UiNodeDefinition) -> JsonValue {
    json!({
        "node_id": &node.node_id,
        "kind": format!("{:?}", node.kind),
        "widget_type": &node.widget_type,
        "component": &node.component,
        "component_ref": &node.component_ref,
        "slot_name": &node.slot_name,
        "control_id": &node.control_id,
        "classes": &node.classes,
        "param_keys": node.params.keys().collect::<Vec<_>>(),
        "prop_keys": node.props.keys().collect::<Vec<_>>(),
        "layout_keys": node
            .layout
            .as_ref()
            .map(|layout| layout.keys().collect::<Vec<_>>())
            .unwrap_or_default(),
        "binding_ids": node.bindings.iter().map(|binding| &binding.id).collect::<Vec<_>>(),
        "style_override_keys": style_declaration_keys(&node.style_overrides),
        "child_node_ids": node.children.iter().map(|child| &child.node.node_id).collect::<Vec<_>>(),
    })
}

fn component_summary(component: &UiComponentDefinition) -> JsonValue {
    json!({
        "root_node_id": &component.root.node_id,
        "style_scope": format!("{:?}", component.style_scope),
        "param_names": component.params.keys().collect::<Vec<_>>(),
        "slot_names": component.slots.keys().collect::<Vec<_>>(),
        "public_part_count": component.contract.public_parts.len(),
        "root_class_policy": format!("{:?}", component.contract.root_class_policy),
    })
}

fn style_sheets_summary(stylesheets: &[UiStyleSheet]) -> JsonValue {
    json!(stylesheets
        .iter()
        .map(style_sheet_summary)
        .collect::<Vec<_>>())
}

fn style_sheet_summary(stylesheet: &UiStyleSheet) -> JsonValue {
    json!({
        "id": &stylesheet.id,
        "rule_count": stylesheet.rules.len(),
        "rule_selectors": stylesheet
            .rules
            .iter()
            .map(|rule| &rule.selector)
            .collect::<Vec<_>>(),
    })
}

fn style_rule_summary(rule: &UiStyleRule) -> JsonValue {
    json!({
        "id": &rule.id,
        "selector": &rule.selector,
        "declaration_keys": style_declaration_keys(&rule.set),
    })
}

fn style_declaration_keys(block: &UiStyleDeclarationBlock) -> JsonValue {
    json!({
        "self": block.self_values.keys().collect::<Vec<_>>(),
        "slot": block.slot.keys().collect::<Vec<_>>(),
    })
}

fn toml_value_kind(value: &TomlValue) -> &'static str {
    match value {
        TomlValue::String(_) => "string",
        TomlValue::Integer(_) => "integer",
        TomlValue::Float(_) => "float",
        TomlValue::Boolean(_) => "boolean",
        TomlValue::Datetime(_) => "datetime",
        TomlValue::Array(_) => "array",
        TomlValue::Table(_) => "table",
    }
}

fn stable_text_hash(source: &str) -> u64 {
    source
        .as_bytes()
        .iter()
        .fold(FNV_1A_64_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_1A_64_PRIME)
        })
}

fn sanitize_text_for_artifact(input: &str) -> String {
    input
        .split_whitespace()
        .map(|token| {
            if is_absolute_path_token(token) {
                "[path]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_absolute_path_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|ch: char| {
        matches!(
            ch,
            '`' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
        )
    });
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 3
        && bytes[1] == b':'
        && bytes[0].is_ascii_alphabetic()
        && matches!(bytes[2], b'\\' | b'/')
    {
        return true;
    }
    trimmed.starts_with("\\\\") || trimmed.starts_with('/')
}
