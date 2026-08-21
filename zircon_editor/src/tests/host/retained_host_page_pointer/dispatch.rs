use crate::core::editor_event::{
    EditorEvent, LayoutCommand, MainPageId, ViewInstanceId as EventViewInstanceId,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_host_page_overflow_pointer_click, dispatch_shared_host_page_pointer_click,
    BuiltinHostWindowTemplateBridge,
};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::host_page_pointer::{
    build_host_page_pointer_layout, HostPagePointerBridge, HostPagePointerRoute,
};
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

use super::support::sample_overflow_host_page_layout;

#[test]
fn shared_host_page_pointer_click_dispatches_activate_main_page_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_host_page_pointer_activate");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut pointer_bridge = HostPagePointerBridge::new();
    let outer_shell_frames = template_bridge.outer_shell_frames();
    pointer_bridge.sync(build_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&outer_shell_frames),
    ));

    let dispatched = dispatch_shared_host_page_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        0,
        8.0,
        92.0,
        UiPoint::new(12.0, 12.0),
    )
    .expect("shared host page route should dispatch activate main page");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostPagePointerRoute::Tab {
            item_index: 0,
            page_id: MainPageId::workbench().0,
        })
    );
    let effects = dispatched
        .effects
        .expect("host page click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::workbench(),
        })
    );
}

#[test]
fn shared_host_page_close_click_dispatches_close_view_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_host_page_pointer_close");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should resolve");
    let opened_instance = manager
        .open_view(ViewDescriptorId::new("editor.asset_browser"), None)
        .expect("asset browser should open as a real exclusive page");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut pointer_bridge = HostPagePointerBridge::new();
    let outer_shell_frames = template_bridge.outer_shell_frames();
    let layout = build_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&outer_shell_frames),
    );
    let tab = layout
        .tabs
        .iter()
        .find(|tab| tab.close_frame.is_some())
        .cloned()
        .expect("real exclusive page should project a closeable host tab");
    let close = tab.close_frame.expect("closeable page close frame");
    pointer_bridge.sync(layout);

    let dispatched = dispatch_shared_host_page_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        tab.page_index,
        tab.frame.x,
        tab.frame.width,
        UiPoint::new(
            close.x + close.width * 0.5 - tab.frame.x,
            close.y + close.height * 0.5 - tab.frame.y,
        ),
    )
    .expect("shared host page close route should dispatch close view");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostPagePointerRoute::Close {
            item_index: tab.page_index,
            instance_id: opened_instance.0.clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("host page close should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::CloseView {
            instance_id: EventViewInstanceId::new(opened_instance.0.clone()),
        })
    );
    assert!(harness
        .runtime
        .current_view_instances()
        .iter()
        .all(|instance| instance.instance_id != opened_instance));
    assert_eq!(
        harness.runtime.current_layout().active_main_page,
        crate::ui::workbench::layout::MainPageId::workbench()
    );
    let reopened_instance = manager
        .open_view(ViewDescriptorId::new("editor.asset_browser"), None)
        .expect("closed single-instance view should reopen through a clean registry");
    assert_ne!(reopened_instance, opened_instance);
}

#[test]
fn shared_host_page_overflow_click_opens_popup_and_hidden_page_selection_activates_page() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_host_page_overflow_activate");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(420.0, 260.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostPagePointerBridge::new();
    pointer_bridge.sync(sample_overflow_host_page_layout());

    let overflow = dispatch_shared_host_page_overflow_pointer_click(
        &mut pointer_bridge,
        UiPoint::new(6.0, 12.0),
    )
    .expect("overflow click should be routed");

    assert_eq!(
        overflow.pointer.route,
        Some(HostPagePointerRoute::Overflow {
            hidden_page_indices: vec![2, 3],
        })
    );
    assert_eq!(
        overflow.effects,
        Some({
            let mut effects = UiHostEventEffects::default();
            effects.request_paint_only();
            effects
        })
    );

    let selected = dispatch_shared_host_page_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        2,
        236.0,
        164.0,
        UiPoint::new(8.0, 12.0),
    )
    .expect("hidden overflow item should activate its page");

    assert_eq!(
        selected.pointer.route,
        Some(HostPagePointerRoute::Tab {
            item_index: 2,
            page_id: "assets".to_string(),
        })
    );
    assert!(selected
        .effects
        .expect("hidden page selection should dispatch into the runtime")
        .dirty_domains()
        .contains(HostInvalidationMask::PRESENTATION_DATA));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("assets"),
        })
    );
}
