use super::support::{chrome_fixture, editor_data_with_drawer_fixture, pane_body_spec};

use crate::ui::layouts::windows::workbench_host_window::{
    PanePayload, PanePayloadBuildContext, build_pane_body_presentation,
};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

#[test]
fn inspector_payload_preserves_inspector_customization_template_metadata() {
    let chrome = EditorChromeSnapshot {
        inspector: editor_data_with_drawer_fixture().inspector,
        ..chrome_fixture()
    };
    let context = PanePayloadBuildContext::new(&chrome);
    let body = build_pane_body_presentation(&pane_body_spec("editor.inspector"), &context);

    let PanePayload::InspectorV1(payload) = body.payload else {
        panic!("expected inspector payload");
    };
    let component = payload
        .plugin_components
        .iter()
        .find(|component| component.component_id == "weather.Component.CloudLayer")
        .expect("drawer component projected");

    assert_eq!(
        component.customization_ui_document.as_deref(),
        Some("asset://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        component.customization_controller.as_deref(),
        Some("weather.editor.CloudLayerInspectorController")
    );
    assert_eq!(
        component.customization_template_id.as_deref(),
        Some("weather.cloud_layer.inspector")
    );
    assert_eq!(
        component.customization_bindings,
        ["weather.cloud_layer.refresh"]
    );
}
