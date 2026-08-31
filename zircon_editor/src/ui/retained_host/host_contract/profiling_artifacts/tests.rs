use std::rc::Rc;

use super::super::data::{
    FrameRect, HostChromeTabData, HostWindowPresentationData, TemplateNodeFrameData,
    TemplatePaneNodeData,
};
use super::super::presenter::HostPresenterBackend;
use super::geometry::collect_surface_frame_controls;
use super::UiProfileGeometry;
use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize, VecModel};
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, AssetContentPaintNodeInput, AssetContentSurface,
};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiSurfaceFrame,
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
        frame: TemplateNodeFrameData {
            x: 120.0,
            y: 120.0,
            width: 80.0,
            height: 24.0,
        },
        has_clip_frame: true,
        clip_frame: TemplateNodeFrameData {
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
fn profile_geometry_exports_the_source_bound_asset_browser_content_viewport() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.region_frame = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 640.0,
        height: 480.0,
    };
    presentation.host_scene_data.document_dock.content_frame = FrameRect {
        x: 5.0,
        y: 30.0,
        width: 620.0,
        height: 430.0,
    };
    presentation.host_scene_data.document_dock.pane.kind = "AssetBrowser".into();
    let nodes = vec![TemplatePaneNodeData {
        control_id: "AssetBrowserThumbGridPanel".into(),
        frame: TemplateNodeFrameData {
            x: 40.0,
            y: 60.0,
            width: 320.0,
            height: 220.0,
        },
        ..TemplatePaneNodeData::default()
    }];
    let metadata = asset_content_paint_metadata(
        nodes.iter().map(|node| {
            AssetContentPaintNodeInput::new(
                node.control_id.as_str(),
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
                node.value_number,
            )
        }),
        AssetContentSurface::Browser,
    );
    presentation
        .host_scene_data
        .document_dock
        .pane
        .asset_browser
        .nodes = ModelRc::with_metadata(nodes, metadata);

    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(1280, 720),
        HostPresenterBackend::Gpu,
    );

    let content = geometry
        .asset_browser_content_frame
        .expect("asset browser content viewport should be exported");
    assert_eq!(content.id, "asset_browser.content_viewport");
    assert_eq!(content.kind, "asset_browser_content_viewport");
    assert_eq!(content.surface, "document");
    assert_eq!(content.frame.x, 55.0);
    assert_eq!(content.frame.y, 110.0);
    assert_eq!(content.frame.width, 320.0);
    assert_eq!(content.frame.height, 220.0);
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
    surface.surface_frame().as_ref().clone()
}
