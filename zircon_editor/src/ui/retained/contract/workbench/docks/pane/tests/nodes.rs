use std::rc::Rc;

use crate::ui::retained_host::primitives::VecModel;

use super::*;

#[test]
fn template_content_remains_present_outside_damage() {
    let nodes = ModelRc::from(Rc::new(VecModel::from(vec![
        TemplatePaneNodeData::default(),
    ])));
    let mut pane = PaneData::default();
    pane.template_v2.nodes = nodes;
    let body = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
    };
    let mut frame = HostRgbaFrame::recording_only(300, 300);
    frame.replace_paint_clip(Some(FrameRect {
        x: 200.0,
        y: 200.0,
        width: 20.0,
        height: 20.0,
    }));

    assert!(draw_pane_template_nodes(
        &mut frame,
        &pane,
        &body,
        &body,
        &HostPaneInteractionStateData::default(),
        None,
    ));
    assert!(frame.into_recorded_commands().is_empty());
}
