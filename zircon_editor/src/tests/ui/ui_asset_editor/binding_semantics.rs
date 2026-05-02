use crate::ui::asset_editor::UiAssetEditorDiagnosticSeverity;

use super::support::open_design_session;

const INVALID_BINDING_VALUE_KIND_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.invalid_kind"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "BindingRoot"
props = { text = "Ready" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "prop", name = "text" }
expression = "true"
"##;

#[test]
fn ui_asset_editor_projects_runtime_binding_diagnostic_and_schema_items() {
    let session = open_design_session(
        "asset://ui/binding_invalid_kind.ui.toml",
        INVALID_BINDING_VALUE_KIND_LAYOUT,
    );

    let diagnostic = session
        .structured_diagnostics()
        .first()
        .expect("runtime binding diagnostic");
    assert_eq!(diagnostic.code, "invalid_value_kind");
    assert_eq!(diagnostic.severity, UiAssetEditorDiagnosticSeverity::Error);
    assert_eq!(
        diagnostic.source_path,
        "root.bindings[0].targets[0].expression"
    );
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));
    assert_eq!(
        diagnostic.target_binding_id.as_deref(),
        Some("Root/onClick")
    );

    let pane = session.pane_presentation();
    assert!(pane
        .structured_diagnostic_items
        .iter()
        .any(|item| item.contains("invalid_value_kind")));
    assert!(pane
        .inspector_binding_schema_items
        .iter()
        .any(|item| item == "target[0] [prop.text] = true"));
    assert!(pane
        .inspector_binding_schema_items
        .iter()
        .any(|item| item.contains("diagnostic [invalid_value_kind]")));
}
