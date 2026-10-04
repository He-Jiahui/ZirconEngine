use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::super::super::data::{
    TemplateNodeFrameData, TemplatePaneNodeData, WelcomePaneLayoutData,
};
use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
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
fn welcome_native_foundation_precedes_shared_template_controls_only() {
    assert!(native_content_precedes_template_nodes("Welcome"));
    for pane_kind in ["Hierarchy", "Assets", "AssetBrowser", "Inspector"] {
        assert!(
            !native_content_precedes_template_nodes(pane_kind),
            "{pane_kind} native overlays must remain after template content"
        );
    }
}

#[test]
fn welcome_native_header_text_is_recorded_after_overlapping_template_mount_surface() {
    let body = rect(0.0, 0.0, 640.0, 480.0);
    let header = rect(96.0, 88.0, 448.0, 48.0);
    let main_panel = rect(64.0, 48.0, 512.0, 384.0);
    let mut pane = PaneData {
        kind: "Welcome".into(),
        ..PaneData::default()
    };
    pane.welcome.layout = WelcomePaneLayoutData {
        has_nodes: true,
        outer_panel: Some(main_panel.clone()),
        main_panel: Some(main_panel),
        new_project_header_panel: Some(header.clone()),
        ..WelcomePaneLayoutData::default()
    };
    pane.template_v2.nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "WelcomeNewProjectHeaderPanel".into(),
        role: "Mount".into(),
        frame: TemplateNodeFrameData {
            x: header.x,
            y: header.y,
            width: header.width,
            height: header.height,
        },
        ..TemplatePaneNodeData::default()
    }])));

    let mut frame = HostRgbaFrame::recording_only(640, 480);
    draw_pane_content_layers(
        &mut frame,
        &pane,
        &body,
        &body,
        &HostPaneInteractionStateData::default(),
        &HostViewportImageSet::default(),
        None,
        None,
    );

    let commands = frame.into_recorded_commands();
    let mount_surface = commands
        .iter()
        .enumerate()
        .filter(|(_, command)| {
            command.frame == header && matches!(&command.kind, HostRecordedPaintKind::Quad { .. })
        })
        .map(|(index, _)| index)
        .last()
        .expect("the template header Mount paints an opaque surface");
    let visible_heading = commands
        .iter()
        .enumerate()
        .filter(|(_, command)| {
            matches!(
                &command.kind,
                HostRecordedPaintKind::Text { text, .. } if text == "New Project"
            )
        })
        .map(|(index, _)| index)
        .last()
        .expect("native New Project heading is recorded");

    assert!(
        visible_heading > mount_surface,
        "the final Welcome heading must render above its template Mount surface"
    );
}
