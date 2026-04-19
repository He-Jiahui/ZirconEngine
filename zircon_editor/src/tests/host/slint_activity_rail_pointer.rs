use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::activity_rail_pointer::{
    build_workbench_activity_rail_pointer_layout, WorkbenchActivityRailPointerBridge,
    WorkbenchActivityRailPointerItem, WorkbenchActivityRailPointerLayout,
    WorkbenchActivityRailPointerRoute, WorkbenchActivityRailPointerSide,
};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_activity_rail_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::{
    compute_workbench_shell_geometry, EditorEvent, LayoutCommand, ShellSizePx,
    WorkbenchChromeMetrics, WorkbenchViewModel,
};
use zircon_runtime::ui::{layout::UiFrame, layout::UiPoint, layout::UiSize};

#[test]
fn shared_activity_rail_pointer_bridge_routes_left_and_right_button_hits() {
    let mut bridge = WorkbenchActivityRailPointerBridge::new();
    bridge.sync(sample_activity_rail_layout());

    let left = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Left,
            UiPoint::new(15.0, 20.0),
        )
        .unwrap();
    assert_eq!(
        left.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );

    let right = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Right,
            UiPoint::new(15.0, 52.0),
        )
        .unwrap();
    assert_eq!(
        right.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Right,
            item_index: 1,
            slot: "right_bottom".to_string(),
            instance_id: "editor.console#1".to_string(),
        })
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_accepts_projected_global_points() {
    let mut bridge = WorkbenchActivityRailPointerBridge::new();
    let layout = sample_activity_rail_layout();
    bridge.sync(layout.clone());

    let left = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Left,
            UiPoint::new(
                layout.left_strip_frame.x + 15.0,
                layout.left_strip_frame.y + 20.0,
            ),
        )
        .unwrap();
    assert_eq!(
        left.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
}

#[test]
fn shared_activity_rail_pointer_click_dispatches_project_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_activity_rail_pointer_project_toggle");
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
    let mut pointer_bridge = WorkbenchActivityRailPointerBridge::new();
    pointer_bridge.sync(build_workbench_activity_rail_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        Some(&template_bridge.root_shell_frames()),
    ));

    let dispatched = dispatch_shared_activity_rail_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        WorkbenchActivityRailPointerSide::Left,
        UiPoint::new(15.0, 20.0),
    )
    .expect("shared activity rail route should dispatch project drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("activity rail click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::SetDrawerMode {
            slot: crate::ActivityDrawerSlot::LeftTop,
            mode: crate::ActivityDrawerMode::Collapsed,
        })
    );
}

#[test]
fn shared_activity_rail_pointer_layout_prefers_shared_root_projection_when_left_region_geometry_is_stale(
) {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_activity_rail_pointer_root_projection");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let mut geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &harness.runtime.current_layout(),
        &harness.runtime.descriptors(),
        ShellSizePx::new(1280.0, 720.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    geometry
        .region_frames
        .insert(crate::ShellRegionId::Left, crate::ShellFrame::default());
    geometry
        .region_frames
        .insert(crate::ShellRegionId::Right, crate::ShellFrame::default());
    geometry
        .region_frames
        .insert(crate::ShellRegionId::Bottom, crate::ShellFrame::default());

    let root_frames = template_bridge.root_shell_frames();
    let layout = build_workbench_activity_rail_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
    );

    assert_eq!(
        layout.left_strip_frame,
        root_frames.activity_rail_frame.unwrap()
    );
    assert_eq!(layout.right_strip_frame, UiFrame::default());
}

#[test]
fn shared_activity_rail_pointer_layout_prefers_shared_visible_drawer_regions_when_cross_axis_geometry_is_stale(
) {
    let fixture = crate::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let root_frames = template_bridge.root_shell_frames();
    let body_frame = root_frames
        .workbench_body_frame
        .expect("workbench body projection frame should exist");
    let left_geometry = crate::ShellFrame::new(180.0, 140.0, 312.0, 519.0);
    let right_geometry = crate::ShellFrame::new(1024.0, 168.0, 256.0, 401.0);
    let bottom_geometry = crate::ShellFrame::new(48.0, 704.0, 777.0, 180.0);
    let expected_strip_height =
        body_frame.height - bottom_geometry.height - metrics.separator_thickness;
    let geometry = crate::WorkbenchShellGeometry {
        region_frames: [
            (crate::ShellRegionId::Left, left_geometry),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(493.0, 140.0, 531.0, 440.0),
            ),
            (crate::ShellRegionId::Right, right_geometry),
            (crate::ShellRegionId::Bottom, bottom_geometry),
        ]
        .into_iter()
        .collect(),
        ..crate::WorkbenchShellGeometry::default()
    };

    let layout = build_workbench_activity_rail_pointer_layout(
        &model,
        &geometry,
        &metrics,
        Some(&root_frames),
    );

    assert_eq!(
        layout.left_strip_frame,
        UiFrame::new(0.0, body_frame.y, metrics.rail_width, expected_strip_height)
    );
    assert_eq!(
        layout.right_strip_frame,
        UiFrame::new(
            body_frame.x + body_frame.width - metrics.rail_width,
            body_frame.y,
            metrics.rail_width,
            expected_strip_height,
        )
    );
}

#[test]
fn shared_activity_rail_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    let direct_toggle_count = workbench
        .matches("clicked => { root.toggle_drawer_tab(tab.slot, tab.id); }")
        .count();
    assert!(
        direct_toggle_count <= 3,
        "workbench shell still exposes legacy rail direct callback sites ({direct_toggle_count})"
    );

    for needle in [
        "callback activity_rail_pointer_clicked(",
        "activity_rail_pointer_clicked(",
    ] {
        assert!(
            workbench.contains(needle) || host_context.contains(needle),
            "workbench shell is missing shared activity rail pointer hook `{needle}`"
        );
    }

    assert!(
        app.contains("host_shell.on_activity_rail_pointer_clicked(")
            || wiring.contains("host_shell.on_activity_rail_pointer_clicked("),
        "slint host app must register shared activity rail pointer clicks"
    );
}

fn sample_activity_rail_layout() -> WorkbenchActivityRailPointerLayout {
    WorkbenchActivityRailPointerLayout {
        left_strip_frame: UiFrame::new(0.0, 51.0, 34.0, 400.0),
        left_tabs: vec![
            WorkbenchActivityRailPointerItem {
                slot: "left_top".to_string(),
                instance_id: "editor.project#1".to_string(),
            },
            WorkbenchActivityRailPointerItem {
                slot: "left_bottom".to_string(),
                instance_id: "editor.hierarchy#1".to_string(),
            },
        ],
        right_strip_frame: UiFrame::new(1246.0, 51.0, 34.0, 400.0),
        right_tabs: vec![
            WorkbenchActivityRailPointerItem {
                slot: "right_top".to_string(),
                instance_id: "editor.inspector#1".to_string(),
            },
            WorkbenchActivityRailPointerItem {
                slot: "right_bottom".to_string(),
                instance_id: "editor.console#1".to_string(),
            },
        ],
    }
}
