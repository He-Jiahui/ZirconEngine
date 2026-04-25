use crate::core::editor_event::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorEvent, LayoutCommand,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_drawer_header_pointer_click, BuiltinHostWindowTemplateBridge,
};
use crate::ui::slint_host::drawer_header_pointer::{
    build_host_drawer_header_pointer_layout, HostDrawerHeaderPointerBridge,
    HostDrawerHeaderPointerRoute,
};
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_drawer_header_pointer_click_dispatches_drawer_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_drawer_header_pointer_toggle");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
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
    let mut pointer_bridge = HostDrawerHeaderPointerBridge::new();
    pointer_bridge.sync(build_host_drawer_header_pointer_layout(
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
        Some(HostDrawerHeaderPointerRoute::Tab {
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
