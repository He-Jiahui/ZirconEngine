use crate::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use toml::Value;
use zircon_ui::{UiAssetKind, UiSize, template::UiAssetLoader};

const THEME_SUMMARY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.theme_summary"
version = 1
display_name = "Theme Summary"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Theme Summary" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "Theme Summary Local" } }
"##;

const IMPORTED_THEME_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
border = "#223344"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Imported Theme" } }
"##;

const IMPORTED_THEME_COLLISION_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

const IMPORTED_THEME_MERGE_PREVIEW_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[imports]
styles = ["res://ui/theme/base_tokens.ui.toml"]

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

const IMPORTED_THEME_RULE_DIFF_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme", background.color = "$accent" } }
"##;

const DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_dedupe"
version = 1
display_name = "Theme Dedupe"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

[tokens]
accent = "#223344"
panel = "$accent"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

const MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_multi_cascade"
version = 1
display_name = "Theme Multi Cascade"

[imports]
styles = [
  "res://ui/theme/shared_a.ui.toml",
  "res://ui/theme/shared_b.ui.toml",
]

[tokens]
accent = "#5588ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "CascadeButton"
props = { text = "Cascade" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Local Theme" } }
"##;

const IMPORTED_THEME_CASCADE_A_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_a"
version = 1
display_name = "Shared Theme A"

[tokens]
accent = "#112233"

[[stylesheets]]
id = "shared_theme_a"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme A" } }
"##;

const IMPORTED_THEME_CASCADE_B_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_b"
version = 1
display_name = "Shared Theme B"

[tokens]
accent = "#334455"

[[stylesheets]]
id = "shared_theme_b"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme B" } }
"##;

#[test]
fn ui_asset_editor_session_projects_theme_sources_and_selection() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_ASSET_TOML).expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");

    let local_pane = session.pane_presentation();
    assert_eq!(
        local_pane.theme_source_items,
        vec![
            "Local Theme • 1 tokens • 1 rules".to_string(),
            "res://ui/theme/shared_theme.ui.toml • 1 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(local_pane.theme_source_selected_index, 0);
    assert_eq!(local_pane.theme_selected_source_kind, "Local");
    assert_eq!(local_pane.theme_selected_source_reference, "local");
    assert_eq!(local_pane.theme_selected_source_token_count, 1);
    assert_eq!(local_pane.theme_selected_source_rule_count, 1);
    assert!(local_pane.theme_selected_source_available);
    assert!(local_pane.theme_can_promote_local);
    assert_eq!(
        local_pane.theme_selected_source_token_items,
        vec!["accent = \"#4488ff\"".to_string()]
    );
    assert_eq!(
        local_pane.theme_selected_source_rule_items,
        vec!["local_theme • #RootLabel".to_string()]
    );
    assert_eq!(
        local_pane.theme_cascade_layer_items,
        vec![
            "1. Imported • res://ui/theme/shared_theme.ui.toml • 1 tokens • 1 rules".to_string(),
            "2. Local • 1 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(
        local_pane.theme_cascade_token_items,
        vec![
            "active • accent • Local = \"#4488ff\"".to_string(),
            "active • border • res://ui/theme/shared_theme.ui.toml = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        local_pane.theme_cascade_rule_items,
        vec![
            "1. Imported • res://ui/theme/shared_theme.ui.toml • shared_theme • Label".to_string(),
            "2. Local • local_theme • #RootLabel".to_string(),
        ]
    );

    assert!(session
        .select_theme_source(1)
        .expect("select imported theme"));
    let imported_pane = session.pane_presentation();
    assert_eq!(imported_pane.theme_source_selected_index, 1);
    assert_eq!(imported_pane.theme_selected_source_kind, "Imported");
    assert_eq!(
        imported_pane.theme_selected_source_reference,
        "res://ui/theme/shared_theme.ui.toml"
    );
    assert_eq!(imported_pane.theme_selected_source_token_count, 1);
    assert_eq!(imported_pane.theme_selected_source_rule_count, 1);
    assert!(imported_pane.theme_selected_source_available);
    assert_eq!(
        imported_pane.theme_selected_source_token_items,
        vec!["border = \"#223344\"".to_string()]
    );
    assert_eq!(
        imported_pane.theme_selected_source_rule_items,
        vec!["shared_theme • Label".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_reports_missing_imported_theme_details_as_unavailable() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    assert!(session
        .select_theme_source(1)
        .expect("select unresolved imported theme"));
    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_selected_source_reference,
        "res://ui/theme/shared_theme.ui.toml"
    );
    assert_eq!(pane.theme_selected_source_kind, "Imported");
    assert!(!pane.theme_selected_source_available);
    assert_eq!(pane.theme_selected_source_token_items, Vec::<String>::new());
    assert_eq!(pane.theme_selected_source_rule_items, Vec::<String>::new());
}

#[test]
fn ui_asset_editor_session_resolves_selected_theme_source_asset_id_only_for_available_imports() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_ASSET_TOML).expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    assert_eq!(session.selected_theme_source_asset_id(), None);

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");
    assert_eq!(
        session.selected_theme_source_asset_id().as_deref(),
        Some("res://ui/theme/shared_theme.ui.toml")
    );

    let mut missing_session = UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(
            "res://ui/tests/theme-summary.ui.toml",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("missing theme summary session");
    missing_session
        .select_theme_source(1)
        .expect("select unresolved imported theme");
    assert_eq!(missing_session.selected_theme_source_asset_id(), None);
}

#[test]
fn ui_asset_editor_session_projects_and_updates_promote_theme_draft_fields() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.theme_promote_asset_id,
        "res://ui/themes/theme_summary_theme.ui.toml"
    );
    assert_eq!(
        initial.theme_promote_document_id,
        "ui.theme.theme_summary_theme"
    );
    assert_eq!(initial.theme_promote_display_name, "Theme Summary Theme");
    assert!(initial.theme_can_edit_promote_draft);

    assert!(session
        .set_promote_theme_asset_id("res://ui/themes/custom/editor_shell.ui.toml")
        .expect("set promote theme asset id"));
    assert!(session
        .set_promote_theme_document_id("ui.theme.custom.editor_shell")
        .expect("set promote theme document id"));
    assert!(session
        .set_promote_theme_display_name("Editor Shell Theme")
        .expect("set promote theme display name"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.theme_promote_asset_id,
        "res://ui/themes/custom/editor_shell.ui.toml"
    );
    assert_eq!(
        updated.theme_promote_document_id,
        "ui.theme.custom.editor_shell"
    );
    assert_eq!(updated.theme_promote_display_name, "Editor Shell Theme");
}

#[test]
fn ui_asset_editor_session_projects_local_cascade_theme_helpers_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_ASSET_TOML).expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");

    let pane = session.pane_presentation();
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade tokens into local layer (1)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade rules into local layer (1)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade changes into local layer (2)".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade token • border = \"#223344\"".to_string()));
    assert!(pane
        .theme_rule_helper_items
        .contains(&"Adopt active cascade rule • shared_theme • Label".to_string()));

    let helper_index = pane
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt active cascade changes into local layer (2)")
        .expect("batch local cascade helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply local cascade helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("batch adopted local cascade source");
    assert_eq!(
        document.tokens.get("border"),
        Some(&Value::String("#223344".to_string()))
    );
    let imported_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "shared_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Label"))
        .expect("local adopted imported rule");
    assert_eq!(
        imported_rule.set.self_values.get("text"),
        Some(&Value::String("Imported Theme".to_string()))
    );
}

#[test]
fn ui_asset_editor_session_detaches_selected_imported_theme_into_local_theme_layer() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    assert!(session
        .detach_selected_theme_source_to_local()
        .expect("detach imported theme into local layer"));

    let pane = session.pane_presentation();
    assert_eq!(pane.theme_source_selected_index, 0);
    assert_eq!(pane.theme_selected_source_kind, "Local");
    assert_eq!(pane.theme_selected_source_reference, "local");
    assert_eq!(pane.theme_selected_source_token_count, 3);
    assert_eq!(pane.theme_selected_source_rule_count, 2);
    assert_eq!(
        pane.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_selected_source_rule_items,
        vec![
            "shared_theme_local_theme • Button".to_string(),
            "local_theme • #RootLabel".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_source_items,
        vec!["Local Theme • 3 tokens • 2 rules".to_string()]
    );

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("detached theme source");
    assert!(document.imports.styles.is_empty());
    assert_eq!(
        document.tokens.get("accent").and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(
        document
            .tokens
            .get("shared_theme_accent")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );
    assert_eq!(
        document.tokens.get("panel").and_then(toml::Value::as_str),
        Some("$shared_theme_accent")
    );
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
    assert_eq!(
        document.stylesheets[0].rules[0]
            .set
            .self_values
            .get("text")
            .and_then(toml::Value::as_str),
        Some("$panel")
    );
}

#[test]
fn ui_asset_editor_session_projects_theme_compare_rule_body_diffs() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-diff.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_RULE_DIFF_ASSET_TOML)
        .expect("imported diff theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme diff session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported diff theme");

    let local_compare = session.pane_presentation();
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("overrides imported • rule • local_theme • Button")));
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("imported self.background.color = \"$accent\"")));
    assert!(local_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("local self.text = \"$panel\"")));

    assert!(session
        .select_theme_source(1)
        .expect("select imported diff theme"));
    let imported_compare = session.pane_presentation();
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("shadowed by local • rule • local_theme • Button")));
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("self.text = \"Imported Theme\"")));
    assert!(imported_compare
        .theme_compare_items
        .iter()
        .any(|item| item.contains("local self.text = \"$panel\"")));
}

#[test]
fn ui_asset_editor_session_clones_selected_imported_theme_into_local_theme_layer() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    assert!(session
        .clone_selected_theme_source_to_local()
        .expect("clone imported theme into local layer"));

    let pane = session.pane_presentation();
    assert_eq!(pane.theme_source_selected_index, 0);
    assert_eq!(pane.theme_selected_source_kind, "Local");
    assert_eq!(pane.theme_selected_source_reference, "local");
    assert_eq!(
        pane.theme_source_items,
        vec![
            "Local Theme • 3 tokens • 2 rules".to_string(),
            "res://ui/theme/shared_theme.ui.toml • 2 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        pane.theme_selected_source_rule_items,
        vec![
            "shared_theme_local_theme • Button".to_string(),
            "local_theme • #RootLabel".to_string(),
        ]
    );

    let document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("cloned theme source");
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );
    assert_eq!(
        document.tokens.get("accent").and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(
        document
            .tokens
            .get("shared_theme_accent")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );
    assert_eq!(
        document.tokens.get("panel").and_then(toml::Value::as_str),
        Some("$shared_theme_accent")
    );
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_projects_and_applies_redundant_imported_theme_refactor_after_clone() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");
    assert!(session
        .clone_selected_theme_source_to_local()
        .expect("clone imported theme into local layer"));

    let before = session.pane_presentation();
    let redundant_index = before
        .theme_refactor_items
        .iter()
        .position(|item| item == "redundant imported theme • res://ui/theme/shared_theme.ui.toml")
        .expect("redundant imported theme refactor");

    assert!(session
        .apply_theme_refactor_item(redundant_index)
        .expect("remove redundant imported theme"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("cloned theme source without redundant import");
    assert!(document.imports.styles.is_empty());
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_projects_local_theme_layer_merge_preview_for_imported_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_MERGE_PREVIEW_ASSET_TOML)
        .expect("imported merge preview theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_merge_preview_items,
        vec![
            "Detach • imports • res://ui/theme/base_tokens.ui.toml".to_string(),
            "Detach • token • accent = \"#4488ff\"".to_string(),
            "Detach • token • panel = \"$shared_theme_accent\"".to_string(),
            "Detach • token • shared_theme_accent = \"#223344\"".to_string(),
            "Detach • rule • shared_theme_local_theme • Button".to_string(),
            "Detach • rule • local_theme • #RootLabel".to_string(),
            "Clone • imports • res://ui/theme/shared_theme.ui.toml, res://ui/theme/base_tokens.ui.toml"
                .to_string(),
            "Clone • token • accent = \"#4488ff\"".to_string(),
            "Clone • token • panel = \"$shared_theme_accent\"".to_string(),
            "Clone • token • shared_theme_accent = \"#223344\"".to_string(),
            "Clone • rule • shared_theme_local_theme • Button".to_string(),
            "Clone • rule • local_theme • #RootLabel".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_theme_compare_items_for_selected_imported_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.theme_compare_items,
        vec![
            "shadowed by local • token • accent • imported = \"#223344\" • local = \"#4488ff\""
                .to_string(),
            "imported-only • token • panel = \"$accent\"".to_string(),
            "imported-only • rule • local_theme • Button • self.text = \"$panel\"".to_string(),
            "local-only • rule • local_theme • #RootLabel • self.text = \"Theme Summary Local\""
                .to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_applies_theme_rule_helper_items_for_selected_imports() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let before = session.pane_presentation();
    assert_eq!(
        before.theme_rule_helper_items,
        vec![
            "Detach res://ui/theme/shared_theme.ui.toml into local theme layer".to_string(),
            "Clone res://ui/theme/shared_theme.ui.toml into local theme layer".to_string(),
            "Adopt compare diffs from selected theme (3)".to_string(),
            "Adopt all imported tokens (2)".to_string(),
            "Adopt all imported rules (1)".to_string(),
            "Adopt all imported changes (3)".to_string(),
            "Adopt imported token • accent = \"#223344\"".to_string(),
            "Adopt imported token • panel = \"$accent\"".to_string(),
            "Adopt imported rule • local_theme • Button".to_string(),
        ]
    );

    assert!(session
        .apply_theme_rule_helper_item(0)
        .expect("apply detach helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("detached theme source");
    assert!(document.imports.styles.is_empty());
    assert!(document.tokens.contains_key("shared_theme_accent"));
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );
}

#[test]
fn ui_asset_editor_session_applies_compare_diff_theme_helper_for_selected_import() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt compare diffs from selected theme (3)")
        .expect("compare diff helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply compare diff helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("compare diff adopted theme source");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    assert_eq!(
        document.tokens.get("panel"),
        Some(&Value::String("$accent".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("compare diff adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    let pane = session.pane_presentation();
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • accent = \"#223344\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • panel = \"$accent\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • rule • local_theme • Button"));
}

#[test]
fn ui_asset_editor_session_adopts_imported_theme_rule_body_helper_items() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let token_helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported token • accent = \"#223344\"")
        .expect("imported token helper");
    assert!(session
        .apply_theme_rule_helper_item(token_helper_index)
        .expect("apply imported token helper"));

    let token_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("token helper source");
    assert_eq!(
        token_document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );

    let rule_helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported rule • local_theme • Button")
        .expect("imported rule helper");
    assert!(session
        .apply_theme_rule_helper_item(rule_helper_index)
        .expect("apply imported rule helper"));

    let rule_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("rule helper source");
    let button_rule = rule_document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("local imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );
}

#[test]
fn ui_asset_editor_session_applies_theme_batch_adopt_helper_items() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-summary.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported collision theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_SUMMARY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme summary session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt all imported changes (3)")
        .expect("batch imported theme change helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply batch imported theme change helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("batch adopted theme source");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    assert_eq!(
        document.tokens.get("panel"),
        Some(&Value::String("$accent".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("batch adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    let pane = session.pane_presentation();
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • accent = \"#223344\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • token • panel = \"$accent\""));
    assert!(pane
        .theme_compare_items
        .iter()
        .any(|item| item == "shared • rule • local_theme • Button"));
}

#[test]
fn ui_asset_editor_session_prunes_selected_theme_compare_duplicates() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");
    session
        .select_theme_source(1)
        .expect("select imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Prune compare duplicates shared with selected theme (3)")
        .expect("compare prune helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply compare prune helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("compare pruned theme source");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_applies_theme_refactor_items_individually() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");

    let before = session.pane_presentation();
    assert_eq!(
        before.theme_refactor_items,
        vec![
            "duplicate local token • accent • inherited = \"#223344\"".to_string(),
            "duplicate local token • panel • inherited = \"$accent\"".to_string(),
            "duplicate local rule • local_theme • Button".to_string(),
            "redundant imported theme • res://ui/theme/shared_theme.ui.toml".to_string(),
        ]
    );

    assert!(session
        .apply_theme_refactor_item(0)
        .expect("remove duplicate token"));
    let after_token =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("token pruned source");
    assert!(!after_token.tokens.contains_key("accent"));
    assert!(after_token.tokens.contains_key("panel"));
    assert_eq!(after_token.stylesheets[0].rules.len(), 1);

    assert!(session
        .apply_theme_refactor_item(1)
        .expect("remove duplicate rule"));
    let after_rule =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("rule pruned source");
    assert!(!after_rule.tokens.contains_key("accent"));
    assert!(after_rule.tokens.contains_key("panel"));
    assert!(after_rule.stylesheets.is_empty());
}

#[test]
fn ui_asset_editor_session_applies_all_theme_refactors_from_helper() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-dedupe.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme = UiAssetLoader::load_toml_str(IMPORTED_THEME_COLLISION_ASSET_TOML)
        .expect("imported duplicate theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        DUPLICATE_LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme dedupe session");

    session
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
        .expect("register imported theme");

    let helper_index = session
        .pane_presentation()
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Apply all theme refactors (4)")
        .expect("batch theme refactor helper");
    assert!(session
        .apply_theme_rule_helper_item(helper_index)
        .expect("apply batch theme refactor helper"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text())
        .expect("batch refactored source");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert!(document.imports.styles.is_empty());
    assert!(session.pane_presentation().theme_refactor_items.is_empty());
}

#[test]
fn ui_asset_editor_session_projects_cross_asset_theme_rule_cascade_activity() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-multi-cascade.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme_a =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_CASCADE_A_ASSET_TOML).expect("theme a");
    let imported_theme_b =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_CASCADE_B_ASSET_TOML).expect("theme b");
    let mut session = UiAssetEditorSession::from_source(
        route,
        MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("multi cascade session");

    session
        .register_style_import("res://ui/theme/shared_a.ui.toml", imported_theme_a)
        .expect("register theme a");
    session
        .register_style_import("res://ui/theme/shared_b.ui.toml", imported_theme_b)
        .expect("register theme b");

    let pane = session.pane_presentation();
    assert!(pane
        .theme_cascade_token_items
        .contains(&"active • accent • Local = \"#5588ff\"".to_string()));
    assert!(pane.theme_cascade_token_items.contains(
        &"shadowed • accent • res://ui/theme/shared_b.ui.toml = \"#334455\"".to_string()
    ));
    assert!(pane.theme_cascade_token_items.contains(
        &"shadowed • accent • res://ui/theme/shared_a.ui.toml = \"#112233\"".to_string()
    ));
    assert!(pane.theme_cascade_rule_items.contains(
        &"active • rule • Button • Local • local_theme • self.text = \"Local Theme\"".to_string()
    ));
    assert!(pane
        .theme_cascade_rule_items
        .contains(
            &"shadowed • rule • Button • res://ui/theme/shared_b.ui.toml • shared_theme_b • self.text = \"Imported Theme B\""
                .to_string(),
        ));
    assert!(pane
        .theme_cascade_rule_items
        .contains(
            &"shadowed • rule • Button • res://ui/theme/shared_a.ui.toml • shared_theme_a • self.text = \"Imported Theme A\""
                .to_string(),
        ));
}

#[test]
fn ui_asset_editor_session_theme_compare_uses_active_imported_cascade_values() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/theme-multi-cascade.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme_a =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_CASCADE_A_ASSET_TOML).expect("theme a");
    let imported_theme_b =
        UiAssetLoader::load_toml_str(IMPORTED_THEME_CASCADE_B_ASSET_TOML).expect("theme b");
    let mut session = UiAssetEditorSession::from_source(
        route,
        MULTI_IMPORTED_THEME_CASCADE_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("multi cascade session");

    session
        .register_style_import("res://ui/theme/shared_a.ui.toml", imported_theme_a)
        .expect("register theme a");
    session
        .register_style_import("res://ui/theme/shared_b.ui.toml", imported_theme_b)
        .expect("register theme b");

    let pane = session.pane_presentation();
    assert!(pane.theme_compare_items.contains(
        &"overrides imported • token • accent • imported = \"#334455\" • local = \"#5588ff\""
            .to_string(),
    ));
    assert!(pane
        .theme_compare_items
        .contains(
            &"overrides imported • rule • local_theme • Button • imported self.text = \"Imported Theme B\" • local self.text = \"Local Theme\""
                .to_string(),
        ));
    assert!(!pane.theme_compare_items.contains(
        &"overrides imported • token • accent • imported = \"#112233\" • local = \"#5588ff\""
            .to_string(),
    ));
}

