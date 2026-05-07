use super::support::*;

const RUNTIME_REPORT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.runtime_report_productization"
version = 1
display_name = "Runtime Report Productization"

[imports]
resources = [
  { kind = "image", uri = "res://ui/icons/runtime-report.svg", fallback = { mode = "optional" } },
]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "title" }, { child = "button" }]

[nodes.title]
kind = "native"
type = "Label"
control_id = "TitleLabel"
props = { text = { text_key = "editor.runtime_report.title", table = "editor", fallback = "Runtime Report", direction = "ltr" }, title = "Runtime report title" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "FetchButton"
props = { text = "Fetch", icon = "asset://ui/icons/fetch.svg" }
bindings = [
  { id = "FetchButton/onClick", event = "Click", route = "Network.Fetch" },
  { id = "FetchButton/saveAsset", event = "Click", route = "Asset.Save" },
]
"##;

#[test]
fn ui_asset_editor_projects_runtime_report_policy_locale_and_resource_rows() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/runtime-report.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        RUNTIME_REPORT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    let pane = session.pane_presentation();
    assert!(pane
        .action_policy_items
        .iter()
        .any(|item| item.contains("Network") && item.contains("FetchButton/onClick")));
    assert!(pane
        .capability_explanation_items
        .contains(&"allowed side effects: LocalUi, EditorMutation, AssetIo".to_string()));
    assert!(pane
        .capability_explanation_items
        .contains(&"blocked side effects: SceneMutation, ExternalProcess, Network".to_string()));
    assert!(pane
        .host_enforcement_items
        .iter()
        .any(|item| item.contains("runtime-default")
            && item.contains("FetchButton/saveAsset")
            && item.contains("AssetIo")
            && item.contains("blocked")));
    assert!(pane
        .host_enforcement_items
        .iter()
        .any(|item| item.contains("editor-authoring")
            && item.contains("FetchButton/onClick")
            && item.contains("Network")
            && item.contains("blocked")));
    assert!(pane
        .unsafe_action_guidance_items
        .iter()
        .any(|item| item.contains("FetchButton/saveAsset")
            && item.contains("editor-only")
            && item.contains("runtime-default")));
    assert!(pane
        .unsafe_action_guidance_items
        .iter()
        .any(|item| item.contains("FetchButton/onClick")
            && item.contains("explicit host capability")
            && item.contains("editor-authoring")));

    assert_eq!(pane.locale_preview_selected_locale, "authoring-fallback");
    assert_eq!(pane.locale_preview_selected_index, 0);
    assert!(pane
        .locale_preview_items
        .iter()
        .any(|item| item.contains("editor.runtime_report.title")));
    assert!(session.set_locale_preview("zh-CN"));
    let localized_pane = session.pane_presentation();
    assert_eq!(localized_pane.locale_preview_selected_locale, "zh-CN");
    assert_eq!(localized_pane.locale_preview_selected_index, 2);
    assert!(localized_pane.locale_preview_items[2].starts_with("zh-CN"));

    assert!(pane
        .locale_dependency_items
        .iter()
        .any(|item| item.contains("nodes.title.props.text") && item.contains("LeftToRight")));
    assert!(pane
        .locale_extraction_items
        .iter()
        .any(|item| item.contains("nodes.title.props.title")));

    assert!(pane
        .resource_dependency_items
        .iter()
        .any(|item| item.contains("DocumentImport") && item.contains("runtime-report.svg")));
    assert!(pane
        .resource_dependency_items
        .iter()
        .any(|item| item.contains("NodeProp") && item.contains("fetch.svg")));
    assert!(pane.resource_diagnostic_items.is_empty());
}
