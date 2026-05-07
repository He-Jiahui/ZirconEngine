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
const MENU_BUTTON_COUNT: usize = 7;

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
    let menu_count = menu_bar.menus.len().max(MENU_BUTTON_COUNT);
    let menu_labels = menu_bar
        .menus
        .iter()
        .map(|menu| menu.label.as_str())
        .collect::<Vec<_>>();
    let button_frames = shared_root_frames
        .and_then(|frames| frames.menu_bar_frame)
        .map(|frame| menu_button_frames_from_chrome_asset(frame, &menu_labels, menu_count))
        .unwrap_or_else(|| {
            menu_button_frames_from_chrome_asset(shell_frame, &menu_labels, menu_count)
        });
    let menu_bar_content_width = menu_bar_content_width(&button_frames, shell_frame.x);
    let active_preset_name = active_layout_preset.unwrap_or_default().to_string();
    let resolved_preset_name = if active_preset_name.is_empty() {
        "rider".to_string()
    } else {
        active_preset_name.clone()
    };
    let window_popup_height = compute_window_menu_popup_height(
        shell_frame.height,
        button_frames
            .get(WINDOW_MENU_INDEX)
            .copied()
            .unwrap_or(shell_frame),
        preset_names.len(),
    );

    HostMenuPointerLayout {
        shell_frame,
        button_frames,
        menu_bar_content_width,
        save_project_enabled: chrome.project_open,
        undo_enabled: chrome.can_undo,
        redo_enabled: chrome.can_redo,
        delete_enabled: chrome.inspector.is_some(),
        preset_names: preset_names.to_vec(),
        active_preset_name,
        resolved_preset_name,
        window_popup_height,
        menu_overflow_mode: chrome.menu_overflow_mode,
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
        items.extend(pointer_menu_item_tree(&menu.items));
        items.extend(
            preset_names
                .iter()
                .map(|preset| menu_action(format!("LoadPreset.{preset}"), true)),
        );
        items
    } else {
        pointer_menu_item_tree(&menu.items)
    }
}

fn pointer_menu_item(item: &MenuItemModel) -> MenuItemSpec {
    let children = pointer_menu_item_tree(&item.children);
    MenuItemSpec {
        action_id: if children.is_empty() {
            item.enabled.then(|| menu_item_action_id(item)).flatten()
        } else {
            None
        },
        enabled: item.enabled,
        children,
    }
}

fn pointer_menu_item_tree(items: &[MenuItemModel]) -> Vec<MenuItemSpec> {
    items.iter().map(pointer_menu_item).collect()
}

fn menu_item_action_id(item: &MenuItemModel) -> Option<String> {
    match item.binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => Some(action_id.clone()),
        EditorUiBindingPayload::EditorOperation { operation_id, .. } => Some(operation_id.clone()),
        _ => None,
    }
}

fn menu_action(action_id: impl Into<String>, enabled: bool) -> MenuItemSpec {
    MenuItemSpec {
        action_id: enabled.then(|| action_id.into()),
        enabled,
        children: Vec::new(),
    }
}

fn menu_button_frames_from_chrome_asset(
    frame: UiFrame,
    menu_labels: &[&str],
    menu_count: usize,
) -> Vec<UiFrame> {
    let nodes = build_view_template_nodes(
        "host.menu.pointer.chrome",
        MENU_CHROME_ASSET,
        &[],
        UiSize::new(frame.width.max(0.0), frame.height.max(0.0)),
        &BTreeMap::new(),
    )
    .unwrap_or_default();

    let mut stencil_frames = [UiFrame::default(); MENU_BUTTON_COUNT];
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
            stencil_frames[index] = UiFrame::new(
                frame.x + node.frame.x,
                frame.y + node.frame.y,
                node.frame.width,
                node.frame.height,
            );
        }
    }

    let mut frames = if stencil_frames.iter().all(|slot| slot.width > 0.0) {
        stencil_frames.to_vec()
    } else {
        fallback_menu_button_frames(frame, menu_labels, MENU_BUTTON_COUNT)
    };
    let gap = menu_button_gap(&frames).unwrap_or(2.0);
    while frames.len() < menu_count {
        let previous = frames
            .last()
            .copied()
            .unwrap_or_else(|| UiFrame::new(frame.x + 8.0, frame.y + 2.0, 40.0, 22.0));
        let label = menu_labels.get(frames.len()).copied().unwrap_or_default();
        frames.push(UiFrame::new(
            previous.x + previous.width + gap,
            previous.y,
            menu_label_slot_width(label),
            previous.height,
        ));
    }
    frames.truncate(menu_count);
    frames
}

fn fallback_menu_button_frames(
    frame: UiFrame,
    menu_labels: &[&str],
    menu_count: usize,
) -> Vec<UiFrame> {
    let mut x = frame.x + 8.0;
    (0..menu_count)
        .map(|index| {
            let label = menu_labels.get(index).copied().unwrap_or_default();
            let width = menu_label_slot_width(label);
            let slot = UiFrame::new(x, frame.y + 2.0, width, 22.0);
            x += width + 4.0;
            slot
        })
        .collect()
}

fn menu_label_slot_width(label: &str) -> f32 {
    ((label.chars().count() as f32 * 7.0) + 24.0).clamp(40.0, 128.0)
}

fn menu_button_gap(frames: &[UiFrame]) -> Option<f32> {
    frames
        .windows(2)
        .rev()
        .filter_map(|pair| {
            let gap = pair[1].x - (pair[0].x + pair[0].width);
            (gap > 0.0).then_some(gap)
        })
        .next()
}

fn menu_bar_content_width(frames: &[UiFrame], viewport_x: f32) -> f32 {
    frames
        .iter()
        .map(|frame| frame.x + frame.width - viewport_x)
        .fold(0.0, f32::max)
}
