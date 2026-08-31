mod stylesheet_state;

use self::stylesheet_state::style_rule_insert_replay_bundle;
use super::{
    inspector_fields::{
        set_selected_node_control_id, set_selected_node_layout_height_preferred,
        set_selected_node_layout_width_preferred, set_selected_node_mount,
        set_selected_node_prop_value, set_selected_node_slot_height_preferred,
        set_selected_node_slot_padding, set_selected_node_slot_width_preferred,
        set_selected_node_state_value, set_selected_node_text_property,
    },
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group,
        delete_selected_layout_semantic as delete_selected_layout_semantic_field,
        delete_selected_slot_semantic as delete_selected_slot_semantic_field,
        set_selected_layout_semantic_value as set_selected_layout_semantic_value_field,
        set_selected_slot_semantic_value as set_selected_slot_semantic_value_field,
    },
    style_inspection::{normalized_class_name, parse_token_literal, selected_node_selector},
    style_rule_identity::unique_style_rule_id,
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
};

pub(super) fn editable_stylesheet(document: &mut UiAssetDocument) -> usize {
    if document.stylesheets.is_empty() {
        document.stylesheets.push(UiStyleSheet {
            id: "local_editor_rules".to_string(),
            rules: Vec::new(),
        });
    }
    document.stylesheets.len() - 1
}

impl UiAssetEditorSession {
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
        let stylesheet_index = editable_stylesheet(&mut document);
        let stylesheet_id = document.stylesheets[stylesheet_index].id.clone();
        let rule = UiStyleRule {
            id: Some(unique_style_rule_id(&document, &selector)),
            selector,
            set: UiStyleDeclarationBlock::default(),
        };
        let rule_index = document.stylesheets[stylesheet_index].rules.len();
        if !document.insert_style_rule(&stylesheet_id, rule_index, rule.clone())? {
            return Ok(false);
        }
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_replay_and_style_rule_selection(
            document,
            "Create Stylesheet Rule",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule.clone(),
            ),
            rule.id,
        )?;
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
        let Some(node) = document.node_mut(node_id) else {
            return Ok(false);
        };
        if node.style_overrides.self_values.is_empty() && node.style_overrides.slot.is_empty() {
            return Ok(false);
        }

        let overrides = std::mem::take(&mut node.style_overrides);
        let stylesheet_index = editable_stylesheet(&mut document);
        let stylesheet_id = document.stylesheets[stylesheet_index].id.clone();
        let rule = UiStyleRule {
            id: Some(unique_style_rule_id(&document, &selector)),
            selector,
            set: overrides,
        };
        let rule_index = document.stylesheets[stylesheet_index].rules.len();
        if !document.insert_style_rule(&stylesheet_id, rule_index, rule.clone())? {
            return Ok(false);
        }
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_replay_and_style_rule_selection(
            document,
            "Extract Inline Overrides",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule.clone(),
            ),
            rule.id,
        )?;
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
        let Some(node) = document.node_mut(node_id) else {
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
        let Some(node) = document.node_mut(node_id) else {
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

    pub fn set_selected_widget_prop_literal(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_prop_value(
            &mut document,
            &self.selection,
            path.as_ref(),
            parse_token_literal(literal.as_ref()),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Widget Prop Edit")?;
        Ok(true)
    }

    pub fn set_selected_widget_state_literal(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_state_value(
            &mut document,
            &self.selection,
            path.as_ref(),
            parse_token_literal(literal.as_ref()),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Widget State Edit")?;
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

    pub fn select_slot_semantic(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let semantic_group = build_slot_semantic_group(&self.last_valid_document, &self.selection);
        let Some(entry) = semantic_group.entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_slot_semantic_path.as_deref() != Some(entry.path.as_str());
        self.selected_slot_semantic_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn set_selected_slot_semantic_value(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_slot_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !set_selected_slot_semantic_value_field(
            &mut document,
            &self.selection,
            &path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Slot Semantic Edit")?;
        Ok(true)
    }

    pub fn set_selected_slot_semantic_field(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let path = path.as_ref().trim();
        if path.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        if !set_selected_slot_semantic_value_field(
            &mut document,
            &self.selection,
            path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.selected_slot_semantic_path = Some(path.to_string());
        self.apply_document_edit_with_label(document, "Slot Semantic Edit")?;
        Ok(true)
    }

    pub fn delete_selected_slot_semantic(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_slot_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !delete_selected_slot_semantic_field(&mut document, &self.selection, &path) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Slot Semantic Delete")?;
        Ok(true)
    }

    pub fn select_layout_semantic(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let semantic_group =
            build_layout_semantic_group(&self.last_valid_document, &self.selection);
        let Some(entry) = semantic_group.entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_layout_semantic_path.as_deref() != Some(entry.path.as_str());
        self.selected_layout_semantic_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn set_selected_layout_semantic_value(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_layout_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !set_selected_layout_semantic_value_field(
            &mut document,
            &self.selection,
            &path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Layout Semantic Edit")?;
        Ok(true)
    }

    pub fn set_selected_layout_semantic_field(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let path = path.as_ref().trim();
        if path.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        if !set_selected_layout_semantic_value_field(
            &mut document,
            &self.selection,
            path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.selected_layout_semantic_path = Some(path.to_string());
        self.apply_document_edit_with_label(document, "Layout Semantic Edit")?;
        Ok(true)
    }

    pub fn delete_selected_layout_semantic(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_layout_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !delete_selected_layout_semantic_field(&mut document, &self.selection, &path) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Layout Semantic Delete")?;
        Ok(true)
    }
}
