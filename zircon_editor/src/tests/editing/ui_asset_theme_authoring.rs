use crate::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use zircon_ui::{UiAssetKind, UiAssetLoader, UiSize};

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
            "1. Imported • res://ui/theme/shared_theme.ui.toml • 1 tokens • 1 rules"
                .to_string(),
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
            "1. Imported • res://ui/theme/shared_theme.ui.toml • shared_theme • Label"
                .to_string(),
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

    let document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("detached theme source");
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
