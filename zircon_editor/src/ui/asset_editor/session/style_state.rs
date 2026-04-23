use super::{
    command::{UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand},
    inspector_fields::{
        set_selected_node_control_id, set_selected_node_layout_height_preferred,
        set_selected_node_layout_width_preferred, set_selected_node_mount,
        set_selected_node_slot_height_preferred, set_selected_node_slot_padding,
        set_selected_node_slot_width_preferred, set_selected_node_text_property,
    },
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group,
        delete_selected_layout_semantic as delete_selected_layout_semantic_field,
        delete_selected_slot_semantic as delete_selected_slot_semantic_field,
        set_selected_layout_semantic_value as set_selected_layout_semantic_value_field,
        set_selected_slot_semantic_value as set_selected_slot_semantic_value_field,
    },
    style_inspection::{
        build_style_inspector, local_style_rule_entries, local_style_token_entries,
        matched_style_rule_entries_for_selection, normalized_class_name, normalized_selector,
        normalized_token_name, parse_token_literal, reconcile_selected_matched_style_rule_index,
        selected_node_selector, selected_style_rule_declaration_entries, SUPPORTED_PSEUDO_STATES,
    },
    style_rule_declarations::{
        declaration_entries, parse_declaration_literal, remove_declaration, set_declaration,
        UiStyleRuleDeclarationPath,
    },
    theme_state::theme_document_replay_bundle,
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};
use zircon_runtime::ui::template::{
    UiAssetDocument, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
};

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
        let stylesheet_index = if document.stylesheets.is_empty() {
            0
        } else {
            document.stylesheets.len() - 1
        };
        let rule = UiStyleRule {
            selector,
            set: UiStyleDeclarationBlock::default(),
        };
        let rule_index = editable_stylesheet(&mut document).rules.len();
        editable_stylesheet(&mut document).rules.push(rule.clone());
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Create Stylesheet Rule",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule,
            ),
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
        let stylesheet_index = if document.stylesheets.is_empty() {
            0
        } else {
            document.stylesheets.len() - 1
        };
        let rule = UiStyleRule {
            selector,
            set: overrides,
        };
        let rule_index = editable_stylesheet(&mut document).rules.len();
        editable_stylesheet(&mut document).rules.push(rule.clone());
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Extract Inline Overrides",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule,
            ),
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
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Upsert Style Token", replay)?;
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
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Delete Style Token", replay)?;
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

    pub fn move_selected_stylesheet_rule_up(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_stylesheet_rule(-1)
    }

    pub fn move_selected_stylesheet_rule_down(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_stylesheet_rule(1)
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
        let removed_rule = rules.remove(entry.rule_index);
        let remaining = local_style_rule_entries(&document).len();
        self.selected_style_rule_index = (remaining > 0).then(|| index.min(remaining - 1));
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Delete Stylesheet Rule",
            style_rule_remove_replay_bundle(entry.stylesheet_index, entry.rule_index, removed_rule),
        )?;
        Ok(true)
    }

    fn move_selected_stylesheet_rule(
        &mut self,
        delta: isize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rules = &mut document.stylesheets[entry.stylesheet_index].rules;
        let next_rule_index = entry.rule_index as isize + delta;
        if next_rule_index < 0 || next_rule_index >= rules.len() as isize {
            return Ok(false);
        }
        let from_index = entry.rule_index;
        let to_index = next_rule_index as usize;
        rules.swap(entry.rule_index, next_rule_index as usize);
        self.selected_style_rule_index = Some((index as isize + delta) as usize);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            if delta < 0 {
                "Move Stylesheet Rule Up"
            } else {
                "Move Stylesheet Rule Down"
            },
            style_rule_move_replay_bundle(entry.stylesheet_index, from_index, to_index),
        )?;
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
}

pub(super) fn editable_stylesheet(document: &mut UiAssetDocument) -> &mut UiStyleSheet {
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

pub(super) fn style_rule_insert_replay_bundle(
    before_document: &UiAssetDocument,
    stylesheet_index: usize,
    rule_index: usize,
    rule: UiStyleRule,
) -> UiAssetEditorDocumentReplayBundle {
    if before_document.stylesheets.is_empty() {
        let stylesheet = UiStyleSheet {
            id: "local_editor_rules".to_string(),
            rules: vec![rule],
        };
        return UiAssetEditorDocumentReplayBundle {
            undo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                index: stylesheet_index,
                stylesheet_id: stylesheet.id.clone(),
            }],
            redo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                index: stylesheet_index,
                stylesheet_id: stylesheet.id.clone(),
                stylesheet: Some(stylesheet),
            }],
        };
    }

    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
            rule: Some(rule),
        }],
    }
}

pub(super) fn style_rule_remove_replay_bundle(
    stylesheet_index: usize,
    rule_index: usize,
    rule: UiStyleRule,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
            rule: Some(rule.clone()),
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector,
        }],
    }
}

pub(super) fn style_rule_move_replay_bundle(
    stylesheet_index: usize,
    from_index: usize,
    to_index: usize,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index: to_index,
            to_index: from_index,
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index,
            to_index,
        }],
    }
}
