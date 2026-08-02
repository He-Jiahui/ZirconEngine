use crate::ui::retained_host::console_output::{ConsoleOutputPaintMetadata, ConsoleOutputViewport};
use crate::ui::retained_host::host_contract::data::{
    ConsolePaneData, FrameRect, PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::PanePointerTarget;
use super::super::super::mode::PaneRouteMode;
use super::pane_route_from_pane;

#[test]
fn console_scroll_route_uses_projected_output_viewport_over_template_leaf_hits() {
    let content = frame(100.0, 50.0, 300.0, 180.0);
    let pointer = pane_route_from_pane(
        &console_pane(),
        &content,
        120.0,
        100.0,
        None,
        PaneRouteMode::PointerScroll,
    )
    .expect("console output scroll route");

    assert!(matches!(pointer.target, PanePointerTarget::Console));
    assert_eq!(pointer.frame, frame(108.0, 84.0, 284.0, 146.0));
    assert_eq!(pointer.local_x, 12.0);
    assert_eq!(pointer.local_y, 16.0);
    assert_eq!(pointer.width, 284.0);
    assert_eq!(pointer.height, 146.0);
}

fn console_pane() -> PaneData {
    let nodes = vec![TemplatePaneNodeData {
        node_id: "ConsoleBodySection".into(),
        control_id: "ConsoleBodySection".into(),
        role: "VerticalGroup".into(),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 34.0,
            width: 284.0,
            height: 146.0,
        },
        ..TemplatePaneNodeData::default()
    }];
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 8.0,
            y: 34.0,
            width: 284.0,
            height: 146.0,
        },
        34.0,
        1,
        10,
    )
    .expect("console output metadata");

    PaneData {
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes: ModelRc::with_metadata(nodes, metadata),
            status_text: "line".into(),
        },
        ..PaneData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
