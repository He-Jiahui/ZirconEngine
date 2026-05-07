use super::support::open_design_session;

const ACTION_POLICY_AND_LOCALIZATION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.runtime_reports"
version = 3
display_name = "Runtime Reports"

[root]
node_id = "root"
kind = "native"
type = "Label"
control_id = "FetchRemote"

[root.props]
text = { text_key = "editor.runtime_reports.title", table = "editor", fallback = "Runtime Reports", direction = "ltr" }
title = "Literal tooltip"

[[root.bindings]]
id = "Network/Fetch"
event = "Click"
route = "Network.Fetch"

[[root.bindings]]
id = "Asset/Save"
event = "Click"
route = "Asset.Save"

[[root.children]]
[root.children.node]
node_id = "status"
kind = "native"
type = "Label"
control_id = "Status"
props = { text = "Ready" }
"##;

#[test]
fn ui_asset_editor_projects_runtime_action_policy_and_localization_reports() {
    let session = open_design_session(
        "asset://ui/tests/runtime_reports.ui.toml",
        ACTION_POLICY_AND_LOCALIZATION_LAYOUT,
    );
    let pane = session.pane_presentation();

    assert!(pane.action_policy_items.iter().any(|item| {
        item.contains("Network") && item.contains("root") && item.contains("Network/Fetch")
    }));
    assert!(pane
        .host_enforcement_items
        .iter()
        .any(|item| item.contains("runtime-default")
            && item.contains("Asset/Save")
            && item.contains("AssetIo")
            && item.contains("blocked")));
    assert!(pane
        .host_enforcement_items
        .iter()
        .any(|item| item.contains("editor-authoring")
            && item.contains("Network/Fetch")
            && item.contains("Network")
            && item.contains("blocked")));
    assert!(pane
        .unsafe_action_guidance_items
        .iter()
        .any(|item| item.contains("Asset/Save")
            && item.contains("editor-only")
            && item.contains("runtime-default")));
    assert!(pane
        .unsafe_action_guidance_items
        .iter()
        .any(|item| item.contains("Network/Fetch")
            && item.contains("explicit host capability")
            && item.contains("editor-authoring")));
    assert!(pane.locale_dependency_items.iter().any(|item| {
        item.contains("nodes.root.props.text")
            && item.contains("editor.runtime_reports.title")
            && item.contains("LeftToRight")
    }));
    assert!(pane
        .locale_extraction_items
        .iter()
        .any(|item| item.contains("nodes.status.props.text") && item.contains("Ready")));
    assert!(pane.locale_diagnostic_items.is_empty());
}
