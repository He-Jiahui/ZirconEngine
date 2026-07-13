use crate::core::commands::EditorCommandDescriptor;
pub(super) use crate::core::commands::{MenuBarModel, MenuItemModel, MenuModel};
pub(super) use crate::core::editor_event::{EditorEvent, LayoutCommand, MainPageId, MenuAction};
pub(super) use crate::core::editor_extension::EditorExtensionRegistry;
pub(super) use crate::core::editor_operation::EditorOperationPath;
pub(super) use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
pub(super) use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
pub(super) use crate::ui::retained_host::callback_dispatch::{
    dispatch_menu_action, dispatch_shared_menu_pointer_click, BuiltinHostOuterShellFrames,
    BuiltinHostWindowTemplateBridge,
};
use crate::ui::retained_host::measure_runtime_text_width;
pub(super) use crate::ui::retained_host::menu_pointer::{
    build_host_menu_pointer_layout, HostMenuPointerBridge, HostMenuPointerLayout,
    HostMenuPointerRoute, HostMenuPointerState, MenuItemSpec,
};
use crate::ui::workbench::menu_bar::{
    workbench_menu_slot_width_from_label_width, WORKBENCH_MENU_SLOT_FONT_SIZE,
};
pub(super) use crate::ui::workbench::model::WorkbenchViewModel;
pub(super) use crate::ui::workbench::window_registry::MenuOverflowMode;
pub(super) use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint, UiSize};

const DEFAULT_MENU_LABELS: [&str; 7] = [
    "File",
    "Edit",
    "Selection",
    "Play",
    "View",
    "Window",
    "Help",
];

pub(super) fn default_menu_layout() -> HostMenuPointerLayout {
    let button_frames = expected_menu_button_frames(
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
        &DEFAULT_MENU_LABELS,
        DEFAULT_MENU_LABELS.len(),
    );
    let menu_bar_content_width = menu_bar_content_width(&button_frames, 0.0);

    HostMenuPointerLayout {
        shell_frame: UiFrame::new(0.0, 0.0, 1280.0, 720.0),
        button_frames,
        menu_bar_content_width,
        popup_widths: Vec::new(),
        save_project_enabled: true,
        undo_enabled: true,
        redo_enabled: true,
        delete_enabled: true,
        preset_names: vec!["rider".to_string(), "compact".to_string()],
        active_preset_name: "rider".to_string(),
        resolved_preset_name: "rider".to_string(),
        window_popup_height: 132.0,
        menu_overflow_mode: MenuOverflowMode::Auto,
        menus: Vec::new(),
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

pub(super) fn expected_menu_button_frames_for_model(
    frame: UiFrame,
    menu_bar: &MenuBarModel,
) -> Vec<UiFrame> {
    let labels = menu_bar
        .menus
        .iter()
        .map(|menu| menu.label.as_str())
        .collect::<Vec<_>>();
    expected_menu_button_frames(frame, &labels, labels.len().max(DEFAULT_MENU_LABELS.len()))
}

pub(super) fn expected_menu_button_frames(
    frame: UiFrame,
    labels: &[&str],
    menu_count: usize,
) -> Vec<UiFrame> {
    let mut x = frame.x + 8.0;
    let y = frame.y + 2.0;
    (0..menu_count)
        .map(|index| {
            let width = runtime_menu_slot_width(labels.get(index).copied().unwrap_or_default());
            let frame = UiFrame::new(x, y, width, 22.0);
            x += width + 2.0;
            frame
        })
        .collect()
}

pub(super) fn runtime_menu_slot_width(label: &str) -> f32 {
    workbench_menu_slot_width_from_label_width(measure_runtime_text_width(
        label,
        WORKBENCH_MENU_SLOT_FONT_SIZE,
    ))
}

fn menu_bar_content_width(frames: &[UiFrame], viewport_x: f32) -> f32 {
    frames
        .iter()
        .map(|frame| frame.x + frame.width - viewport_x)
        .fold(0.0, f32::max)
}
