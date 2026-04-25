use toml::Value;

use super::asset_migration_support::{
    legacy_template_layout_document, legacy_template_layout_source, migrate_flat_ui_asset_toml_str,
};
use crate::ui::event_ui::UiTreeId;
use crate::ui::surface::{UiRenderCommandKind, UiVisualAssetRef};
use crate::ui::template::{
    UiAssetLoader, UiDocumentCompiler, UiTemplateLoader, UiTemplateSurfaceBuilder,
};
use crate::ui::{layout::UiSize, template::UiAssetKind};

const IMPORTED_BUTTON_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.buttons"
version = 1
display_name = "Common Buttons"

[root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = ["toolbar-button"]
props = { text = "$param.label", icon = "$param.icon" }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }

[components.ToolbarButton]
style_scope = "closed"

[components.ToolbarButton.params.label]
type = "string"
default = "Toolbar"

[components.ToolbarButton.params.icon]
type = "string"
default = "ellipse-outline"

[components.ToolbarButton.root]
node_id = "button_root"
kind = "native"
type = "Button"
classes = ["toolbar-button"]
props = { text = "$param.label", icon = "$param.icon" }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }
"##;

const IMPORTED_TOOLBAR_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar"
version = 1
display_name = "Toolbar Shell"

[root]
node_id = "toolbar_root"
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }

[[root.children]]
[root.children.node]
node_id = "leading_slot"
kind = "slot"
slot_name = "leading"

[components.ToolbarShell]
style_scope = "closed"

[components.ToolbarShell.slots.leading]
required = false
multiple = true

[components.ToolbarShell.root]
node_id = "toolbar_root"
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }

[[components.ToolbarShell.root.children]]
[components.ToolbarShell.root.children.node]
node_id = "leading_slot"
kind = "slot"
slot_name = "leading"
"##;

const IMPORTED_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.editor_base"
version = 1
display_name = "Editor Base"

[tokens]
accent = "#4488ff"
open_text = "Open Styled"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = ".toolbar > Button.primary"
set = { self = { background = { color = "$accent" }, layout = { width = { preferred = 144.0 } } } }

[[stylesheets.rules]]
selector = "#OpenButton"
set = { self = { text = "$open_text" }, slot = { layout = { height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" } } } }
"##;

const LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 2
display_name = "UI Asset Editor"

[imports]
widgets = [
  "asset://ui/common/toolbar.ui#ToolbarShell",
  "asset://ui/common/buttons.ui#ToolbarButton",
]
styles = ["asset://ui/theme/editor_base.ui"]

[tokens]
panel_gap = 12.0

[root]
node_id = "editor_root"
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 12.0 } }

[[root.children]]
[root.children.node]
node_id = "toolbar"
kind = "reference"
component_ref = "asset://ui/common/toolbar.ui#ToolbarShell"
control_id = "ToolbarHost"
classes = ["toolbar"]

[[root.children.node.children]]
mount = "leading"
slot = { layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" } } }
[root.children.node.children.node]
node_id = "open_button"
kind = "reference"
component_ref = "asset://ui/common/buttons.ui#ToolbarButton"
control_id = "OpenButton"
classes = ["primary"]
params = { label = "Open", icon = "folder-open-outline" }
style_overrides = { self = { text = "Open Override" } }
"##;

const FLAT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 2
display_name = "UI Asset Editor"

[imports]
widgets = [
  "asset://ui/common/toolbar.ui#ToolbarShell",
  "asset://ui/common/buttons.ui#ToolbarButton",
]
styles = ["asset://ui/theme/editor_base.ui"]

[tokens]
panel_gap = 12.0

[root]
node = "editor_root"

[nodes.editor_root]
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 12.0 } }
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "asset://ui/common/toolbar.ui#ToolbarShell"
control_id = "ToolbarHost"
classes = ["toolbar"]
children = [{ child = "open_button", mount = "leading", slot = { layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" } } } }]

[nodes.open_button]
kind = "reference"
component_ref = "asset://ui/common/buttons.ui#ToolbarButton"
control_id = "OpenButton"
classes = ["primary"]
params = { label = "Open", icon = "folder-open-outline" }
style_overrides = { self = { text = "Open Override" } }
"##;

const LEGACY_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "VerticalBox"
control_id = "LegacyRoot"
attributes = { layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } } }
children = [
  { component = "Button", control_id = "LegacyButton", bindings = [{ id = "Legacy/Button", event = "Click", route = "MenuAction.OpenProject" }], attributes = { text = "Open" } }
]
"#;

const STYLE_WITH_RULE_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.rule_ids"
version = 1
display_name = "Rule Ids"

[[stylesheets]]
id = "rule_id_sheet"

[[stylesheets.rules]]
id = "primary_button_hover"
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Label" } }

[[stylesheets]]
id = "secondary_sheet"

[[stylesheets.rules]]
id = "secondary_label_rule"
selector = "Label.secondary"
set = { self = { text = "Secondary" } }
"##;

#[test]
fn ui_asset_stylesheet_rules_preserve_stable_rule_ids() {
    let document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let rules = &document.stylesheets[0].rules;
    assert_eq!(rules[0].id.as_deref(), Some("primary_button_hover"));
    assert_eq!(rules[1].id, None);

    let roundtrip = toml::to_string_pretty(&document).unwrap();
    assert!(roundtrip.contains("id = \"primary_button_hover\""));
    let reparsed = UiAssetLoader::load_toml_str(&roundtrip).unwrap();
    assert_eq!(
        reparsed.stylesheets[0].rules[0].id.as_deref(),
        Some("primary_button_hover")
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_found_and_edited_by_stable_id() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );
    assert!(document.style_rule("missing_rule").is_none());

    document
        .style_rule_mut("primary_button_hover")
        .unwrap()
        .selector = "Button.primary:pressed".to_string();

    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:pressed")
    );
}

#[test]
fn ui_asset_stylesheets_can_be_found_and_edited_by_stable_id() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
    assert!(document.style_sheet("missing_sheet").is_none());

    document.style_sheet_mut("rule_id_sheet").unwrap().id = "renamed_rule_id_sheet".to_string();

    assert!(document.style_sheet("rule_id_sheet").is_none());
    assert_eq!(
        document
            .style_sheet("renamed_rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_renamed_without_breaking_id_uniqueness() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .rename_style_rule("primary_button_hover", "primary_button_pressed")
        .unwrap());
    assert!(document.style_rule("primary_button_hover").is_none());
    assert_eq!(
        document
            .style_rule("primary_button_pressed")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );
    assert!(!document
        .rename_style_rule("missing_rule", "new_rule")
        .unwrap());

    let duplicate_error = document
        .rename_style_rule("primary_button_pressed", "secondary_label_rule")
        .expect_err("renaming to a duplicate style rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id secondary_label_rule"),
        "unexpected error: {duplicate_error:?}"
    );

    let empty_error = document
        .rename_style_rule("primary_button_pressed", " ")
        .expect_err("renaming to an empty style rule id should fail");
    assert!(
        empty_error
            .to_string()
            .contains("style rule id cannot be empty"),
        "unexpected error: {empty_error:?}"
    );
}

#[test]
fn ui_asset_stylesheets_can_be_renamed_without_breaking_id_uniqueness() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .rename_style_sheet("rule_id_sheet", "renamed_rule_id_sheet")
        .unwrap());
    assert!(document.style_sheet("rule_id_sheet").is_none());
    assert_eq!(
        document
            .style_sheet("renamed_rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(2)
    );
    assert!(!document
        .rename_style_sheet("missing_sheet", "new_sheet")
        .unwrap());

    let duplicate_error = document
        .rename_style_sheet("renamed_rule_id_sheet", "secondary_sheet")
        .expect_err("renaming to a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );

    let empty_error = document
        .rename_style_sheet("renamed_rule_id_sheet", " ")
        .expect_err("renaming to an empty stylesheet id should fail");
    assert!(
        empty_error
            .to_string()
            .contains("stylesheet id cannot be empty"),
        "unexpected error: {empty_error:?}"
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_removed_by_stable_id_for_editor_undo() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let removed = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed");
    assert_eq!(removed.id.as_deref(), Some("primary_button_hover"));
    assert_eq!(removed.selector, "Button.primary:hover");
    assert!(document.style_rule("primary_button_hover").is_none());
    assert_eq!(document.stylesheets[0].rules.len(), 1);

    assert!(document.remove_style_rule("missing_rule").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_removed_by_stable_id_for_editor_undo() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let removed = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed");
    assert_eq!(removed.id, "secondary_sheet");
    assert_eq!(removed.rules.len(), 1);
    assert!(document.style_sheet("secondary_sheet").is_none());
    assert_eq!(document.stylesheets.len(), 1);

    assert!(document.remove_style_sheet("missing_sheet").is_none());
}

#[test]
fn ui_asset_stylesheet_rules_can_be_inserted_at_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let removed = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed");

    assert!(document
        .insert_style_rule("rule_id_sheet", 0, removed)
        .unwrap());
    assert_eq!(
        document.stylesheets[0].rules[0].id.as_deref(),
        Some("primary_button_hover")
    );
    assert!(!document
        .insert_style_rule(
            "missing_sheet",
            0,
            document.style_rule("primary_button_hover").unwrap().clone()
        )
        .unwrap());

    let duplicate_error = document
        .insert_style_rule(
            "rule_id_sheet",
            0,
            document.style_rule("primary_button_hover").unwrap().clone(),
        )
        .expect_err("inserting a duplicate style rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id primary_button_hover"),
        "unexpected error: {duplicate_error:?}"
    );
}

#[test]
fn ui_asset_stylesheet_rules_can_be_replaced_atomically_for_editor_edits() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let mut replacement = document
        .style_rule("primary_button_hover")
        .expect("style rule")
        .clone();
    replacement.selector = "Button.primary:focus".to_string();

    let previous = document
        .replace_style_rule("primary_button_hover", replacement)
        .unwrap()
        .expect("style rule should be replaced");
    assert_eq!(previous.selector, "Button.primary:hover");
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:focus")
    );
    assert!(document
        .replace_style_rule(
            "missing_rule",
            document.style_rule("primary_button_hover").unwrap().clone()
        )
        .unwrap()
        .is_none());

    let duplicate_replacement = document
        .style_rule("secondary_label_rule")
        .expect("secondary rule")
        .clone();
    let duplicate_error = document
        .replace_style_rule("primary_button_hover", duplicate_replacement)
        .expect_err("replacing with a duplicate rule id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate style rule id secondary_label_rule"),
        "unexpected error: {duplicate_error:?}"
    );
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:focus")
    );
}

#[test]
fn ui_asset_stylesheet_rule_write_apis_reject_invalid_selectors_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let mut invalid_replacement = document
        .style_rule("primary_button_hover")
        .expect("style rule")
        .clone();
    invalid_replacement.selector = "Button#".to_string();

    let replace_error = document
        .replace_style_rule("primary_button_hover", invalid_replacement)
        .expect_err("replacing with an invalid selector should fail");
    assert!(
        matches!(
            replace_error,
            crate::ui::template::UiAssetError::InvalidSelector(_)
        ),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(
        document
            .style_rule("primary_button_hover")
            .map(|rule| rule.selector.as_str()),
        Some("Button.primary:hover")
    );

    let mut invalid_insert = document
        .remove_style_rule("primary_button_hover")
        .expect("style rule should be removed before reinserting");
    invalid_insert.selector = "Button#".to_string();
    let rule_count = document.stylesheets[0].rules.len();

    let insert_error = document
        .insert_style_rule("rule_id_sheet", 0, invalid_insert)
        .expect_err("inserting an invalid selector should fail");
    assert!(
        matches!(
            insert_error,
            crate::ui::template::UiAssetError::InvalidSelector(_)
        ),
        "unexpected error: {insert_error:?}"
    );
    assert_eq!(document.stylesheets[0].rules.len(), rule_count);
    assert!(document.style_rule("primary_button_hover").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_inserted_at_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let removed = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed");

    let inserted_index = document.insert_style_sheet(0, removed).unwrap();
    assert_eq!(inserted_index, 0);
    assert_eq!(document.stylesheets[0].id, "secondary_sheet");

    let duplicate_error = document
        .insert_style_sheet(0, document.style_sheet("secondary_sheet").unwrap().clone())
        .expect_err("inserting a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );
}

#[test]
fn ui_asset_stylesheets_can_be_replaced_atomically_for_editor_edits() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let mut replacement = document
        .style_sheet("rule_id_sheet")
        .expect("stylesheet")
        .clone();
    replacement.rules.clear();

    let previous = document
        .replace_style_sheet("rule_id_sheet", replacement)
        .unwrap()
        .expect("stylesheet should be replaced");
    assert_eq!(previous.rules.len(), 2);
    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(0)
    );
    assert!(document
        .replace_style_sheet(
            "missing_sheet",
            document.style_sheet("rule_id_sheet").unwrap().clone()
        )
        .unwrap()
        .is_none());

    let duplicate_replacement = document
        .style_sheet("secondary_sheet")
        .expect("secondary sheet")
        .clone();
    let duplicate_error = document
        .replace_style_sheet("rule_id_sheet", duplicate_replacement)
        .expect_err("replacing with a duplicate stylesheet id should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("duplicate stylesheet id secondary_sheet"),
        "unexpected error: {duplicate_error:?}"
    );
    assert_eq!(
        document
            .style_sheet("rule_id_sheet")
            .map(|stylesheet| stylesheet.rules.len()),
        Some(0)
    );
}

#[test]
fn ui_asset_stylesheet_write_apis_reject_invalid_selectors_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    let mut invalid_replacement = document
        .style_sheet("rule_id_sheet")
        .expect("stylesheet")
        .clone();
    invalid_replacement.rules[0].selector = "Button#".to_string();

    let replace_error = document
        .replace_style_sheet("rule_id_sheet", invalid_replacement)
        .expect_err("replacing a stylesheet with an invalid selector should fail");
    assert!(
        matches!(
            replace_error,
            crate::ui::template::UiAssetError::InvalidSelector(_)
        ),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(
        document.stylesheets[0].rules[0].selector,
        "Button.primary:hover"
    );

    let mut invalid_insert = document
        .remove_style_sheet("secondary_sheet")
        .expect("stylesheet should be removed before reinserting");
    invalid_insert.rules[0].selector = "Button#".to_string();
    let stylesheet_count = document.stylesheets.len();

    let insert_error = document
        .insert_style_sheet(0, invalid_insert)
        .expect_err("inserting a stylesheet with an invalid selector should fail");
    assert!(
        matches!(
            insert_error,
            crate::ui::template::UiAssetError::InvalidSelector(_)
        ),
        "unexpected error: {insert_error:?}"
    );
    assert_eq!(document.stylesheets.len(), stylesheet_count);
    assert!(document.style_sheet("secondary_sheet").is_none());
}

#[test]
fn ui_asset_stylesheets_can_be_replaced_atomically_for_editor_replay() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let original_stylesheets = document.stylesheets.clone();
    let mut replacement = original_stylesheets.clone();
    replacement[0].rules[0].selector = "Button#".to_string();

    let replace_error = document
        .set_style_sheets(replacement)
        .expect_err("setting invalid stylesheets should fail");
    assert!(
        matches!(
            replace_error,
            crate::ui::template::UiAssetError::InvalidSelector(_)
        ),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(document.stylesheets, original_stylesheets);

    let mut replacement = original_stylesheets.clone();
    replacement[0].rules.clear();
    assert!(document
        .set_style_sheets(replacement.clone())
        .expect("valid stylesheets should be accepted"));
    assert_eq!(document.stylesheets, replacement);
    assert!(!document
        .set_style_sheets(replacement)
        .expect("unchanged stylesheets should be a no-op"));
}

#[test]
fn ui_asset_stylesheets_replacement_rejects_duplicate_ids_atomically() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();
    let original_stylesheets = document.stylesheets.clone();
    let mut replacement = original_stylesheets.clone();
    replacement[1].id = replacement[0].id.clone();

    let replace_error = document
        .set_style_sheets(replacement)
        .expect_err("setting duplicate stylesheet ids should fail");
    assert!(
        replace_error
            .to_string()
            .contains("duplicate stylesheet id"),
        "unexpected error: {replace_error:?}"
    );
    assert_eq!(document.stylesheets, original_stylesheets);
}

#[test]
fn ui_asset_stylesheet_rules_can_be_moved_to_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert!(document
        .move_style_rule("secondary_label_rule", "rule_id_sheet", 1)
        .unwrap());
    assert_eq!(document.stylesheets[0].rules.len(), 3);
    assert_eq!(
        document.stylesheets[0].rules[1].id.as_deref(),
        Some("secondary_label_rule")
    );
    assert!(document.stylesheets[1].rules.is_empty());

    assert!(!document
        .move_style_rule("missing_rule", "rule_id_sheet", 0)
        .unwrap());
    assert!(!document
        .move_style_rule("secondary_label_rule", "missing_sheet", 0)
        .unwrap());
}

#[test]
fn ui_asset_stylesheets_can_be_moved_to_stable_editor_positions() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(document.move_style_sheet("secondary_sheet", 0), Some(0));
    assert_eq!(document.stylesheets[0].id, "secondary_sheet");
    assert_eq!(document.stylesheets[1].id, "rule_id_sheet");

    assert_eq!(document.move_style_sheet("secondary_sheet", 99), Some(1));
    assert_eq!(document.stylesheets[1].id, "secondary_sheet");
    assert_eq!(document.move_style_sheet("missing_sheet", 0), None);
}

#[test]
fn ui_asset_style_positions_follow_editor_reorder_operations() {
    let mut document = UiAssetLoader::load_toml_str(STYLE_WITH_RULE_IDS).unwrap();

    assert_eq!(document.style_sheet_index("rule_id_sheet"), Some(0));
    assert_eq!(document.style_sheet_index("secondary_sheet"), Some(1));
    let position = document
        .style_rule_position("secondary_label_rule")
        .expect("style rule position");
    assert_eq!(position.stylesheet_id, "secondary_sheet");
    assert_eq!(position.stylesheet_index, 1);
    assert_eq!(position.rule_index, 0);

    document
        .move_style_rule("secondary_label_rule", "rule_id_sheet", 1)
        .unwrap();
    document.move_style_sheet("secondary_sheet", 0);

    assert_eq!(document.style_sheet_index("rule_id_sheet"), Some(1));
    let position = document
        .style_rule_position("secondary_label_rule")
        .expect("style rule position after move");
    assert_eq!(position.stylesheet_id, "rule_id_sheet");
    assert_eq!(position.stylesheet_index, 1);
    assert_eq!(position.rule_index, 1);
    assert!(document.style_rule_position("missing_rule").is_none());
}

#[test]
fn ui_asset_loader_rejects_duplicate_stable_style_rule_ids() {
    const STYLE_WITH_DUPLICATE_RULE_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.duplicate_rule_ids"
version = 1
display_name = "Duplicate Rule Ids"

[[stylesheets]]
id = "first_sheet"

[[stylesheets.rules]]
id = "primary_button"
selector = "Button.primary"
set = { self = { text = "Primary" } }

[[stylesheets]]
id = "second_sheet"

[[stylesheets.rules]]
id = "primary_button"
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_DUPLICATE_RULE_IDS)
        .expect_err("duplicate stable style rule ids should be rejected");

    assert!(
        matches!(
            error,
            crate::ui::template::UiAssetError::InvalidDocument { .. }
        ),
        "unexpected error: {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("duplicate style rule id primary_button"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_duplicate_stable_stylesheet_ids() {
    const STYLE_WITH_DUPLICATE_STYLESHEET_IDS: &str = r##"
[asset]
kind = "style"
id = "ui.theme.duplicate_stylesheet_ids"
version = 1
display_name = "Duplicate Stylesheet Ids"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = "Button.primary"
set = { self = { text = "Primary" } }

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = "Button.primary:hover"
set = { self = { text = "Hover" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_DUPLICATE_STYLESHEET_IDS)
        .expect_err("duplicate stable stylesheet ids should be rejected");

    assert!(
        matches!(
            error,
            crate::ui::template::UiAssetError::InvalidDocument { .. }
        ),
        "unexpected error: {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("duplicate stylesheet id editor_base"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_blank_stable_stylesheet_ids() {
    const STYLE_WITH_BLANK_STYLESHEET_ID: &str = r##"
[asset]
kind = "style"
id = "ui.theme.blank_stylesheet_id"
version = 1
display_name = "Blank Stylesheet Id"

[[stylesheets]]
id = " "

[[stylesheets.rules]]
selector = "Button.primary"
set = { self = { text = "Primary" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_BLANK_STYLESHEET_ID)
        .expect_err("blank stable stylesheet ids should be rejected");

    assert!(
        matches!(
            error,
            crate::ui::template::UiAssetError::InvalidDocument { .. }
        ),
        "unexpected error: {error:?}"
    );
    assert!(
        error.to_string().contains("stylesheet id cannot be empty"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_loader_rejects_invalid_style_rule_selectors() {
    const STYLE_WITH_INVALID_SELECTOR: &str = r##"
[asset]
kind = "style"
id = "ui.theme.invalid_selector"
version = 1
display_name = "Invalid Selector"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
id = "bad_rule"
selector = "Button#"
set = { self = { text = "Bad" } }
"##;

    let error = UiAssetLoader::load_toml_str(STYLE_WITH_INVALID_SELECTOR)
        .expect_err("invalid style rule selectors should be rejected");

    assert!(
        matches!(error, crate::ui::template::UiAssetError::InvalidSelector(_)),
        "unexpected error: {error:?}"
    );
    assert!(
        error.to_string().contains("Button#"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets() {
    let button_asset = UiAssetLoader::load_toml_str(IMPORTED_BUTTON_ASSET_TOML).unwrap();
    let toolbar_asset = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_ASSET_TOML).unwrap();
    let style_asset = UiAssetLoader::load_toml_str(IMPORTED_STYLE_ASSET_TOML).unwrap();
    let layout_asset = UiAssetLoader::load_toml_str(LAYOUT_ASSET_TOML).unwrap();

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/buttons.ui#ToolbarButton", button_asset)
        .unwrap();
    compiler
        .register_widget_import("asset://ui/common/toolbar.ui#ToolbarShell", toolbar_asset)
        .unwrap();
    compiler
        .register_style_import("asset://ui/theme/editor_base.ui", style_asset)
        .unwrap();

    let compiled = compiler.compile(&layout_asset).unwrap();
    assert_eq!(compiled.asset.kind, UiAssetKind::Layout);
    assert_eq!(compiled.asset.id, "editor.ui_asset_editor");

    let instance = compiled.clone().into_template_instance();
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("HorizontalBox")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("ToolbarHost")
    );

    let open_button = instance.root.children[0]
        .children
        .iter()
        .find(|child| child.control_id.as_deref() == Some("OpenButton"))
        .unwrap();
    assert_eq!(open_button.component.as_deref(), Some("Button"));
    assert_eq!(
        open_button.attributes.get("text").unwrap().as_str(),
        Some("Open Override")
    );
    assert_eq!(
        open_button.attributes.get("icon").unwrap().as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(open_button.classes, vec!["toolbar-button", "primary"]);
    assert_eq!(
        open_button
            .attributes
            .get("background")
            .unwrap()
            .get("color")
            .unwrap()
            .as_str(),
        Some("#4488ff")
    );
    assert_eq!(
        open_button
            .attributes
            .get("layout")
            .unwrap()
            .get("width")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(144.0)
    );
    assert_eq!(
        open_button
            .slot_attributes
            .get("layout")
            .unwrap()
            .get("height")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(40.0)
    );

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("ui.asset.layout"),
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(800.0, 600.0)).unwrap();

    let open_button_node = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenButton")
        })
        .unwrap();
    assert_eq!(open_button_node.layout_cache.frame.width, 144.0);
    assert_eq!(open_button_node.layout_cache.frame.height, 40.0);

    let open_button_render = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == open_button_node.node_id)
        .unwrap();
    assert_eq!(open_button_render.kind, UiRenderCommandKind::Quad);
    assert_eq!(open_button_render.text.as_deref(), Some("Open Override"));
    assert_eq!(
        open_button_render.image,
        Some(UiVisualAssetRef::Icon("folder-open-outline".to_string()))
    );
    assert_eq!(open_button_render.opacity, 1.0);
    assert_eq!(
        open_button_render.style.background_color.as_deref(),
        Some("#4488ff")
    );
}

#[test]
fn ui_asset_loader_materializes_recursive_tree_authority_in_memory() {
    let document = UiAssetLoader::load_toml_str(LAYOUT_ASSET_TOML).unwrap();
    let root = document.root.as_ref().expect("layout root");

    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(root.children.len(), 1);

    let toolbar = &root.children[0];
    assert_eq!(toolbar.mount.as_deref(), None);
    assert_eq!(toolbar.node.node_id, "toolbar");
    assert_eq!(
        toolbar.node.component_ref.as_deref(),
        Some("asset://ui/common/toolbar.ui#ToolbarShell")
    );
    assert_eq!(toolbar.node.children.len(), 1);

    let open_button = &toolbar.node.children[0];
    assert_eq!(open_button.mount.as_deref(), Some("leading"));
    assert_eq!(open_button.node.node_id, "open_button");
    assert_eq!(
        open_button.node.component_ref.as_deref(),
        Some("asset://ui/common/buttons.ui#ToolbarButton")
    );
}

#[test]
fn ui_legacy_template_adapter_converts_template_documents_into_asset_documents() {
    let legacy = UiTemplateLoader::load_toml_str(LEGACY_TEMPLATE_TOML).unwrap();

    let asset_document =
        legacy_template_layout_document("legacy.workbench", "Legacy Workbench", &legacy).unwrap();

    assert_eq!(asset_document.asset.kind, UiAssetKind::Layout);
    assert_eq!(asset_document.asset.id, "legacy.workbench");
    assert_eq!(asset_document.asset.display_name, "Legacy Workbench");
    assert_eq!(asset_document.root.as_ref().unwrap().node_id, "root");
    assert_eq!(asset_document.root.as_ref().unwrap().children.len(), 1);
    assert_eq!(
        asset_document.root.as_ref().unwrap().children[0]
            .node
            .node_id,
        "root_0"
    );

    let compiler = UiDocumentCompiler::default();
    let compiled = compiler.compile(&asset_document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("LegacyRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("Button")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
    assert_eq!(
        instance.root.children[0].attributes.get("text"),
        Some(&Value::String("Open".to_string()))
    );
    assert_eq!(instance.root.children[0].bindings[0].id, "Legacy/Button");
}

#[test]
fn ui_legacy_template_adapter_emits_canonical_asset_source_that_roundtrips() {
    let legacy = UiTemplateLoader::load_toml_str(LEGACY_TEMPLATE_TOML).unwrap();

    let source =
        legacy_template_layout_source("legacy.workbench", "Legacy Workbench", &legacy).unwrap();
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(document.asset.id, "legacy.workbench");
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
}

#[test]
fn ui_flat_asset_migration_adapter_converts_flat_assets_into_tree_authority_source() {
    let source = migrate_flat_ui_asset_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    assert!(
        !source.contains("[nodes."),
        "migrated source should stop emitting flat [nodes.*] tables"
    );
    let root = document.root.as_ref().expect("migrated root");
    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].node.node_id, "toolbar");
    assert_eq!(root.children[0].node.children.len(), 1);
    assert_eq!(
        root.children[0].node.children[0].node.node_id,
        "open_button"
    );

    let button_asset = UiAssetLoader::load_toml_str(IMPORTED_BUTTON_ASSET_TOML).unwrap();
    let toolbar_asset = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_ASSET_TOML).unwrap();
    let style_asset = UiAssetLoader::load_toml_str(IMPORTED_STYLE_ASSET_TOML).unwrap();

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/buttons.ui#ToolbarButton", button_asset)
        .unwrap();
    compiler
        .register_widget_import("asset://ui/common/toolbar.ui#ToolbarShell", toolbar_asset)
        .unwrap();
    compiler
        .register_style_import("asset://ui/theme/editor_base.ui", style_asset)
        .unwrap();

    let compiled = compiler.compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(document.asset.id, "editor.ui_asset_editor");
    assert_eq!(instance.root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("ToolbarHost")
    );
    assert_eq!(
        instance.root.children[0].children[0].control_id.as_deref(),
        Some("OpenButton")
    );
}

#[test]
fn ui_asset_loader_rejects_flat_asset_documents_on_formal_path() {
    let error = UiAssetLoader::load_toml_str(FLAT_LAYOUT_ASSET_TOML)
        .expect_err("formal loader should reject flat authority documents");

    assert!(
        matches!(
            error,
            crate::ui::template::UiAssetError::ParseToml(_)
                | crate::ui::template::UiAssetError::InvalidDocument { .. }
        ),
        "unexpected error: {error:?}"
    );
}

#[test]
fn ui_asset_compiler_is_split_into_folder_backed_pipeline_modules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("template")
        .join("asset")
        .join("compiler");

    for relative in [
        "mod.rs",
        "ui_document_compiler.rs",
        "compile.rs",
        "node_expander.rs",
        "component_instance_expander.rs",
        "ui_style_resolver.rs",
        "style_apply.rs",
        "value_normalizer.rs",
        "shape_validator.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected compiler pipeline module {relative} under {:?}",
            root
        );
    }
}
