use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityDiagnostic, UiAccessibilityDiagnosticCode,
        UiAccessibilityDiagnosticSeverity, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    event_ui::{UiNodeId, UiTreeId},
};

use crate::core::framework::render::CapturedFrame;
use crate::core::math::UVec2;

pub(in crate::dynamic_api::session) fn empty_captured_frame(size: UVec2) -> CapturedFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let rgba = vec![0; width as usize * height as usize * 4];
    CapturedFrame::new(width, height, rgba, 0)
}

pub(in crate::dynamic_api::session) fn dynamic_preview_accessibility_snapshot(
) -> UiAccessibilityTreeSnapshot {
    let root = UiNodeId::new(1);
    UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("zircon-runtime-dynamic-preview"),
        roots: vec![root],
        nodes: vec![UiAccessibilityNode {
            node_id: root,
            role: UiA11yRole::Panel,
            name: Some("Zircon Runtime Preview".to_string()),
            ..UiAccessibilityNode::default()
        }],
        focused: None,
        diagnostics: vec![UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Info,
            code: UiAccessibilityDiagnosticCode::MissingBounds,
            node_id: Some(root),
            message: "runtime UI surface accessibility extraction unavailable in dynamic preview"
                .to_string(),
        }],
    }
}
