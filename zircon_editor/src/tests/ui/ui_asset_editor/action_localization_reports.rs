use crate::ui::asset_editor::UiAssetEditorDiagnosticSeverity;

use super::support::open_design_session;

const ACTION_POLICY_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.policy.network"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "SyncNow"
props = { text = "Sync" }

[[root.bindings]]
id = "SyncNow/onClick"
event = "Click"
route = "Route.Network.Sync"
"##;

const LOCALIZATION_REPORT_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.localization.report"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "LocalizedRoot"
props = {
    text = { text_key = "", table = "ui" },
    label = { text_key = "menu.play", table = "ui", fallback = "Play", direction = "rtl" },
    title = "Ready"
}
"##;

const LOCALIZATION_RESOLVER_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.localization.resolver"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "LocalizedRoot"
props = { label = { text_key = "menu.play", table = "ui", fallback = "Play" } }
"##;

#[test]
fn ui_asset_editor_projects_action_policy_report_items() {
    let session = open_design_session("asset://ui/policy_network.ui.toml", ACTION_POLICY_LAYOUT);
    let pane = session.pane_presentation();

    assert!(pane
        .action_policy_items
        .iter()
        .any(|item| item.contains("SyncNow/onClick") && item.contains("Network")));
    assert!(pane
        .action_policy_items
        .iter()
        .any(|item| item.contains("Route.Network.Sync")));
}

#[test]
fn ui_asset_editor_projects_localization_report_items_and_diagnostics() {
    let session = open_design_session(
        "asset://ui/localization_report.ui.toml",
        LOCALIZATION_REPORT_LAYOUT,
    );

    let diagnostic = session
        .structured_diagnostics()
        .first()
        .expect("localized ref diagnostic");
    assert_eq!(diagnostic.code, "localization_invalid_ref");
    assert_eq!(diagnostic.severity, UiAssetEditorDiagnosticSeverity::Error);
    assert_eq!(diagnostic.source_path, "nodes.root.props.text");
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));

    let pane = session.pane_presentation();
    assert!(pane
        .locale_dependency_items
        .iter()
        .any(|item| item.contains("menu.play")
            && item.contains("ui")
            && item.contains("RightToLeft")));
    assert!(pane
        .locale_extraction_items
        .iter()
        .any(|item| item.contains("nodes.root.props.title") && item.contains("\"Ready\"")));
    assert!(pane
        .locale_diagnostic_items
        .iter()
        .any(|item| item.contains("nodes.root.props.text")));
}

#[test]
fn ui_asset_editor_projects_locale_table_missing_key_diagnostics() {
    let mut session = open_design_session(
        "asset://ui/localization_resolver.ui.toml",
        LOCALIZATION_RESOLVER_LAYOUT,
    );
    assert!(session.structured_diagnostics().is_empty());

    assert!(session.set_locale_preview("en-US"));

    assert!(session
        .pane_presentation()
        .locale_diagnostic_items
        .iter()
        .any(|item| item.contains("missing_locale_table")
            && item.contains("en-US/ui")
            && item.contains("menu.play")));

    session.register_locale_table_keys(
        "en-US",
        "ui",
        Some("res://locales/en-US/ui.toml".to_string()),
        ["menu.stop"],
    );

    let pane = session.pane_presentation();
    assert!(pane
        .locale_diagnostic_items
        .iter()
        .any(|item| item.contains("missing_locale_key")
            && item.contains("menu.play")
            && item.contains("res://locales/en-US/ui.toml")));
    assert!(session
        .structured_diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "missing_locale_key"
            && diagnostic.target_node_id.as_deref() == Some("root")));

    session.register_locale_table_keys(
        "en-US",
        "ui",
        Some("res://locales/en-US/ui.toml".to_string()),
        ["menu.play"],
    );

    assert!(session
        .pane_presentation()
        .locale_diagnostic_items
        .is_empty());
    assert!(session.structured_diagnostics().is_empty());
}
