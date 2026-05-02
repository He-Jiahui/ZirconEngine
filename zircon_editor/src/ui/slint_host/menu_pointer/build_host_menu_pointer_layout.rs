use std::collections::BTreeMap;

use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::layouts::views::build_view_template_nodes;
use crate::ui::slint_host::app::compute_window_menu_popup_height;
use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::workbench::model::{MenuBarModel, MenuItemModel, MenuModel};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::constants::WINDOW_MENU_INDEX;
use super::menu_item_spec::MenuItemSpec;
use super::HostMenuPointerLayout;

const MENU_CHROME_ASSET: &str = "/assets/ui/editor/workbench_menu_chrome.ui.toml";
const MENU_SLOT_PREFIX: &str = "MenuSlot";
const MENU_BUTTON_COUNT: usize = 6;

pub(crate) fn build_host_menu_pointer_layout(
    menu_bar: &MenuBarModel,
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
        .map(menu_button_frames_from_chrome_asset)
        .unwrap_or_else(|| menu_button_frames_from_chrome_asset(shell_frame));
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
        menus: pointer_menus(menu_bar, preset_names, active_layout_preset),
    }
}

fn pointer_menus(
    menu_bar: &MenuBarModel,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
) -> Vec<Vec<MenuItemSpec>> {
    menu_bar
        .menus
        .iter()
        .map(|menu| pointer_menu_items(menu, preset_names, active_layout_preset))
        .collect()
}

fn pointer_menu_items(
    menu: &MenuModel,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
) -> Vec<MenuItemSpec> {
    if menu.label.eq_ignore_ascii_case("Window") {
        let resolved_preset_name = active_layout_preset.unwrap_or("rider");
        let mut items = vec![menu_action(
            format!("SavePreset.{resolved_preset_name}"),
            true,
        )];
        items.extend(menu.items.iter().map(pointer_menu_item));
        items.extend(
            preset_names
                .iter()
                .map(|preset| menu_action(format!("LoadPreset.{preset}"), true)),
        );
        items
    } else {
        menu.items.iter().map(pointer_menu_item).collect()
    }
}

fn pointer_menu_item(item: &MenuItemModel) -> MenuItemSpec {
    MenuItemSpec {
        action_id: item.enabled.then(|| menu_item_action_id(item)).flatten(),
        enabled: item.enabled,
    }
}

fn menu_item_action_id(item: &MenuItemModel) -> Option<String> {
    match item.binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => Some(action_id.clone()),
        _ => None,
    }
}

fn menu_action(action_id: impl Into<String>, enabled: bool) -> MenuItemSpec {
    MenuItemSpec {
        action_id: enabled.then(|| action_id.into()),
        enabled,
    }
}

fn menu_button_frames_from_chrome_asset(frame: UiFrame) -> [UiFrame; MENU_BUTTON_COUNT] {
    let nodes = build_view_template_nodes(
        "host.menu.pointer.chrome",
        MENU_CHROME_ASSET,
        &[],
        UiSize::new(frame.width.max(0.0), frame.height.max(0.0)),
        &BTreeMap::new(),
    )
    .unwrap_or_default();

    let mut frames = [UiFrame::default(); MENU_BUTTON_COUNT];
    for node in nodes {
        let Some(index) = node
            .control_id
            .as_str()
            .strip_prefix(MENU_SLOT_PREFIX)
            .and_then(|suffix| suffix.parse::<usize>().ok())
        else {
            continue;
        };
        if index < MENU_BUTTON_COUNT {
            frames[index] = UiFrame::new(
                frame.x + node.frame.x,
                frame.y + node.frame.y,
                node.frame.width,
                node.frame.height,
            );
        }
    }
    frames
}
