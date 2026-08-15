use super::support::{
    active_ui_debug_snapshot_fixture, chrome_fixture, pane_body_spec, runtime_diagnostics_fixture,
};
use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

use crate::ui::layouts::windows::workbench_host_window::{
    PanePayload, PanePayloadBuildContext, build_pane_body_presentation,
};

#[test]
fn runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available() {
    let chrome = chrome_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let active_snapshot = active_ui_debug_snapshot_fixture();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_active_ui_debug_snapshot(&active_snapshot);

    let body =
        build_pane_body_presentation(&pane_body_spec("editor.runtime_diagnostics"), &context);

    let PanePayload::RuntimeDiagnosticsV1(payload) = body.payload else {
        panic!("expected runtime diagnostics payload");
    };

    assert_eq!(payload.summary, "3 runtime systems available");
    assert_eq!(
        payload.ui_debug_reflector_summary,
        "UI Debug Reflector: 2 nodes, 3 commands, schema v1"
    );
    assert!(
        payload
            .ui_debug_reflector_nodes
            .iter()
            .any(|node| node.contains("runtime/root/live_button") && node.contains("node=2"))
    );
    assert!(
        payload
            .ui_debug_reflector_details
            .iter()
            .any(|detail| detail.contains("Selected: runtime/root/live_button"))
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "Layout Engine:")
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "Canvas Layers:")
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "  parent=1 layer=0 z=1 children=[2]")
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "Pipeline:")
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "ECS Projection:")
    );
    assert!(
        payload
            .ui_debug_reflector_sections
            .iter()
            .any(|line| line == "  selected: taffy=1 zircon=1")
    );
    assert!(payload.ui_debug_reflector_sections.iter().any(|line| {
        line.contains("node=2")
            && line.contains("family=Overlay")
            && line.contains("selected=Zircon")
            && line.contains("reason=ZirconOwnedSemantics")
    }));
    assert!(
        payload
            .ui_debug_reflector_export_status
            .contains("JSON export ready")
    );
    assert_eq!(payload.ui_debug_reflector_overlay_primitives.len(), 1);
    assert_eq!(
        payload.ui_debug_reflector_overlay_primitives[0].kind,
        UiDebugOverlayPrimitiveKind::SelectedFrame
    );
    assert!(payload.ui_debug_reflector_has_active_snapshot);
}
