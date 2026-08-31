use crate::core::editor_event::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorEvent, LayoutCommand, ViewInstanceId,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::retained_host::callback_dispatch::dispatch_shared_drawer_header_pointer_click;
use crate::ui::retained_host::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout, HostDrawerHeaderPointerBridge,
    HostDrawerHeaderPointerRoute,
};
use crate::ui::workbench::layout::ActivityDrawerSlot as UiActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn shared_drawer_header_pointer_click_dispatches_drawer_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_drawer_header_pointer_toggle");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let left_top_key = "left";
    let mut pointer_bridge = HostDrawerHeaderPointerBridge::new();
    let pointer_layout = build_host_drawer_header_pointer_layout(&model);
    let left_top = pointer_layout
        .surfaces
        .iter()
        .find(|surface| surface.key == left_top_key)
        .and_then(|surface| {
            surface
                .items
                .iter()
                .enumerate()
                .find(|(_, item)| item.slot == UiActivityDrawerSlot::LeftTop)
        })
        .map(|(index, item)| (index, item.instance_id.clone()))
        .expect("left top drawer header item should be projected");
    pointer_bridge.sync(pointer_layout);

    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &pointer_bridge,
        left_top_key,
        left_top.0,
    )
    .expect("shared drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_index: 0,
            item_index: left_top.0,
        })
    );
    assert_eq!(
        pointer_bridge
            .target_for_route(dispatched.pointer.route.expect("drawer route"))
            .map(|(slot, instance_id)| (slot, instance_id.0.as_str())),
        Some((UiActivityDrawerSlot::LeftTop, left_top.1 .0.as_str()))
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
fn shared_bottom_drawer_header_pointer_click_activates_runtime_diagnostics_tab() {
    let _guard = env_lock().lock().unwrap();

    let harness =
        EventRuntimeHarness::new("zircon_retained_drawer_header_pointer_bottom_runtime_diag");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let bottom_key = "bottom";
    let mut pointer_bridge = HostDrawerHeaderPointerBridge::new();
    let pointer_layout = build_host_drawer_header_pointer_layout(&model);
    let runtime_diagnostics = pointer_layout
        .surfaces
        .iter()
        .find(|surface| surface.key == bottom_key)
        .and_then(|surface| {
            surface
                .items
                .iter()
                .enumerate()
                .find(|(_, item)| item.instance_id == "editor.runtime_diagnostics#1")
        })
        .map(|(index, item)| (index, item.instance_id.clone()))
        .expect("runtime diagnostics bottom drawer header item should be projected");
    let bottom_surface_index = pointer_layout
        .surfaces
        .iter()
        .position(|surface| surface.key == bottom_key)
        .expect("bottom receipt surface index");
    pointer_bridge.sync(pointer_layout);

    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &pointer_bridge,
        bottom_key,
        runtime_diagnostics.0,
    )
    .expect("shared bottom drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_index: bottom_surface_index,
            item_index: runtime_diagnostics.0,
        })
    );
    let effects = dispatched
        .effects
        .expect("bottom drawer header click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::ActivateDrawerTab {
            slot: ActivityDrawerSlot::Bottom,
            instance_id: ViewInstanceId::new("editor.runtime_diagnostics#1"),
        })
    );
}
