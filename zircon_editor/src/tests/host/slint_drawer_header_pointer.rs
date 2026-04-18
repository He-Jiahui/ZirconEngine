use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_drawer_header_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::ui::slint_host::drawer_header_pointer::{
    build_workbench_drawer_header_pointer_layout, WorkbenchDrawerHeaderPointerBridge,
    WorkbenchDrawerHeaderPointerItem, WorkbenchDrawerHeaderPointerLayout,
    WorkbenchDrawerHeaderPointerRoute, WorkbenchDrawerHeaderPointerSurface,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{
    compute_workbench_shell_geometry, ActivityDrawerMode, ActivityDrawerSlot, EditorEvent,
    LayoutCommand, ShellSizePx, WorkbenchChromeMetrics, WorkbenchViewModel,
};
use zircon_ui::{UiFrame, UiPoint, UiSize};

#[test]
fn shared_drawer_header_pointer_bridge_routes_group_tabs_from_shared_hit_test() {
    let mut bridge = WorkbenchDrawerHeaderPointerBridge::new();
    bridge.sync(sample_drawer_header_layout());

    let route = bridge
        .handle_click("left", 1, 112.0, 96.0, UiPoint::new(120.0, 12.0))
        .unwrap();
    assert_eq!(
        route.route,
        Some(WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key: "left".to_string(),
            item_index: 1,
            slot: "left_bottom".to_string(),
            instance_id: "editor.hierarchy#1".to_string(),
        })
    );
}

#[test]
fn shared_drawer_header_pointer_click_dispatches_drawer_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_drawer_header_pointer_toggle");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &harness.runtime.current_layout(),
        &harness.runtime.descriptors(),
        ShellSizePx::new(1280.0, 720.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    let mut pointer_bridge = WorkbenchDrawerHeaderPointerBridge::new();
    pointer_bridge.sync(build_workbench_drawer_header_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        Some(&template_bridge.root_shell_frames()),
    ));

    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "left",
        0,
        6.0,
        96.0,
        UiPoint::new(24.0, 12.0),
    )
    .expect("shared drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key: "left".to_string(),
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("drawer header click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::Collapsed,
        })
    );
}

#[test]
fn shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions() {
    let fixture = crate::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("builtin workbench template bridge should recompute visible drawer frames");
    let root_frames = bridge.root_shell_frames();
    let left_geometry = UiFrame::new(180.0, 140.0, 180.0, 519.0);
    let bottom_geometry = UiFrame::new(52.0, 704.0, 777.0, 120.0);
    let geometry = crate::WorkbenchShellGeometry {
        region_frames: [
            (
                crate::ShellRegionId::Left,
                crate::ShellFrame::new(
                    left_geometry.x,
                    left_geometry.y,
                    left_geometry.width,
                    left_geometry.height,
                ),
            ),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(493.0, 140.0, 531.0, 440.0),
            ),
            (
                crate::ShellRegionId::Right,
                crate::ShellFrame::new(1025.0, 140.0, 255.0, 440.0),
            ),
            (
                crate::ShellRegionId::Bottom,
                crate::ShellFrame::new(
                    bottom_geometry.x,
                    bottom_geometry.y,
                    bottom_geometry.width,
                    bottom_geometry.height,
                ),
            ),
        ]
        .into_iter()
        .collect(),
        ..crate::WorkbenchShellGeometry::default()
    };

    let layout = build_workbench_drawer_header_pointer_layout(
        &model,
        &geometry,
        &metrics,
        Some(&root_frames),
    );

    let left_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == "left")
        .expect("left drawer surface should exist");
    assert_eq!(
        left_surface.strip_frame,
        root_frames
            .left_drawer_header_frame
            .expect("shared left drawer header frame should exist")
    );

    let bottom_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == "bottom")
        .expect("bottom drawer surface should exist");
    assert_eq!(
        bottom_surface.strip_frame,
        root_frames
            .bottom_drawer_header_frame
            .expect("shared bottom drawer header frame should exist")
    );
}

#[test]
fn shared_drawer_header_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    assert!(
        !workbench.contains("clicked => { root.toggle_drawer_tab(tab.slot, tab.id); }"),
        "workbench shell still exposes legacy direct drawer header callback"
    );
    for needle in [
        "callback drawer_header_pointer_clicked(",
        "pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle) || chrome.contains(needle),
            "drawer header shared pointer hook `{needle}` is missing"
        );
    }
    assert!(
        !app.contains("ui.on_toggle_drawer_tab("),
        "slint host app should no longer register direct drawer toggle callback"
    );
    assert!(
        app.contains("ui.on_drawer_header_pointer_clicked(")
            || wiring.contains("ui.on_drawer_header_pointer_clicked("),
        "slint host app must register shared drawer header callback"
    );
}

fn sample_drawer_header_layout() -> WorkbenchDrawerHeaderPointerLayout {
    WorkbenchDrawerHeaderPointerLayout {
        surfaces: vec![
            WorkbenchDrawerHeaderPointerSurface {
                key: "left".to_string(),
                strip_frame: UiFrame::new(35.0, 53.0, 240.0, 25.0),
                items: vec![
                    WorkbenchDrawerHeaderPointerItem {
                        slot: "left_top".to_string(),
                        instance_id: "editor.project#1".to_string(),
                    },
                    WorkbenchDrawerHeaderPointerItem {
                        slot: "left_bottom".to_string(),
                        instance_id: "editor.hierarchy#1".to_string(),
                    },
                ],
            },
            WorkbenchDrawerHeaderPointerSurface {
                key: "right".to_string(),
                strip_frame: UiFrame::new(1002.0, 53.0, 240.0, 25.0),
                items: vec![WorkbenchDrawerHeaderPointerItem {
                    slot: "right_top".to_string(),
                    instance_id: "editor.inspector#1".to_string(),
                }],
            },
        ],
    }
}
