use super::*;
use crate::ui::retained_host::primitives::VecModel;
use std::rc::Rc;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn profile_geometry_exports_absolute_splitter_and_tab_frames() {
    let mut presentation = HostWindowPresentationData::default();
    presentation
        .host_scene_data
        .resize_layer
        .left_splitter_frame = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 4.0,
        height: 500.0,
    };
    presentation.host_scene_data.document_dock.region_frame = FrameRect {
        x: 120.0,
        y: 80.0,
        width: 600.0,
        height: 400.0,
    };
    presentation.host_scene_data.document_dock.header_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 600.0,
        height: 32.0,
    };
    presentation.host_scene_data.document_dock.tab_frames =
        ModelRc::from(Rc::new(VecModel::from(vec![HostChromeTabData {
            control_id: "scene-tab".into(),
            frame: FrameRect {
                x: 12.0,
                y: 4.0,
                width: 120.0,
                height: 24.0,
            },
            ..HostChromeTabData::default()
        }])));

    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(1280, 720),
        HostPresenterBackend::Gpu,
    );

    assert_eq!(geometry.presenter_backend, "gpu");
    assert_eq!(geometry.resize_splitters.len(), 1);
    assert_eq!(geometry.document_tabs.len(), 1);
    assert_eq!(geometry.document_tabs[0].frame.x, 132.0);
    assert_eq!(geometry.document_tabs[0].frame.y, 84.0);
    assert!(geometry
        .hit_samples
        .iter()
        .any(|sample| sample.id == "scene-tab" && sample.expected_hit));
}

#[test]
fn profile_geometry_omits_template_controls_disjoint_from_clip() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.region_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    };
    presentation.host_scene_data.document_dock.content_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    };
    presentation.host_scene_data.document_dock.pane.kind = "Project".into();
    presentation
        .host_scene_data
        .document_dock
        .pane
        .project_overview
        .nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "OffClipAction".into(),
        action_id: "workbench.project.off_clip_action".into(),
        frame: super::super::data::TemplateNodeFrameData {
            x: 120.0,
            y: 120.0,
            width: 80.0,
            height: 24.0,
        },
        has_clip_frame: true,
        clip_frame: super::super::data::TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    }])));

    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(320, 240),
        HostPresenterBackend::Gpu,
    );

    assert!(geometry
        .template_controls
        .iter()
        .all(|frame| { frame.id != "template.document.OffClipAction" }));
    assert!(geometry
        .hit_samples
        .iter()
        .all(|sample| sample.id != "template.document.OffClipAction"));
}

#[test]
fn profile_geometry_clips_viewport_toolbar_controls_to_surface_clip() {
    let surface_frame = viewport_toolbar_surface_frame_for_test(vec![(
        2,
        "partial",
        UiFrame::new(90.0, 0.0, 30.0, 20.0),
    )]);
    let mut controls = Vec::new();

    collect_surface_frame_controls(
        "viewport_toolbar_control",
        "document",
        &FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 28.0,
        },
        Some(&surface_frame),
        &mut controls,
    );

    assert_eq!(controls.len(), 1);
    assert_eq!(controls[0].frame.x, 100.0);
    assert_eq!(controls[0].frame.width, 10.0);
}

#[test]
fn profile_geometry_omits_viewport_toolbar_controls_not_top_hit_at_center() {
    let surface_frame = viewport_toolbar_surface_frame_for_test(vec![
        (2, "covered", UiFrame::new(0.0, 0.0, 80.0, 20.0)),
        (3, "top", UiFrame::new(0.0, 0.0, 80.0, 20.0)),
    ]);
    let mut controls = Vec::new();

    collect_surface_frame_controls(
        "viewport_toolbar_control",
        "document",
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 28.0,
        },
        Some(&surface_frame),
        &mut controls,
    );

    assert!(controls
        .iter()
        .all(|frame| frame.id != "viewport_toolbar_control.document.covered"));
    assert!(controls
        .iter()
        .any(|frame| frame.id == "viewport_toolbar_control.document.top"));
}

fn viewport_toolbar_surface_frame_for_test(nodes: Vec<(u64, &str, UiFrame)>) -> UiSurfaceFrame {
    let mut surface = UiSurface::new(UiTreeId::new("test.viewport_toolbar_profile"));
    let root_frame = UiFrame::new(0.0, 0.0, 100.0, 28.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for (node_id, control_id, frame) in nodes {
        let node = UiTreeNode::new(
            UiNodeId::new(node_id),
            UiNodePath::new(format!("root/{control_id}")),
        )
        .with_frame(frame)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(UiTemplateNodeMetadata {
            control_id: Some(control_id.to_string()),
            ..Default::default()
        });
        surface.tree.insert_child(UiNodeId::new(1), node).unwrap();
    }
    surface.rebuild();
    surface.surface_frame()
}
