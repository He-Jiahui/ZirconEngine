use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::{TemplateNodeFrameData, TemplatePaneNodeData};

#[test]
fn viewport_chrome_consumers_use_relocated_authored_root_extent() {
    let mut pane = PaneData {
        kind: "Scene".into(),
        show_toolbar: true,
        ..Default::default()
    };
    pane.viewport.toolbar_template_nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "SceneViewportToolbarRoot".into(),
        frame: TemplateNodeFrameData {
            x: 3.0,
            y: 2.0,
            width: 480.0,
            height: 36.0,
        },
        ..Default::default()
    }]);
    let content = FrameRect {
        x: 240.0,
        y: 100.0,
        width: 600.0,
        height: 400.0,
    };
    assert_eq!(
        viewport_toolbar_frame(&pane, &content),
        Some(FrameRect {
            x: 240.0,
            y: 100.0,
            width: 483.0,
            height: 38.0,
        })
    );
}
