pub(super) use crate::core::editor_event::{EditorEvent, LayoutCommand, MenuAction};
pub(super) use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
pub(super) use crate::ui::slint_host::callback_dispatch::{
    dispatch_menu_action, dispatch_shared_menu_pointer_click, BuiltinHostRootShellFrames,
    BuiltinHostWindowTemplateBridge,
};
pub(super) use crate::ui::slint_host::menu_pointer::{
    build_host_menu_pointer_layout, HostMenuPointerBridge, HostMenuPointerLayout,
    HostMenuPointerRoute, HostMenuPointerState,
};
pub(super) use zircon_runtime::ui::{layout::UiFrame, layout::UiPoint, layout::UiSize};

pub(super) fn default_menu_layout() -> HostMenuPointerLayout {
    HostMenuPointerLayout {
        shell_frame: UiFrame::new(0.0, 0.0, 1280.0, 720.0),
        button_frames: [
            UiFrame::new(8.0, 1.0, 40.0, 22.0),
            UiFrame::new(50.0, 1.0, 42.0, 22.0),
            UiFrame::new(94.0, 1.0, 74.0, 22.0),
            UiFrame::new(170.0, 1.0, 42.0, 22.0),
            UiFrame::new(214.0, 1.0, 56.0, 22.0),
            UiFrame::new(272.0, 1.0, 40.0, 22.0),
        ],
        save_project_enabled: true,
        undo_enabled: true,
        redo_enabled: true,
        delete_enabled: true,
        preset_names: vec!["rider".to_string(), "compact".to_string()],
        active_preset_name: "rider".to_string(),
        resolved_preset_name: "rider".to_string(),
        window_popup_height: 132.0,
    }
}

pub(super) fn window_menu_layout(preset_count: usize) -> HostMenuPointerLayout {
    let mut layout = default_menu_layout();
    layout.preset_names = (0..preset_count)
        .map(|index| format!("alpha-{index:02}"))
        .collect();
    layout.window_popup_height = 192.0;
    layout
}
