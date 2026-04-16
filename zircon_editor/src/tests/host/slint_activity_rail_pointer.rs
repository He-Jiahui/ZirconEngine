use crate::host::slint_host::activity_rail_pointer::{
    build_workbench_activity_rail_pointer_layout, WorkbenchActivityRailPointerBridge,
    WorkbenchActivityRailPointerItem, WorkbenchActivityRailPointerLayout,
    WorkbenchActivityRailPointerRoute, WorkbenchActivityRailPointerSide,
};
use crate::host::slint_host::callback_dispatch::{
    dispatch_shared_activity_rail_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{
    compute_workbench_shell_geometry, EditorEvent, LayoutCommand, ShellSizePx,
    WorkbenchChromeMetrics, WorkbenchViewModel,
};
use zircon_ui::{UiFrame, UiPoint, UiSize};

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
fn shared_activity_rail_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
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
            workbench.contains(needle),
            "workbench shell is missing shared activity rail pointer hook `{needle}`"
        );
    }

    assert!(
        app.contains("ui.on_activity_rail_pointer_clicked(")
            || wiring.contains("ui.on_activity_rail_pointer_clicked("),
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
