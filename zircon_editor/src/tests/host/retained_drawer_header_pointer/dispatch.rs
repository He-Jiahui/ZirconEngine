use crate::core::editor_event::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorEvent, LayoutCommand, ViewInstanceId,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_drawer_header_pointer_click, BuiltinHostWindowTemplateBridge,
};
use crate::ui::retained_host::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout, HostDrawerHeaderPointerBridge,
    HostDrawerHeaderPointerRoute,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_drawer_header_pointer_click_dispatches_drawer_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_drawer_header_pointer_toggle");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let left_top_key = "left";
    let mut template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    template_bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("builtin workbench template bridge should project drawer headers");
    let mut pointer_bridge = HostDrawerHeaderPointerBridge::new();
    let pointer_layout = build_host_drawer_header_pointer_layout(
        &model,
        &metrics,
        Some(&template_bridge.root_shell_frames()),
    );
    let left_top = pointer_layout
        .surfaces
        .iter()
        .find(|surface| surface.key == left_top_key)
        .and_then(|surface| {
            surface
                .items
                .iter()
                .enumerate()
                .find(|(_, item)| item.slot == "left_top")
        })
        .map(|(index, item)| (index, item.instance_id.clone()))
        .expect("left top drawer header item should be projected");
    pointer_bridge.sync(pointer_layout);

    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        left_top_key,
        left_top.0,
        6.0,
        96.0,
        UiPoint::new(24.0 + left_top.0 as f32 * 112.0, 12.0),
    )
    .expect("shared drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_key: left_top_key.to_string(),
            item_index: left_top.0,
            slot: "left_top".to_string(),
            instance_id: left_top.1,
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
fn shared_bottom_drawer_header_pointer_click_activates_runtime_diagnostics_tab() {
    let _guard = env_lock().lock().unwrap();

    let harness =
        EventRuntimeHarness::new("zircon_retained_drawer_header_pointer_bottom_runtime_diag");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let bottom_key = "bottom";
    let mut template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    template_bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("builtin workbench template bridge should project drawer headers");
    let mut pointer_bridge = HostDrawerHeaderPointerBridge::new();
    let pointer_layout = build_host_drawer_header_pointer_layout(
        &model,
        &metrics,
        Some(&template_bridge.root_shell_frames()),
    );
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
    pointer_bridge.sync(pointer_layout);

    let tab_x = 112.0 * runtime_diagnostics.0 as f32;
    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        bottom_key,
        runtime_diagnostics.0,
        tab_x,
        160.0,
        UiPoint::new(tab_x + 12.0, 12.0),
    )
    .expect("shared bottom drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_key: bottom_key.to_string(),
            item_index: runtime_diagnostics.0,
            slot: "bottom".to_string(),
            instance_id: runtime_diagnostics.1.clone(),
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
