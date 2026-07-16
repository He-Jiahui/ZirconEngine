#![cfg(feature = "ui")]

//! Public Runtime behavior gate for descriptor-authoritative cross-control bindings.

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::template::{collect_asset_binding_report, UiAssetLoader};
use zircon_runtime_interface::ui::template::UiBindingDiagnosticCode;

fn target_layout(expression: &str, source_type: &str) -> String {
    format!(
        r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref.integration"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Container"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "source"
kind = "native"
type = {source_type:?}
control_id = "BindingSource"
props = {{ text = "Ready" }}

[[root.children]]
[root.children.node]
node_id = "consumer"
kind = "native"
type = "Button"
control_id = "BindingConsumer"
props = {{ text = "Apply" }}

[[root.children.node.bindings]]
id = "Consumer/onClick"
event = "Click"
route = "Route.Valid"

[[root.children.node.bindings.targets]]
target = {{ kind = "visibility" }}
expression = {expression:?}
"##
    )
}

const ACTION_PAYLOAD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref.payload.integration"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Container"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "entity_source"
kind = "native"
type = "TreeView"
control_id = "BindingEntitySource"
props = { selected_index = 7 }

[[root.children]]
[root.children.node]
node_id = "force_source"
kind = "native"
type = "Checkbox"
control_id = "BindingForceSource"
props = { checked = true }

[[root.children]]
[root.children.node]
node_id = "consumer"
kind = "native"
type = "Button"
control_id = "BindingConsumer"
props = { text = "Apply" }

[[root.children.node.bindings]]
id = "Consumer/onClick"
event = "Click"
route = "Route.Valid"

[root.children.node.bindings.action]
route = "Route.Valid"

[root.children.node.bindings.action.payload]
surface_entity = "=control.BindingEntitySource.prop.selected_index"
force_full_rebuild = "=control.BindingForceSource.prop.checked"
"##;

const COMPONENT_SCOPE_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.binding.control_prop_ref.scope.integration"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "RootConsumer"
props = { text = "Apply" }

[[root.bindings]]
id = "Root/onClick"
event = "Click"
route = "Route.Invalid"

[[root.bindings.targets]]
target = { kind = "visibility" }
expression = 'control.ComponentSource.prop.checked'

[components.ScopedSource.root]
node_id = "component_source"
kind = "native"
type = "Checkbox"
control_id = "ComponentSource"
props = { checked = true }
"##;

fn report_for(source: &str) -> zircon_runtime_interface::ui::template::UiBindingReport {
    let document = UiAssetLoader::load_toml_str(source).unwrap();
    collect_asset_binding_report(&document, &UiComponentDescriptorRegistry::editor_showcase())
}

#[test]
fn resolves_known_control_property_kind() {
    let report = report_for(&target_layout(
        "control.BindingSource.prop.text == \"Ready\"",
        "Label",
    ));

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn reports_unknown_control_and_property() {
    for (expression, expected_message) in [
        (
            "control.MissingSource.prop.text == \"Ready\"",
            "unknown control MissingSource",
        ),
        (
            "control.BindingSource.prop.missing == \"Ready\"",
            "control BindingSource references unknown prop missing",
        ),
    ] {
        let report = report_for(&target_layout(expression, "Label"));

        assert_eq!(report.diagnostics.len(), 1, "{expression}");
        assert_eq!(
            report.diagnostics[0].code,
            UiBindingDiagnosticCode::UnresolvedRef
        );
        assert!(report.diagnostics[0].message.contains(expected_message));
    }
}

#[test]
fn reports_target_kind_mismatch() {
    let report = report_for(&target_layout("control.BindingSource.prop.text", "Label"));

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::InvalidValueKind
    );
}

#[test]
fn validates_navigation_action_payload_kinds() {
    let report = report_for(ACTION_PAYLOAD_LAYOUT);

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn rejects_authored_prop_kind_without_a_descriptor() {
    let report = report_for(&target_layout(
        "control.BindingSource.prop.text == \"Ready\"",
        "MissingWidget",
    ));

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
}

#[test]
fn isolates_control_refs_to_the_current_component_tree() {
    let report = report_for(COMPONENT_SCOPE_LAYOUT);

    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        UiBindingDiagnosticCode::UnresolvedRef
    );
    assert!(report.diagnostics[0]
        .message
        .contains("unknown control ComponentSource"));
}
