use crate::core::editor_event::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorEvent, LayoutCommand,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::retained_host::activity_rail_pointer::{
    build_host_activity_rail_pointer_layout, HostActivityRailPointerBridge,
    HostActivityRailPointerRoute, HostActivityRailPointerSide,
};
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_activity_rail_pointer_click, BuiltinHostWindowTemplateBridge,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_activity_rail_pointer_click_dispatches_left_top_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_activity_rail_pointer_project_toggle");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let mut pointer_bridge = HostActivityRailPointerBridge::new();
    let pointer_layout = build_host_activity_rail_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&template_bridge.root_shell_frames()),
    );
    let left_top_index = pointer_layout
        .left_tabs
        .iter()
        .position(|tab| tab.slot == "left_top")
        .expect("left-top drawer tab should exist in left activity rail");
    let left_top_instance_id = pointer_layout.left_tabs[left_top_index].instance_id.clone();
    pointer_bridge.sync(pointer_layout.clone());

    let dispatched = dispatch_shared_activity_rail_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        HostActivityRailPointerSide::Left,
        UiPoint::new(15.0, 20.0 + left_top_index as f32 * 36.0),
    )
    .expect("shared activity rail route should dispatch left-top drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: left_top_index,
            slot: "left_top".to_string(),
            instance_id: left_top_instance_id,
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
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::Collapsed,
        })
    );
}
