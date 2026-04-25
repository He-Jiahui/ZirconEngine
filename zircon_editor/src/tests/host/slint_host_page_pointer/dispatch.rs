use crate::core::editor_event::{EditorEvent, LayoutCommand, MainPageId};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_host_page_pointer_click, BuiltinHostWindowTemplateBridge,
};
use crate::ui::slint_host::host_page_pointer::{
    build_host_page_pointer_layout, HostPagePointerBridge, HostPagePointerRoute,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_host_page_pointer_click_dispatches_activate_main_page_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_activate");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let mut pointer_bridge = HostPagePointerBridge::new();
    let root_frames = template_bridge.root_shell_frames();
    pointer_bridge.sync(build_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
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
