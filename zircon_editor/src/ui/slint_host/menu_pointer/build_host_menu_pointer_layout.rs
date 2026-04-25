use zircon_runtime::ui::{layout::UiFrame, layout::UiSize};

use crate::ui::slint_host::app::compute_window_menu_popup_height;
use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::constants::{
    MENU_BUTTON_HEIGHT, MENU_BUTTON_ROW_GAP, MENU_BUTTON_ROW_X, MENU_BUTTON_ROW_Y,
    MENU_BUTTON_WIDTHS, WINDOW_MENU_INDEX,
};
use super::HostMenuPointerLayout;

pub(crate) fn build_host_menu_pointer_layout(
    chrome: &EditorChromeSnapshot,
    shell_size: UiSize,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> HostMenuPointerLayout {
    let shell_frame = shared_root_frames
        .and_then(|frames| frames.shell_frame)
        .unwrap_or_else(|| UiFrame::new(0.0, 0.0, shell_size.width, shell_size.height));
    let button_frames = shared_root_frames
        .and_then(|frames| frames.menu_bar_frame)
        .map(menu_button_frames_from_frame)
        .unwrap_or_else(|| menu_button_frames_from_frame(shell_frame));
    let active_preset_name = active_layout_preset.unwrap_or_default().to_string();
    let resolved_preset_name = if active_preset_name.is_empty() {
        "rider".to_string()
    } else {
        active_preset_name.clone()
    };
    let window_popup_height = compute_window_menu_popup_height(
        shell_frame.height,
        button_frames[WINDOW_MENU_INDEX],
        preset_names.len(),
    );

    HostMenuPointerLayout {
        shell_frame,
        button_frames,
        save_project_enabled: chrome.project_open,
        undo_enabled: chrome.can_undo,
        redo_enabled: chrome.can_redo,
        delete_enabled: chrome.inspector.is_some(),
        preset_names: preset_names.to_vec(),
        active_preset_name,
        resolved_preset_name,
        window_popup_height,
    }
}

fn menu_button_frames_from_frame(frame: UiFrame) -> [UiFrame; 6] {
    let mut next_x = frame.x + MENU_BUTTON_ROW_X;
    MENU_BUTTON_WIDTHS.map(|width| {
        let frame = UiFrame::new(
            next_x,
            frame.y + MENU_BUTTON_ROW_Y,
            width,
            MENU_BUTTON_HEIGHT,
        );
        next_x += width + MENU_BUTTON_ROW_GAP;
        frame
    })
}
