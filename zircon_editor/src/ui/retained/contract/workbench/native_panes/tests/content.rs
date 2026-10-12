use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::super::data::{HierarchyPaneData, SceneNodeData, WelcomePaneData};
use super::*;

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn native_content_clip_is_limited_to_frame_damage() {
    let pane = rect(10.0, 20.0, 100.0, 80.0);

    assert_eq!(
        effective_native_clip(&pane, Some(&rect(40.0, 30.0, 20.0, 25.0))),
        Some(rect(40.0, 30.0, 20.0, 25.0))
    );
    assert_eq!(effective_native_clip(&pane, None), Some(pane));
}

#[test]
fn native_content_clip_rejects_disjoint_damage() {
    assert!(effective_native_clip(
        &rect(10.0, 20.0, 100.0, 80.0),
        Some(&rect(200.0, 200.0, 20.0, 20.0)),
    )
    .is_none());
}

#[test]
fn disjoint_damage_reports_logical_native_content_without_hiding_empty_states() {
    let body = rect(10.0, 20.0, 100.0, 80.0);
    let interaction = HostPaneInteractionStateData::default();
    let mut frame = HostRgbaFrame::filled(320, 240, [0, 0, 0, 0]);
    frame.replace_paint_clip(Some(rect(200.0, 200.0, 20.0, 20.0)));

    let welcome = PaneData {
        kind: "Welcome".into(),
        welcome: WelcomePaneData {
            title: "Welcome".into(),
            ..WelcomePaneData::default()
        },
        ..PaneData::default()
    };
    assert!(draw_native_pane_content(
        &mut frame,
        &welcome,
        &body,
        &body,
        &interaction,
        None,
    ));

    let hierarchy = PaneData {
        kind: "Hierarchy".into(),
        hierarchy: HierarchyPaneData {
            hierarchy_nodes: ModelRc::from(Rc::new(VecModel::from(vec![SceneNodeData::default()]))),
            ..HierarchyPaneData::default()
        },
        ..PaneData::default()
    };
    assert!(draw_native_pane_content(
        &mut frame,
        &hierarchy,
        &body,
        &body,
        &interaction,
        None,
    ));

    let mut full_frame = HostRgbaFrame::recording_only(320, 240);
    for pane_kind in [
        "Welcome",
        "Hierarchy",
        "Assets",
        "AssetBrowser",
        "Inspector",
    ] {
        let empty = PaneData {
            kind: pane_kind.into(),
            ..PaneData::default()
        };
        assert!(!draw_native_pane_content(
            &mut frame,
            &empty,
            &body,
            &body,
            &interaction,
            None,
        ));
        assert!(!draw_native_pane_content(
            &mut full_frame,
            &empty,
            &body,
            &body,
            &interaction,
            None,
        ));
    }
}
