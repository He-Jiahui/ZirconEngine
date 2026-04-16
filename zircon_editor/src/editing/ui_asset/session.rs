use std::collections::BTreeMap;

use thiserror::Error;
use toml::Value;
use zircon_editor_ui::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiDesignerSelectionModel,
    UiStyleInspectorReflectionModel,
};
use zircon_ui::{
    UiAssetDocument, UiAssetError, UiAssetKind, UiAssetLoader, UiCompiledDocument,
    UiDocumentCompiler, UiSize, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
    UiTemplateBuildError, UiTreeError,
};

use super::{
    binding_inspector::{
        add_default_binding, build_binding_fields,
        delete_selected_binding as delete_selected_binding_field, reconcile_selected_binding_index,
        set_selected_binding_event as set_selected_binding_event_field,
        set_selected_binding_id as set_selected_binding_id_field,
        set_selected_binding_route as set_selected_binding_route_field,
    },
    command::UiAssetEditorCommand,
    inspector_fields::{
        build_inspector_fields, set_selected_node_control_id,
        set_selected_node_layout_height_preferred, set_selected_node_layout_width_preferred,
        set_selected_node_mount, set_selected_node_slot_height_preferred,
        set_selected_node_slot_padding, set_selected_node_slot_width_preferred,
        set_selected_node_text_property,
    },
    matched_rule_inspection::{
        matched_style_rule_entries, selector_component_name, selector_is_valid,
        MatchedStyleRuleEntry,
    },
    presentation::UiAssetEditorPanePresentation,
    preview_host::UiAssetPreviewHost,
    source_buffer::UiAssetSourceBuffer,
    style_rule_declarations::{
        declaration_entries, parse_declaration_literal, remove_declaration, set_declaration,
        UiStyleRuleDeclarationEntry, UiStyleRuleDeclarationPath,
    },
    undo_stack::UiAssetEditorUndoStack,
};

#[derive(Debug, Error)]
pub enum UiAssetEditorSessionError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error("expected ui asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
    #[error("cannot serialize an invalid ui asset source buffer")]
    InvalidSourceBuffer,
    #[error("ui asset selection index {index} is out of range")]
    InvalidSelectionIndex { index: usize },
    #[error("ui asset preview index {index} did not map to a selectable node")]
    InvalidPreviewIndex { index: usize },
    #[error("ui asset stylesheet rule index {index} is out of range")]
    InvalidStyleRuleIndex { index: usize },
    #[error("ui asset matched style rule index {index} is out of range")]
    InvalidMatchedStyleRuleIndex { index: usize },
    #[error("ui asset stylesheet rule declaration index {index} is out of range")]
    InvalidStyleRuleDeclarationIndex { index: usize },
    #[error("ui asset style token index {index} is out of range")]
    InvalidStyleTokenIndex { index: usize },
    #[error("ui asset binding index {index} is out of range")]
    InvalidBindingIndex { index: usize },
    #[error("ui asset stylesheet selector is invalid: {selector}")]
    InvalidStyleSelector { selector: String },
    #[error("ui asset stylesheet declaration path is invalid: {path}")]
    InvalidStyleDeclarationPath { path: String },
    #[error("ui asset inspector field {field} expects a numeric literal, received: {value}")]
    InvalidInspectorNumericLiteral { field: &'static str, value: String },
    #[error("ui asset binding event is invalid: {value}")]
    InvalidBindingEvent { value: String },
}

#[derive(Default)]
struct UiAssetCompilerImports {
    widgets: BTreeMap<String, UiAssetDocument>,
    styles: BTreeMap<String, UiAssetDocument>,
}

pub struct UiAssetEditorSession {
    route: UiAssetEditorRoute,
    source_buffer: UiAssetSourceBuffer,
    last_valid_document: UiAssetDocument,
    last_valid_compiled: Option<UiCompiledDocument>,
    preview_host: Option<UiAssetPreviewHost>,
    undo_stack: UiAssetEditorUndoStack,
    diagnostics: Vec<String>,
    selection: UiDesignerSelectionModel,
    style_inspector: UiStyleInspectorReflectionModel,
    selected_style_rule_index: Option<usize>,
    selected_matched_style_rule_index: Option<usize>,
    selected_style_rule_declaration_path: Option<String>,
    selected_style_token_name: Option<String>,
    selected_binding_index: Option<usize>,
    compiler_imports: UiAssetCompilerImports,
}

impl UiAssetEditorSession {
    pub fn from_source(
        route: UiAssetEditorRoute,
        source: impl Into<String>,
        preview_size: UiSize,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let source = source.into();
        let document = UiAssetLoader::load_toml_str(&source)?;
        ensure_asset_kind(route.asset_kind, document.asset.kind)?;
        let selection = default_selection(&document);
        let compiler_imports = UiAssetCompilerImports::default();
        let style_inspector =
            build_style_inspector(&document, &selection, &compiler_imports, &Vec::new());
        let selected_binding_index = reconcile_selected_binding_index(&document, &selection, None);
        let (last_valid_compiled, preview_host, diagnostics) =
            match compile_preview(&document, preview_size, &compiler_imports) {
                Ok((compiled, preview_host)) => (compiled, preview_host, Vec::new()),
                Err(error) => (None, None, vec![error.to_string()]),
            };
        Ok(Self {
            route,
            source_buffer: UiAssetSourceBuffer::new(source),
            last_valid_document: document,
            last_valid_compiled,
            preview_host,
            undo_stack: UiAssetEditorUndoStack::default(),
            diagnostics,
            selection,
            style_inspector,
            selected_style_rule_index: None,
            selected_matched_style_rule_index: None,
            selected_style_rule_declaration_path: None,
            selected_style_token_name: None,
            selected_binding_index,
            compiler_imports,
        })
    }

    pub fn route(&self) -> &UiAssetEditorRoute {
        &self.route
    }

    pub fn source_buffer(&self) -> &UiAssetSourceBuffer {
        &self.source_buffer
    }

    pub fn preview_host(&self) -> &UiAssetPreviewHost {
        self.preview_host
            .as_ref()
            .expect("preview host is only available for layout/widget assets")
    }

    pub fn preview_host_opt(&self) -> Option<&UiAssetPreviewHost> {
        self.preview_host.as_ref()
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    pub fn reflection_model(&self) -> UiAssetEditorReflectionModel {
        let mut model = UiAssetEditorReflectionModel::new(
            self.route.clone(),
            self.last_valid_document.asset.display_name.clone(),
        )
        .with_source_dirty(self.source_buffer.is_dirty())
        .with_undo_state(self.can_undo(), self.can_redo())
        .with_preview_available(self.preview_host.is_some())
        .with_selection(self.selection.clone())
        .with_style_inspector(self.style_inspector.clone());
        if let Some(error) = self.diagnostics.first() {
            model = model.with_last_error(error.clone());
        }
        model
    }

    pub fn pane_presentation(&self) -> UiAssetEditorPanePresentation {
        let reflection = self.reflection_model();
        let preview_summary = preview_summary(self.preview_host.as_ref());
        let selector_hint = selected_node_selector(&self.last_valid_document, &self.selection);
        let inspector_fields = build_inspector_fields(&self.last_valid_document, &self.selection);
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        let style_rules = local_style_rule_entries(&self.last_valid_document);
        let matched_style_rules = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        let style_tokens = local_style_token_entries(&self.last_valid_document);
        let selected_style_rule = self
            .selected_style_rule_index
            .and_then(|index| style_rules.get(index));
        let selected_matched_style_rule = self
            .selected_matched_style_rule_index
            .and_then(|index| matched_style_rules.get(index).map(|entry| (index, entry)));
        let style_rule_declarations = selected_style_rule
            .map(|entry| {
                declaration_entries(
                    &self.last_valid_document.stylesheets[entry.stylesheet_index].rules
                        [entry.rule_index]
                        .set,
                )
            })
            .unwrap_or_default();
        let selected_style_rule_declaration = self
            .selected_style_rule_declaration_path
            .as_deref()
            .and_then(|path| {
                style_rule_declarations
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                style_rule_declarations
                    .get(index)
                    .map(|entry| (index, entry))
            });
        let selected_style_token = self
            .selected_style_token_name
            .as_deref()
            .and_then(|name| {
                style_tokens
                    .iter()
                    .position(|entry| entry.name.as_str() == name)
            })
            .and_then(|index| style_tokens.get(index).map(|entry| (index, entry)));
        let can_create_rule =
            self.diagnostics.is_empty() && selector_hint.is_some() && self.preview_host.is_some();
        let can_extract_rule = can_create_rule
            && selected_node_has_inline_overrides(&self.last_valid_document, &self.selection);
        UiAssetEditorPanePresentation {
            asset_id: reflection.route.asset_id.clone(),
            mode: format!("{:?}", reflection.route.mode),
            source_dirty: reflection.source_dirty,
            can_save: reflection.source_dirty && reflection.last_error.is_none(),
            can_undo: reflection.can_undo,
            can_redo: reflection.can_redo,
            can_create_rule,
            can_extract_rule,
            preview_available: reflection.preview_available,
            style_state_hover: pseudo_state_active(&reflection.style_inspector, "hover"),
            style_state_focus: pseudo_state_active(&reflection.style_inspector, "focus"),
            style_state_pressed: pseudo_state_active(&reflection.style_inspector, "pressed"),
            style_state_disabled: pseudo_state_active(&reflection.style_inspector, "disabled"),
            style_state_selected: pseudo_state_active(&reflection.style_inspector, "selected"),
            style_class_items: reflection.style_inspector.classes.clone(),
            style_rule_items: style_rules
                .iter()
                .map(|rule| rule.selector.clone())
                .collect(),
            style_rule_selected_index: self
                .selected_style_rule_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            style_selected_rule_selector: selected_style_rule
                .map(|rule| rule.selector.clone())
                .unwrap_or_default(),
            style_can_edit_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            style_can_delete_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            style_matched_rule_items: matched_style_rules
                .iter()
                .map(MatchedStyleRuleEntry::label)
                .collect(),
            style_matched_rule_selected_index: selected_matched_style_rule
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_origin: selected_matched_style_rule
                .map(|(_, rule)| rule.origin_id.clone())
                .unwrap_or_default(),
            style_selected_matched_rule_selector: selected_matched_style_rule
                .map(|(_, rule)| rule.selector.clone())
                .unwrap_or_default(),
            style_selected_matched_rule_specificity: selected_matched_style_rule
                .map(|(_, rule)| rule.specificity as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_source_order: selected_matched_style_rule
                .map(|(_, rule)| rule.source_order as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_declaration_items: selected_matched_style_rule
                .map(|(_, rule)| rule.declaration_items())
                .unwrap_or_default(),
            style_rule_declaration_items: style_rule_declarations
                .iter()
                .map(|entry| format!("{} = {}", entry.path, entry.literal))
                .collect(),
            style_rule_declaration_selected_index: selected_style_rule_declaration
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_rule_declaration_path: selected_style_rule_declaration
                .map(|(_, entry)| entry.path.clone())
                .unwrap_or_default(),
            style_selected_rule_declaration_value: selected_style_rule_declaration
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            style_can_edit_rule_declaration: self.diagnostics.is_empty()
                && selected_style_rule.is_some(),
            style_can_delete_rule_declaration: self.diagnostics.is_empty()
                && selected_style_rule_declaration.is_some(),
            style_token_items: style_tokens
                .iter()
                .map(|entry| format!("{} = {}", entry.name, entry.literal))
                .collect(),
            style_token_selected_index: selected_style_token
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_token_name: selected_style_token
                .map(|(_, entry)| entry.name.clone())
                .unwrap_or_default(),
            style_selected_token_value: selected_style_token
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            style_can_edit_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
            style_can_delete_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
            last_error: reflection.last_error.clone().unwrap_or_default(),
            selection_summary: selection_summary(&reflection.selection),
            source_text: self.source_buffer.text().to_string(),
            preview_summary,
            inspector_selected_node_id: inspector_fields.selected_node_id,
            inspector_parent_node_id: inspector_fields.parent_node_id,
            inspector_mount: inspector_fields.mount,
            inspector_slot_padding: inspector_fields.slot_padding,
            inspector_slot_width_preferred: inspector_fields.slot_width_preferred,
            inspector_slot_height_preferred: inspector_fields.slot_height_preferred,
            inspector_layout_width_preferred: inspector_fields.layout_width_preferred,
            inspector_layout_height_preferred: inspector_fields.layout_height_preferred,
            inspector_binding_items: binding_fields.items,
            inspector_binding_selected_index: binding_fields.selected_index,
            inspector_binding_id: binding_fields.binding_id,
            inspector_binding_event: binding_fields.binding_event,
            inspector_binding_route: binding_fields.binding_route,
            inspector_can_edit_binding: self.diagnostics.is_empty() && binding_fields.can_edit,
            inspector_can_delete_binding: self.diagnostics.is_empty() && binding_fields.can_delete,
            inspector_widget_kind: inspector_fields.widget_kind,
            inspector_widget_label: inspector_fields.widget_label,
            inspector_control_id: inspector_fields.control_id,
            inspector_text_prop: inspector_fields.text_prop,
            inspector_can_edit_control_id: inspector_fields.can_edit_control_id,
            inspector_can_edit_text_prop: inspector_fields.can_edit_text_prop,
            palette_items: default_palette_items(),
            hierarchy_items: build_hierarchy_items(
                &self.last_valid_document,
                reflection.selection.primary_node_id.as_deref(),
            ),
            inspector_items: build_inspector_items(&reflection),
            stylesheet_items: build_stylesheet_items(&reflection.style_inspector, selector_hint),
            preview_items: build_preview_items(
                &self.last_valid_document,
                self.preview_host.as_ref(),
            ),
        }
    }

    pub fn set_mode(&mut self, mode: UiAssetEditorMode) -> Result<(), UiAssetEditorSessionError> {
        self.route.mode = mode;
        self.revalidate()
    }

    pub fn select_hierarchy_index(
        &mut self,
        index: usize,
    ) -> Result<(), UiAssetEditorSessionError> {
        let node_id = hierarchy_node_ids(&self.last_valid_document)
            .into_iter()
            .nth(index)
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.selection = selection_for_node(&self.last_valid_document, &node_id);
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        self.selected_binding_index = reconcile_selected_binding_index(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        self.selected_matched_style_rule_index = None;
        Ok(())
    }

    pub fn select_preview_index(&mut self, index: usize) -> Result<(), UiAssetEditorSessionError> {
        let Some(control_id) = self
            .preview_host
            .as_ref()
            .and_then(|host| preview_control_id_for_index(host, index))
        else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        let Some(node_id) = node_id_by_control_id(&self.last_valid_document, control_id) else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        self.selection = selection_for_node(&self.last_valid_document, &node_id);
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        self.selected_binding_index = reconcile_selected_binding_index(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        self.selected_matched_style_rule_index = None;
        Ok(())
    }

    pub fn create_rule_from_selection(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selector) = selected_node_selector(&self.last_valid_document, &self.selection)
        else {
            return Ok(false);
        };
        if self
            .last_valid_document
            .stylesheets
            .iter()
            .flat_map(|sheet| sheet.rules.iter())
            .any(|rule| rule.selector == selector)
        {
            return Ok(false);
        }

        let mut document = self.last_valid_document.clone();
        editable_stylesheet(&mut document).rules.push(UiStyleRule {
            selector,
            set: UiStyleDeclarationBlock::default(),
        });
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn extract_inline_overrides_to_rule(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(selector) = selected_node_selector(&self.last_valid_document, &self.selection)
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        if node.style_overrides.self_values.is_empty() && node.style_overrides.slot.is_empty() {
            return Ok(false);
        }

        let overrides = std::mem::take(&mut node.style_overrides);
        editable_stylesheet(&mut document).rules.push(UiStyleRule {
            selector,
            set: overrides,
        });
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn add_class_to_selection(
        &mut self,
        class_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(class_name) = normalized_class_name(class_name.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        if node.classes.iter().any(|existing| existing == &class_name) {
            return Ok(false);
        }
        node.classes.push(class_name);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn remove_class_from_selection(
        &mut self,
        class_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(class_name) = normalized_class_name(class_name.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        let before = node.classes.len();
        node.classes.retain(|existing| existing != &class_name);
        if before == node.classes.len() {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_widget_control_id(
        &mut self,
        control_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_control_id(&mut document, &self.selection, control_id.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_widget_text_property(
        &mut self,
        text: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_text_property(&mut document, &self.selection, text.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_mount(
        &mut self,
        mount: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_mount(&mut document, &self.selection, mount.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_padding(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed = set_selected_node_slot_padding(&mut document, &self.selection, literal)
            .map_err(
                |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                    field,
                    value: literal.to_string(),
                },
            )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_width_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_slot_width_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_height_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_slot_height_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_layout_width_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_layout_width_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_layout_height_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_layout_height_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_binding(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        if index >= binding_fields.items.len() {
            return Err(UiAssetEditorSessionError::InvalidBindingIndex { index });
        }
        let changed = self.selected_binding_index != Some(index);
        self.selected_binding_index = Some(index);
        Ok(changed)
    }

    pub fn add_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(next_index) = add_default_binding(&mut document, &self.selection) else {
            return Ok(false);
        };
        self.selected_binding_index = Some(next_index);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !delete_selected_binding_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
        ) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_binding_id(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_id_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_binding_event(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let value = value.as_ref();
        let changed = set_selected_binding_event_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value,
        )
        .map_err(|_| UiAssetEditorSessionError::InvalidBindingEvent {
            value: value.to_string(),
        })?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_binding_route(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_route_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_style_token(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let entries = local_style_token_entries(&self.last_valid_document);
        let Some(entry) = entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidStyleTokenIndex { index });
        };
        let changed = self.selected_style_token_name.as_deref() != Some(entry.name.as_str());
        self.selected_style_token_name = Some(entry.name.clone());
        Ok(changed)
    }

    pub fn upsert_style_token(
        &mut self,
        token_name: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(token_name) = normalized_token_name(token_name.as_ref()) else {
            return Ok(false);
        };
        let Some(value) = parse_token_literal(value_literal.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let current_value = document.tokens.get(&token_name).cloned();
        if let Some(selected_name) = self.selected_style_token_name.as_deref() {
            if selected_name != token_name {
                let _ = document.tokens.remove(selected_name);
            }
        }
        let _ = document.tokens.insert(token_name.clone(), value.clone());
        if self.selected_style_token_name.as_deref() == Some(token_name.as_str())
            && current_value.as_ref() == Some(&value)
        {
            return Ok(false);
        }
        if self.selected_style_token_name.is_none()
            && current_value.as_ref() == Some(&value)
            && self.last_valid_document.tokens.contains_key(&token_name)
        {
            self.selected_style_token_name = Some(token_name);
            return Ok(true);
        }
        self.selected_style_token_name = Some(token_name);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_style_token(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_name) = self.selected_style_token_name.clone() else {
            return Ok(false);
        };
        let current_entries = local_style_token_entries(&self.last_valid_document);
        let Some(current_index) = current_entries
            .iter()
            .position(|entry| entry.name.as_str() == selected_name)
        else {
            self.selected_style_token_name = None;
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if document.tokens.remove(&selected_name).is_none() {
            self.selected_style_token_name = None;
            return Ok(false);
        }

        let remaining_entries = local_style_token_entries(&document);
        self.selected_style_token_name = if remaining_entries.is_empty() {
            None
        } else {
            Some(
                remaining_entries[current_index.min(remaining_entries.len() - 1)]
                    .name
                    .clone(),
            )
        };
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_stylesheet_rule(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if local_style_rule_entries(&self.last_valid_document)
            .get(index)
            .is_none()
        {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        }
        let changed = self.selected_style_rule_index != Some(index);
        self.selected_style_rule_index = Some(index);
        self.selected_style_rule_declaration_path = None;
        Ok(changed)
    }

    pub fn select_matched_style_rule(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let entries = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        if entries.get(index).is_none() {
            return Err(UiAssetEditorSessionError::InvalidMatchedStyleRuleIndex { index });
        }
        let changed = self.selected_matched_style_rule_index != Some(index);
        self.selected_matched_style_rule_index = Some(index);
        Ok(changed)
    }

    pub fn select_stylesheet_rule_declaration(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let entries = selected_style_rule_declaration_entries(
            &self.last_valid_document,
            self.selected_style_rule_index,
        );
        let Some(entry) = entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleDeclarationIndex { index });
        };
        let changed =
            self.selected_style_rule_declaration_path.as_deref() != Some(entry.path.as_str());
        self.selected_style_rule_declaration_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn upsert_selected_stylesheet_rule_declaration(
        &mut self,
        path: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(rule_index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let Some(path) = UiStyleRuleDeclarationPath::parse(path.as_ref()) else {
            return Err(UiAssetEditorSessionError::InvalidStyleDeclarationPath {
                path: path.as_ref().trim().to_string(),
            });
        };
        let Some(value) = parse_declaration_literal(value_literal.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(rule_index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index: rule_index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        let next_path = path.display();
        let next_literal = value.to_string();
        let existing_literal = declaration_entries(&rule.set)
            .into_iter()
            .find(|entry| entry.path == next_path)
            .map(|entry| entry.literal);
        if self.selected_style_rule_declaration_path.as_deref() == Some(next_path.as_str())
            && existing_literal.as_deref() == Some(next_literal.as_str())
        {
            return Ok(false);
        }
        if self.selected_style_rule_declaration_path.is_none()
            && existing_literal.as_deref() == Some(next_literal.as_str())
        {
            self.selected_style_rule_declaration_path = Some(next_path);
            return Ok(true);
        }

        if let Some(selected_path) = self.selected_style_rule_declaration_path.as_deref() {
            if selected_path != next_path {
                let selected_path =
                    UiStyleRuleDeclarationPath::parse(selected_path).expect("selected path");
                let _ = remove_declaration(&mut rule.set, &selected_path);
            }
        }
        set_declaration(&mut rule.set, &path, value);
        self.selected_style_rule_declaration_path = Some(next_path);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_stylesheet_rule_declaration(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(rule_index) = self.selected_style_rule_index else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };
        let Some(selected_path) = self.selected_style_rule_declaration_path.clone() else {
            return Ok(false);
        };
        let current_entries =
            selected_style_rule_declaration_entries(&self.last_valid_document, Some(rule_index));
        let Some(current_index) = current_entries
            .iter()
            .position(|entry| entry.path.as_str() == selected_path)
        else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(rule_index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index: rule_index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        let Some(path) = UiStyleRuleDeclarationPath::parse(&selected_path) else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };
        if !remove_declaration(&mut rule.set, &path) {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        }

        let remaining_entries = declaration_entries(&rule.set);
        self.selected_style_rule_declaration_path = if remaining_entries.is_empty() {
            None
        } else {
            Some(
                remaining_entries[current_index.min(remaining_entries.len() - 1)]
                    .path
                    .clone(),
            )
        };
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn rename_selected_stylesheet_rule(
        &mut self,
        selector: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let selector = normalized_selector(selector.as_ref())?;
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        if rule.selector == selector {
            return Ok(false);
        }
        rule.selector = selector;
        self.selected_style_rule_index = Some(index);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_stylesheet_rule(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rules = &mut document.stylesheets[entry.stylesheet_index].rules;
        if entry.rule_index >= rules.len() {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        }
        let _ = rules.remove(entry.rule_index);
        let remaining = local_style_rule_entries(&document).len();
        self.selected_style_rule_index = (remaining > 0).then(|| index.min(remaining - 1));
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn toggle_pseudo_state_preview(
        &mut self,
        state: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let state = state.as_ref();
        if !SUPPORTED_PSEUDO_STATES
            .iter()
            .any(|candidate| candidate == &state)
        {
            return Ok(false);
        }
        if self.selection.primary_node_id.is_none() {
            return Ok(false);
        }

        let mut active_states = self.style_inspector.active_pseudo_states.clone();
        if let Some(index) = active_states
            .iter()
            .position(|candidate| candidate == state)
        {
            let _ = active_states.remove(index);
        } else {
            active_states.push(state.to_string());
        }
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &active_states,
        );
        let matched_entries = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &active_states,
        );
        self.selected_matched_style_rule_index = reconcile_selected_matched_style_rule_index(
            &matched_entries,
            self.selected_matched_style_rule_index,
        );
        Ok(true)
    }

    pub fn apply_command(
        &mut self,
        command: UiAssetEditorCommand,
    ) -> Result<(), UiAssetEditorSessionError> {
        let before = self.source_buffer.text().to_string();
        let after = command.next_source().to_string();
        self.source_buffer.replace(after.clone());
        self.undo_stack.push_source_edit(before, after);
        self.revalidate()
    }

    pub fn undo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(previous_source) = self.undo_stack.undo() else {
            return Ok(false);
        };
        self.source_buffer.replace(previous_source);
        self.revalidate()?;
        Ok(true)
    }

    pub fn redo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(next_source) = self.undo_stack.redo() else {
            return Ok(false);
        };
        self.source_buffer.replace(next_source);
        self.revalidate()?;
        Ok(true)
    }

    pub fn register_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(UiAssetKind::Widget, document.asset.kind)?;
        let _ = self
            .compiler_imports
            .widgets
            .insert(reference.into(), document);
        self.revalidate()?;
        Ok(())
    }

    pub fn register_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(UiAssetKind::Style, document.asset.kind)?;
        let _ = self
            .compiler_imports
            .styles
            .insert(reference.into(), document);
        self.revalidate()?;
        Ok(())
    }

    pub fn replace_imports(
        &mut self,
        widgets: BTreeMap<String, UiAssetDocument>,
        styles: BTreeMap<String, UiAssetDocument>,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.compiler_imports.widgets = widgets;
        self.compiler_imports.styles = styles;
        self.revalidate()
    }

    pub fn import_references(&self) -> (Vec<String>, Vec<String>) {
        (
            self.last_valid_document.imports.widgets.clone(),
            self.last_valid_document.imports.styles.clone(),
        )
    }

    pub fn canonical_source(&self) -> Result<String, UiAssetEditorSessionError> {
        if !self.diagnostics.is_empty() {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        Ok(toml::to_string_pretty(&self.last_valid_document)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))?)
    }

    pub fn save_to_canonical_source(&mut self) -> Result<String, UiAssetEditorSessionError> {
        let canonical = self.canonical_source()?;
        self.source_buffer.replace(canonical.clone());
        self.source_buffer.mark_saved();
        Ok(canonical)
    }

    fn ensure_editable_source(&self) -> Result<(), UiAssetEditorSessionError> {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(UiAssetEditorSessionError::InvalidSourceBuffer)
        }
    }

    fn apply_document_edit(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        let next_source = serialize_document(&document)?;
        self.apply_command(UiAssetEditorCommand::edit_source(next_source))?;
        Ok(())
    }

    fn revalidate(&mut self) -> Result<(), UiAssetEditorSessionError> {
        match UiAssetLoader::load_toml_str(self.source_buffer.text()) {
            Ok(document) => {
                ensure_asset_kind(self.route.asset_kind, document.asset.kind)?;
                self.last_valid_document = document;
                self.selection = reconcile_selection(&self.last_valid_document, &self.selection);
                self.selected_style_rule_index = reconcile_selected_style_rule_index(
                    &self.last_valid_document,
                    self.selected_style_rule_index,
                );
                self.selected_style_rule_declaration_path =
                    reconcile_selected_style_rule_declaration_path(
                        &self.last_valid_document,
                        self.selected_style_rule_index,
                        self.selected_style_rule_declaration_path.as_deref(),
                    );
                self.selected_style_token_name = reconcile_selected_style_token_name(
                    &self.last_valid_document,
                    self.selected_style_token_name.as_deref(),
                );
                self.selected_binding_index = reconcile_selected_binding_index(
                    &self.last_valid_document,
                    &self.selection,
                    self.selected_binding_index,
                );
                let active_states = self.style_inspector.active_pseudo_states.clone();
                self.style_inspector = build_style_inspector(
                    &self.last_valid_document,
                    &self.selection,
                    &self.compiler_imports,
                    &active_states,
                );
                let matched_entries = matched_style_rule_entries_for_selection(
                    &self.last_valid_document,
                    &self.selection,
                    &self.compiler_imports,
                    &active_states,
                );
                self.selected_matched_style_rule_index =
                    reconcile_selected_matched_style_rule_index(
                        &matched_entries,
                        self.selected_matched_style_rule_index,
                    );
                match compile_preview(
                    &self.last_valid_document,
                    preview_size(&self.preview_host),
                    &self.compiler_imports,
                ) {
                    Ok((compiled, preview_host)) => {
                        self.last_valid_compiled = compiled;
                        self.preview_host = preview_host;
                        self.diagnostics.clear();
                    }
                    Err(error) => {
                        self.diagnostics = vec![error.to_string()];
                    }
                }
                Ok(())
            }
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                Ok(())
            }
        }
    }
}

fn compile_preview(
    document: &UiAssetDocument,
    preview_size: UiSize,
    imports: &UiAssetCompilerImports,
) -> Result<(Option<UiCompiledDocument>, Option<UiAssetPreviewHost>), UiAssetEditorSessionError> {
    if matches!(document.asset.kind, UiAssetKind::Style) {
        return Ok((None, None));
    }

    let mut compiler = UiDocumentCompiler::default();
    for (reference, widget) in &imports.widgets {
        compiler.register_widget_import(reference.clone(), widget.clone())?;
    }
    for (reference, style) in &imports.styles {
        compiler.register_style_import(reference.clone(), style.clone())?;
    }
    let compiled = compiler.compile(document)?;
    let preview_host = UiAssetPreviewHost::new(preview_size, &document.asset.id, &compiled)?;
    Ok((Some(compiled), Some(preview_host)))
}

fn preview_size(current: &Option<UiAssetPreviewHost>) -> UiSize {
    current
        .as_ref()
        .map(UiAssetPreviewHost::preview_size)
        .unwrap_or(UiSize::new(1280.0, 720.0))
}

fn ensure_asset_kind(
    expected: UiAssetKind,
    actual: UiAssetKind,
) -> Result<(), UiAssetEditorSessionError> {
    if expected != actual {
        return Err(UiAssetEditorSessionError::UnexpectedKind { expected, actual });
    }
    Ok(())
}

fn default_selection(document: &UiAssetDocument) -> UiDesignerSelectionModel {
    document
        .root
        .as_ref()
        .map(|root| selection_for_node(document, &root.node))
        .unwrap_or_default()
}

fn reconcile_selection(
    document: &UiAssetDocument,
    current: &UiDesignerSelectionModel,
) -> UiDesignerSelectionModel {
    let primary = current.primary_node_id.as_deref();
    if primary.is_some_and(|node_id| document.nodes.contains_key(node_id)) {
        return current.clone();
    }
    default_selection(document)
}

fn build_style_inspector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> UiStyleInspectorReflectionModel {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiStyleInspectorReflectionModel::default();
    };
    let Some(node) = document.nodes.get(node_id) else {
        return UiStyleInspectorReflectionModel::default();
    };

    let mut inspector = UiStyleInspectorReflectionModel::for_node(node_id.to_string());
    for class_name in &node.classes {
        inspector = inspector.with_class(class_name.clone());
    }
    for state in active_states {
        inspector = inspector.with_active_pseudo_state(state.clone());
    }
    for (path, value) in &node.style_overrides.self_values {
        inspector =
            inspector.with_inline_override(format!("self.{path}"), toml_value_to_json(value));
    }
    for (path, value) in &node.style_overrides.slot {
        inspector =
            inspector.with_inline_override(format!("slot.{path}"), toml_value_to_json(value));
    }
    for rule in matched_style_rule_entries(document, &imports.styles, node_id, active_states) {
        inspector = inspector.with_matched_rule(rule.reflection());
    }
    inspector
}

fn toml_value_to_json(value: &toml::Value) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

fn default_palette_items() -> Vec<String> {
    vec![
        "Native / Container".to_string(),
        "Native / Overlay".to_string(),
        "Native / HorizontalBox".to_string(),
        "Native / VerticalBox".to_string(),
        "Native / FlowBox".to_string(),
        "Native / GridBox".to_string(),
        "Native / ScrollableBox".to_string(),
        "Native / Label".to_string(),
        "Native / Image".to_string(),
        "Native / Button".to_string(),
        "Native / TextField".to_string(),
        "Pattern / Toolbar".to_string(),
    ]
}

const SUPPORTED_PSEUDO_STATES: &[&str] = &[
    "hover", "focus", "pressed", "checked", "selected", "disabled",
];

fn serialize_document(document: &UiAssetDocument) -> Result<String, UiAssetEditorSessionError> {
    toml::to_string_pretty(document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()).into())
}

#[derive(Clone, Debug)]
struct LocalStyleRuleEntry {
    stylesheet_index: usize,
    rule_index: usize,
    selector: String,
}

#[derive(Clone, Debug)]
struct LocalStyleTokenEntry {
    name: String,
    literal: String,
}

fn local_style_rule_entries(document: &UiAssetDocument) -> Vec<LocalStyleRuleEntry> {
    let mut entries = Vec::new();
    for (stylesheet_index, stylesheet) in document.stylesheets.iter().enumerate() {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            entries.push(LocalStyleRuleEntry {
                stylesheet_index,
                rule_index,
                selector: rule.selector.clone(),
            });
        }
    }
    entries
}

fn selected_style_rule_declaration_entries(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
) -> Vec<UiStyleRuleDeclarationEntry> {
    selected_rule_index
        .and_then(|index| local_style_rule_entries(document).get(index).cloned())
        .map(|entry| {
            declaration_entries(
                &document.stylesheets[entry.stylesheet_index].rules[entry.rule_index].set,
            )
        })
        .unwrap_or_default()
}

fn matched_style_rule_entries_for_selection(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> Vec<MatchedStyleRuleEntry> {
    selection
        .primary_node_id
        .as_deref()
        .map(|node_id| {
            matched_style_rule_entries(document, &imports.styles, node_id, active_states)
        })
        .unwrap_or_default()
}

fn local_style_token_entries(document: &UiAssetDocument) -> Vec<LocalStyleTokenEntry> {
    document
        .tokens
        .iter()
        .map(|(name, value)| LocalStyleTokenEntry {
            name: name.clone(),
            literal: toml_value_literal(value),
        })
        .collect()
}

fn reconcile_selected_style_rule_index(
    document: &UiAssetDocument,
    current: Option<usize>,
) -> Option<usize> {
    let count = local_style_rule_entries(document).len();
    match (current, count) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn reconcile_selected_style_rule_declaration_path(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
    current: Option<&str>,
) -> Option<String> {
    let entries = selected_style_rule_declaration_entries(document, selected_rule_index);
    current
        .filter(|path| entries.iter().any(|entry| entry.path.as_str() == *path))
        .map(str::to_string)
}

fn reconcile_selected_matched_style_rule_index(
    entries: &[MatchedStyleRuleEntry],
    current: Option<usize>,
) -> Option<usize> {
    match (current, entries.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn reconcile_selected_style_token_name(
    document: &UiAssetDocument,
    current: Option<&str>,
) -> Option<String> {
    current
        .filter(|name| document.tokens.contains_key(*name))
        .map(str::to_string)
}

fn normalized_selector(selector: &str) -> Result<String, UiAssetEditorSessionError> {
    let trimmed = selector.trim();
    if trimmed.is_empty() || !selector_is_valid(trimmed) {
        return Err(UiAssetEditorSessionError::InvalidStyleSelector {
            selector: trimmed.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

fn normalized_class_name(class_name: &str) -> Option<String> {
    let trimmed = class_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

fn normalized_token_name(token_name: &str) -> Option<String> {
    let trimmed = token_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

fn parse_token_literal(value_literal: &str) -> Option<Value> {
    let trimmed = value_literal.trim();
    if trimmed.is_empty() {
        return None;
    }

    let wrapped = format!("value = {trimmed}");
    toml::from_str::<toml::Table>(&wrapped)
        .ok()
        .and_then(|table| table.get("value").cloned())
        .or_else(|| Some(Value::String(trimmed.to_string())))
}

fn toml_value_literal(value: &Value) -> String {
    value.to_string()
}

fn editable_stylesheet(document: &mut UiAssetDocument) -> &mut UiStyleSheet {
    if document.stylesheets.is_empty() {
        document.stylesheets.push(UiStyleSheet {
            id: "local_editor_rules".to_string(),
            rules: Vec::new(),
        });
    }
    document
        .stylesheets
        .last_mut()
        .expect("style sheet should exist after initialization")
}

fn selected_node_selector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .map(selector_for_node)
}

fn selector_for_node(node: &zircon_ui::UiNodeDefinition) -> String {
    if let Some(control_id) = node.control_id.as_deref() {
        return format!("#{control_id}");
    }

    let mut selector = selector_component_name(node).to_string();
    for class_name in &node.classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    selector
}

fn selected_node_has_inline_overrides(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .is_some_and(|node| {
            !node.style_overrides.self_values.is_empty() || !node.style_overrides.slot.is_empty()
        })
}

fn pseudo_state_active(inspector: &UiStyleInspectorReflectionModel, state: &str) -> bool {
    inspector
        .active_pseudo_states
        .iter()
        .any(|candidate| candidate == state)
}

fn selection_summary(selection: &UiDesignerSelectionModel) -> String {
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

fn build_hierarchy_items(document: &UiAssetDocument, selected: Option<&str>) -> Vec<String> {
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

fn build_inspector_items(reflection: &UiAssetEditorReflectionModel) -> Vec<String> {
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

fn build_stylesheet_items(
    inspector: &UiStyleInspectorReflectionModel,
    selector_hint: Option<String>,
) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(selector_hint) = selector_hint {
        items.push(format!("selection selector: {selector_hint}"));
    }
    if !inspector.active_pseudo_states.is_empty() {
        items.push(format!(
            "states: {}",
            inspector.active_pseudo_states.join(", ")
        ));
    }
    for (path, value) in &inspector.inline_overrides {
        items.push(format!("override {path} = {value}"));
    }
    for rule in &inspector.matched_rules {
        items.push(format!(
            "{} (spec {}, order {})",
            rule.selector, rule.specificity, rule.source_order
        ));
    }
    if items.is_empty() {
        items.push("no matched stylesheet rules".to_string());
    }
    items
}

fn preview_summary(preview_host: Option<&UiAssetPreviewHost>) -> String {
    let Some(preview_host) = preview_host else {
        return "preview unavailable".to_string();
    };
    format!(
        "{} rendered nodes @ {:.0}x{:.0}",
        preview_host.surface().render_extract.list.commands.len(),
        preview_host.preview_size().width,
        preview_host.preview_size().height
    )
}

fn build_preview_items(
    document: &UiAssetDocument,
    preview_host: Option<&UiAssetPreviewHost>,
) -> Vec<String> {
    let Some(preview_host) = preview_host else {
        return vec!["no shared preview surface".to_string()];
    };
    preview_host
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .filter_map(|command| {
            let node = preview_host.surface().tree.node(command.node_id)?;
            let metadata = node.template_metadata.as_ref();
            let label = metadata
                .and_then(|metadata| metadata.control_id.clone())
                .unwrap_or_else(|| format!("#{}", command.node_id.0));
            let component = preview_item_component_label(document, metadata)
                .unwrap_or_else(|| "Node".to_string());
            Some(format!(
                "{} [{}] {:.0},{:.0} {:.0}x{:.0}",
                label,
                component,
                command.frame.x,
                command.frame.y,
                command.frame.width,
                command.frame.height
            ))
        })
        .collect()
}

fn preview_item_component_label(
    document: &UiAssetDocument,
    metadata: Option<&zircon_ui::UiTemplateNodeMetadata>,
) -> Option<String> {
    let rendered_component = metadata
        .map(|metadata| metadata.component.clone())
        .filter(|component| !component.is_empty());
    let document_component = metadata
        .and_then(|metadata| metadata.control_id.as_deref())
        .and_then(|control_id| node_id_by_control_id(document, control_id))
        .and_then(|node_id| document.nodes.get(&node_id))
        .and_then(node_component_label);

    match (document_component, rendered_component) {
        (Some(document_component), Some(rendered_component))
            if document_component != rendered_component =>
        {
            Some(format!("{document_component}/{rendered_component}"))
        }
        (Some(document_component), _) => Some(document_component),
        (_, Some(rendered_component)) => Some(rendered_component),
        _ => None,
    }
}

fn node_component_label(node: &zircon_ui::UiNodeDefinition) -> Option<String> {
    node.component_ref
        .as_deref()
        .and_then(|reference| reference.split_once('#').map(|(_, component)| component))
        .map(str::to_string)
        .or_else(|| node.component.clone())
        .or_else(|| node.widget_type.clone())
}

fn hierarchy_node_ids(document: &UiAssetDocument) -> Vec<String> {
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

fn selection_for_node(document: &UiAssetDocument, node_id: &str) -> UiDesignerSelectionModel {
    let mut selection = UiDesignerSelectionModel::single(node_id.to_string());
    if let Some((parent_node_id, mount)) = parent_for_node(document, node_id) {
        selection = selection.with_parent(parent_node_id);
        if let Some(mount) = mount {
            selection = selection.with_mount(mount);
        }
    }
    selection
}

fn parent_for_node(document: &UiAssetDocument, node_id: &str) -> Option<(String, Option<String>)> {
    for (parent_id, node) in &document.nodes {
        for child in &node.children {
            if child.child == node_id {
                return Some((parent_id.clone(), child.mount.clone()));
            }
        }
    }
    None
}

fn preview_control_id_for_index(preview_host: &UiAssetPreviewHost, index: usize) -> Option<&str> {
    let command = preview_host
        .surface()
        .render_extract
        .list
        .commands
        .get(index)?;
    preview_host
        .surface()
        .tree
        .node(command.node_id)?
        .template_metadata
        .as_ref()?
        .control_id
        .as_deref()
}

fn node_id_by_control_id(document: &UiAssetDocument, control_id: &str) -> Option<String> {
    document.nodes.iter().find_map(|(node_id, node)| {
        (node.control_id.as_deref() == Some(control_id)).then(|| node_id.clone())
    })
}
