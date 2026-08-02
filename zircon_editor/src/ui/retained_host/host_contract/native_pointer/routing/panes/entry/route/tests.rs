use crate::ui::retained_host::console_output::{ConsoleOutputPaintMetadata, ConsoleOutputViewport};
use crate::ui::retained_host::host_contract::data::{
    ConsolePaneData, FloatingWindowData, FrameRect, HostBottomDockSurfaceData,
    HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::{ModelRc, model_rc};

use super::super::super::super::PanePointerTarget;
use super::route_pointer_scroll_to_pane;

#[test]
fn floating_console_occludes_local_scroll_route_outside_its_output_viewport() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 60.0,
        },
        20.0,
        0,
        1,
    )
    .expect("console output metadata");
    let console_nodes = ModelRc::with_metadata(vec![TemplatePaneNodeData::default()], metadata);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: frame(0.0, 0.0, 400.0, 300.0),
        content_frame: frame(0.0, 0.0, 400.0, 300.0),
        pane: PaneData {
            kind: "Hierarchy".into(),
            ..PaneData::default()
        },
        ..HostBottomDockSurfaceData::default()
    };
    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            window_id: "floating-console".into(),
            frame: frame(50.0, 50.0, 200.0, 150.0),
            header_frame: frame(0.0, 0.0, 200.0, 30.0),
            active_pane: PaneData {
                kind: "Console".into(),
                console: ConsolePaneData {
                    nodes: console_nodes,
                    status_text: "message".into(),
                },
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        }]);

    let route = route_pointer_scroll_to_pane(&presentation, 55.0, 90.0);

    assert!(route.is_none(), "floating content must consume the miss");
    assert_ne!(
        route.map(|route| route.target),
        Some(PanePointerTarget::Hierarchy)
    );
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
