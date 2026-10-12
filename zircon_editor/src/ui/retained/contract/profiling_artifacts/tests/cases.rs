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
fn zui_visual_acceptance_profile_geometry_records_winit_scale_and_logical_extent_separately() {
    let geometry = UiProfileGeometry::from_presentation(
        &HostWindowPresentationData::default(),
        &PhysicalSize::new(1920, 1200),
        1.5,
        HostPresenterBackend::Gpu,
    );

    assert_eq!(geometry.window_client_size.width, 1920);
    assert_eq!(geometry.window_client_size.height, 1200);
    assert_eq!(geometry.winit_scale_factor, 1.5);
    assert_eq!(geometry.window_client_logical_size.width, 1280.0);
    assert_eq!(geometry.window_client_logical_size.height, 800.0);

    let json = serde_json::to_value(&geometry).expect("profile geometry should serialize");
    assert_eq!(json["window_client_size"]["width"], 1920);
    assert_eq!(json["window_client_size"]["height"], 1200);
    assert_eq!(json["winit_scale_factor"], 1.5);
    assert_eq!(
        json["window_client_logical_size"]["width"].as_f64(),
        Some(1280.0)
    );
    assert_eq!(
        json["window_client_logical_size"]["height"].as_f64(),
        Some(800.0)
    );
}

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
        1.0,
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
        1.0,
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
        1.0,
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

fn unreal_profile_presentation() -> HostWindowPresentationData {
    let node = |id: &str, parent: &str, control: &str, x, y, width, height| TemplatePaneNodeData {
        node_id: id.into(),
        parent_node_id: parent.into(),
        control_id: control.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..Default::default()
    };
    let mut move_control = node(
        "viewport/tools/move",
        "viewport/tools",
        "WorkbenchToolMove",
        8.0,
        90.0,
        80.0,
        24.0,
    );
    move_control.action_id = "WorkbenchSceneMode/Move".into();
    move_control.component_role = "button".into();
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            node("root", "", "WorkbenchWindowRoot", 0.0, 0.0, 1280.0, 800.0),
            node(
                "content",
                "root",
                "UnrealLevelEditorContent",
                0.0,
                74.0,
                1280.0,
                626.0,
            ),
            node(
                "viewport",
                "content",
                "WorkbenchMainBandViewportPanel",
                0.0,
                80.0,
                800.0,
                520.0,
            ),
            node(
                "viewport/media",
                "viewport",
                "WorkbenchViewportSurface",
                0.0,
                120.0,
                800.0,
                480.0,
            ),
            node(
                "viewport/tools",
                "viewport",
                "UnrealViewportToolbar",
                0.0,
                86.0,
                800.0,
                28.0,
            ),
            move_control,
        ]))),
        ..Default::default()
    };
    let old_tab = |id: &str| {
        ModelRc::from(Rc::new(VecModel::from(vec![HostChromeTabData {
            control_id: id.into(),
            frame: FrameRect {
                x: 4.0,
                y: 2.0,
                width: 90.0,
                height: 24.0,
            },
            ..Default::default()
        }])))
    };
    let scene = &mut presentation.host_scene_data;
    scene.document_dock.surface_key = "document".into();
    scene.document_dock.pane.kind = "Scene".into();
    scene.document_dock.pane.show_toolbar = true;
    scene.document_dock.region_frame = FrameRect {
        x: 0.0,
        y: 80.0,
        width: 800.0,
        height: 520.0,
    };
    scene.document_dock.content_frame = FrameRect {
        x: 0.0,
        y: 30.0,
        width: 800.0,
        height: 490.0,
    };
    scene.document_dock.tab_frames = old_tab("old-scene-tab");
    scene.document_dock.pane.viewport.toolbar_surface_frame = Some(std::sync::Arc::new(
        viewport_toolbar_surface_frame_for_test(vec![(
            2,
            "old-tool",
            UiFrame::new(0.0, 0.0, 80.0, 20.0),
        )]),
    ));
    for (dock, id) in [
        (&mut scene.left_dock, "old-left-tab"),
        (&mut scene.right_dock, "old-right-tab"),
    ] {
        dock.tab_frames = old_tab(id);
        dock.region_frame = FrameRect {
            x: 900.0,
            y: 80.0,
            width: 200.0,
            height: 520.0,
        };
        dock.rail_width_px = 32.0;
        dock.rail_button_frames = ModelRc::from(Rc::new(VecModel::from(vec![
            super::super::data::HostChromeControlFrameData {
                control_id: format!("{id}-rail").into(),
                frame: FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 32.0,
                    height: 32.0,
                },
            },
        ])));
    }
    scene.page_chrome.tab_frames = old_tab("old-host-page-tab");
    scene.bottom_dock.tab_frames = old_tab("live-bottom-tab");
    presentation.host_layout.viewport_content_frame = FrameRect {
        x: 0.0,
        y: 150.0,
        width: 100.0,
        height: 80.0,
    };
    presentation
}

#[test]
fn unreal_profile_uses_authored_viewport_toolbar_and_omits_retired_native_chrome() {
    let presentation = unreal_profile_presentation();
    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(1280, 800),
        1.0,
        HostPresenterBackend::Gpu,
    );
    assert!(geometry.document_tabs.is_empty());
    assert!(geometry.host_page_tabs.is_empty());
    assert!(geometry.activity_rail_buttons.is_empty());
    assert_eq!(
        geometry
            .drawer_tabs
            .iter()
            .map(|tab| tab.id.as_str())
            .collect::<Vec<_>>(),
        ["live-bottom-tab"]
    );
    let viewport = geometry
        .viewport_frame
        .as_ref()
        .expect("authored media frame");
    assert_eq!(
        (viewport.x, viewport.y, viewport.width, viewport.height),
        (0.0, 120.0, 800.0, 480.0)
    );
    assert_eq!(geometry.viewport_toolbar_controls.len(), 1);
    let toolbar = &geometry.viewport_toolbar_controls[0];
    assert_eq!(
        toolbar.id,
        "viewport_toolbar_control.document.WorkbenchToolMove"
    );
    assert_eq!((toolbar.frame.x, toolbar.frame.y), (8.0, 90.0));
    assert!(!geometry
        .clickable_frames
        .iter()
        .any(|frame| frame.id.contains("old-")));
    let center = geometry
        .hit_samples
        .iter()
        .find(|sample| sample.id == toolbar.id && sample.sample == "center")
        .unwrap();
    assert!(
        center.route_hit,
        "profile must query the same authored input index as the real window"
    );
    assert!(geometry
        .hit_samples
        .iter()
        .filter(|sample| sample.id == toolbar.id && sample.sample != "center")
        .all(|sample| !sample.route_hit));
}

#[test]
fn unreal_profile_keeps_neighbor_document_frames_and_actual_native_routes() {
    let mut presentation = unreal_profile_presentation();
    let mut neighbor = super::super::data::HostDocumentDockSurfaceData::default();
    neighbor.surface_key = "document:inspector-neighbor".into();
    neighbor.pane.kind = "Inspector".into();
    neighbor.region_frame = FrameRect {
        x: 900.0,
        y: 80.0,
        width: 300.0,
        height: 520.0,
    };
    neighbor.header_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 30.0,
    };
    neighbor.content_frame = FrameRect {
        x: 0.0,
        y: 30.0,
        width: 300.0,
        height: 490.0,
    };
    neighbor.tab_frames = ModelRc::from(Rc::new(VecModel::from(vec![HostChromeTabData {
        control_id: "live-neighbor-tab".into(),
        frame: FrameRect {
            x: 4.0,
            y: 2.0,
            width: 90.0,
            height: 24.0,
        },
        ..Default::default()
    }])));
    neighbor.pane.inspector.nodes =
        ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
            control_id: "NativeNeighborField".into(),
            action_id: "inspector.edit.neighbor".into(),
            component_role: "input-field".into(),
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 40.0,
                width: 100.0,
                height: 24.0,
            },
            ..Default::default()
        }])));
    super::super::surface_hit_test::rebuild_pane_template_hit_artifacts(
        &mut neighbor.pane,
        zircon_runtime_interface::ui::layout::UiSize::new(300.0, 490.0),
    );
    presentation.host_scene_data.document_leaves =
        vec![presentation.host_scene_data.document_dock.clone(), neighbor];
    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(1280, 800),
        1.0,
        HostPresenterBackend::Gpu,
    );
    assert_eq!(geometry.document_tabs.len(), 1);
    assert_eq!(geometry.document_tabs[0].id, "live-neighbor-tab");
    assert_eq!(
        (
            geometry.document_tabs[0].frame.x,
            geometry.document_tabs[0].frame.y
        ),
        (904.0, 82.0)
    );
    let field = geometry
        .template_controls
        .iter()
        .find(|control| control.id.ends_with(".NativeNeighborField"))
        .expect("neighbor native field");
    assert_eq!(field.surface, "document:inspector-neighbor");
    assert_eq!((field.frame.x, field.frame.y), (912.0, 150.0));
    assert!(geometry
        .hit_samples
        .iter()
        .any(|sample| sample.id == field.id && sample.sample == "center" && sample.route_hit));
    assert!(geometry
        .hit_samples
        .iter()
        .any(|sample| sample.id == "live-neighbor-tab"
            && sample.sample == "center"
            && sample.route_hit));
}

#[test]
fn unreal_profile_rejects_authored_toolbar_control_under_wrong_surface_or_kind() {
    let mut presentation = unreal_profile_presentation();
    let route = super::super::profiling_hit_routes::route_contains_profile_frame;
    for surface in ["left", "right", "bottom", "document:not-this-leaf"] {
        let id = format!("viewport_toolbar_control.{surface}.WorkbenchToolMove");
        assert!(
            !route(
                &presentation,
                "viewport_toolbar_control",
                &id,
                surface,
                48.0,
                102.0
            ),
            "{surface}"
        );
    }
    assert!(!route(
        &presentation,
        "template_control",
        "template.document.WorkbenchToolMove",
        "document",
        48.0,
        102.0
    ));
    let mut leaf = presentation.host_scene_data.document_dock.clone();
    leaf.surface_key = "document:scene-leaf".into();
    presentation.host_scene_data.document_leaves = vec![leaf];
    assert!(route(
        &presentation,
        "viewport_toolbar_control",
        "viewport_toolbar_control.document:scene-leaf.WorkbenchToolMove",
        "document:scene-leaf",
        48.0,
        102.0
    ));
    assert!(!route(
        &presentation,
        "viewport_toolbar_control",
        "viewport_toolbar_control.document.WorkbenchToolMove",
        "document",
        48.0,
        102.0
    ));
}

#[test]
fn unreal_profile_retains_native_document_owner_when_authored_media_does_not_fit() {
    let mut presentation = unreal_profile_presentation();
    presentation
        .host_scene_data
        .document_dock
        .region_frame
        .width = 799.0;
    let geometry = UiProfileGeometry::from_presentation(
        &presentation,
        &PhysicalSize::new(1280, 800),
        1.0,
        HostPresenterBackend::Gpu,
    );
    assert_eq!(geometry.document_tabs.len(), 1);
    assert_eq!(geometry.document_tabs[0].id, "old-scene-tab");
    assert_eq!(geometry.viewport_toolbar_controls.len(), 1);
    assert_eq!(
        geometry.viewport_toolbar_controls[0].id,
        "viewport_toolbar_control.document.old-tool"
    );
    assert!(!geometry
        .template_controls
        .iter()
        .any(|control| control.id.ends_with(".WorkbenchToolMove")));
    assert!(geometry.hit_samples.iter().any(|sample| sample.id
        == "viewport_toolbar_control.document.old-tool"
        && sample.sample == "center"
        && sample.route_hit));
}
