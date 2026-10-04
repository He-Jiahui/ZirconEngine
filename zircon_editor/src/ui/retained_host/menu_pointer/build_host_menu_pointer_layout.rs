use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use crate::core::commands::{MenuBarModel, MenuItemModel, MenuModel};
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::layouts::views::{
    build_view_template_node_projection, view_template_resource_generation,
    ViewTemplateResourceGeneration,
};
use crate::ui::retained_host::app::compute_window_menu_popup_height;
use crate::ui::retained_host::callback_dispatch::BuiltinHostOuterShellFrames;
use crate::ui::retained_host::menu_popup_contract::content_measured_menu_popup_width;
use crate::ui::retained_host::{
    current_host_metrics, measure_runtime_text_width, menu_popup_text_width,
};
use crate::ui::workbench::event::menu_item_binding;
use crate::ui::workbench::menu_bar::workbench_menu_slot_width_for_paint;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::constants::WINDOW_MENU_INDEX;
use super::menu_item_spec::MenuItemSpec;
use super::HostMenuPointerLayout;

const MENU_CHROME_ASSET: &str = "/assets/ui/editor/workbench_menu_chrome.zui";
const MENU_SLOT_PREFIX: &str = "MenuSlot";
const MENU_BUTTON_COUNT: usize = 7;
const MENU_STENCIL_REFERENCE_WIDTH: f32 = 1280.0;
const MENU_STENCIL_REFERENCE_HEIGHT: f32 = 24.0;

struct MenuPointerChromeStencilCache {
    generation: ViewTemplateResourceGeneration,
    frames: [UiFrame; MENU_BUTTON_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MenuLabelSlotMetrics {
    /// Physical pixels used for text measurement and slot geometry.
    pub(crate) font_size: f32,
    /// Logical pixels copied into authored scene nodes and scaled at paint time.
    pub(crate) logical_font_size: f32,
    pub(crate) horizontal_inset: f32,
}

pub(crate) fn current_menu_label_slot_metrics() -> MenuLabelSlotMetrics {
    let metrics = current_host_metrics();
    let scale_factor = if metrics.scale_factor.is_finite() && metrics.scale_factor > 0.0 {
        metrics.scale_factor
    } else {
        1.0
    };
    MenuLabelSlotMetrics {
        font_size: metrics.font_body,
        logical_font_size: metrics.font_body / scale_factor,
        horizontal_inset: (metrics.gap_m - metrics.border_width * 2.0).max(0.0),
    }
}

pub(crate) fn menu_label_slot_width(label: &str, metrics: MenuLabelSlotMetrics) -> f32 {
    let label_width = measure_runtime_text_width(label, metrics.font_size);
    menu_label_slot_width_from_measured(label, metrics, label_width)
}

fn menu_label_slot_width_from_measured(
    label: &str,
    metrics: MenuLabelSlotMetrics,
    label_width: f32,
) -> f32 {
    let label_width = if label_width.is_finite() && label_width > 0.0 {
        label_width
    } else {
        // Font admission can defer intrinsic measurement on the first frame.
        // Keep the slot readable until the resolved font generation reprojects it.
        label
            .chars()
            .map(|c| if c.is_ascii() { 0.8 } else { 1.0 })
            .sum::<f32>()
            * metrics.font_size.max(0.0)
    };
    workbench_menu_slot_width_for_paint(label_width, metrics.font_size, metrics.horizontal_inset)
}

thread_local! {
    static MENU_POINTER_CHROME_STENCIL_CACHE: RefCell<Option<MenuPointerChromeStencilCache>> =
        const { RefCell::new(None) };
}

pub(crate) fn build_host_menu_pointer_layout(
    menu_bar: &MenuBarModel,
    chrome: &EditorChromeSnapshot,
    shell_size: UiSize,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
    outer_shell_frames: Option<&BuiltinHostOuterShellFrames>,
) -> HostMenuPointerLayout {
    let (shell_frame, menu_bar_frame) =
        resolved_shell_and_menu_bar_frames(shell_size, outer_shell_frames);
    let menu_count = menu_bar.menus.len().max(MENU_BUTTON_COUNT);
    let menu_labels = menu_bar
        .menus
        .iter()
        .map(|menu| menu.label.as_str())
        .collect::<Vec<_>>();
    let button_frames = menu_button_frames_from_chrome_asset(
        menu_bar_frame,
        &menu_labels,
        menu_count,
        current_menu_label_slot_metrics(),
    );
    let menu_bar_content_width = menu_bar_content_width(&button_frames, shell_frame.x);
    let active_preset_name = active_layout_preset.unwrap_or_default().to_string();
    let resolved_preset_name = if active_preset_name.is_empty() {
        "rider".to_string()
    } else {
        active_preset_name.clone()
    };
    let menus = pointer_menus(menu_bar, preset_names, active_layout_preset);
    let window_item_count = menus
        .get(WINDOW_MENU_INDEX)
        .map(Vec::len)
        .unwrap_or_default();
    let window_popup_height = compute_window_menu_popup_height(
        shell_frame.height,
        button_frames
            .get(WINDOW_MENU_INDEX)
            .copied()
            .unwrap_or(shell_frame),
        window_item_count,
    );
    let intrinsic_popup_widths = measured_root_popup_widths(
        menu_bar,
        preset_names,
        active_layout_preset,
        &resolved_preset_name,
    );
    let popup_widths = resolved_popup_widths(&intrinsic_popup_widths, shell_frame.width);

    HostMenuPointerLayout {
        shell_frame,
        menu_bar_frame,
        button_frames,
        menu_bar_content_width,
        popup_widths: popup_widths.into(),
        intrinsic_popup_widths: intrinsic_popup_widths.into(),
        save_project_enabled: chrome.project_open,
        undo_enabled: chrome.can_undo,
        redo_enabled: chrome.can_redo,
        delete_enabled: chrome.inspector.is_some(),
        preset_names: preset_names.to_vec().into(),
        active_preset_name: Arc::from(active_preset_name),
        resolved_preset_name: Arc::from(resolved_preset_name),
        window_popup_height,
        menu_overflow_mode: chrome.menu_overflow_mode,
        menus: menus.into(),
    }
}

/// Reprojects WindowMetrics geometry while retaining the last published menu semantics.
pub(crate) fn build_host_menu_pointer_geometry_layout(
    previous: &HostMenuPointerLayout,
    menu_bar: &MenuBarModel,
    shell_size: UiSize,
    outer_shell_frames: Option<&BuiltinHostOuterShellFrames>,
) -> HostMenuPointerLayout {
    let (shell_frame, menu_bar_frame) =
        resolved_shell_and_menu_bar_frames(shell_size, outer_shell_frames);
    let menu_labels = menu_bar
        .menus
        .iter()
        .map(|menu| menu.label.as_str())
        .collect::<Vec<_>>();
    let button_frames = menu_button_frames_from_chrome_asset(
        menu_bar_frame,
        &menu_labels,
        menu_labels.len().max(MENU_BUTTON_COUNT),
        current_menu_label_slot_metrics(),
    );
    let menu_bar_content_width = menu_bar_content_width(&button_frames, shell_frame.x);
    let popup_widths =
        resolved_popup_widths(&previous.intrinsic_popup_widths, shell_frame.width).into();
    let window_item_count = previous
        .menus
        .get(WINDOW_MENU_INDEX)
        .map(Vec::len)
        .unwrap_or_default();
    let window_popup_height = compute_window_menu_popup_height(
        shell_frame.height,
        button_frames
            .get(WINDOW_MENU_INDEX)
            .copied()
            .unwrap_or(shell_frame),
        window_item_count,
    );

    zircon_runtime::profile_counter!("editor", "ui.menu_pointer.geometry_product_build_count", 1,);
    HostMenuPointerLayout {
        shell_frame,
        menu_bar_frame,
        button_frames,
        menu_bar_content_width,
        popup_widths,
        intrinsic_popup_widths: Arc::clone(&previous.intrinsic_popup_widths),
        save_project_enabled: previous.save_project_enabled,
        undo_enabled: previous.undo_enabled,
        redo_enabled: previous.redo_enabled,
        delete_enabled: previous.delete_enabled,
        preset_names: Arc::clone(&previous.preset_names),
        active_preset_name: Arc::clone(&previous.active_preset_name),
        resolved_preset_name: Arc::clone(&previous.resolved_preset_name),
        window_popup_height,
        menu_overflow_mode: previous.menu_overflow_mode,
        menus: Arc::clone(&previous.menus),
    }
}

fn resolved_shell_and_menu_bar_frames(
    shell_size: UiSize,
    outer_shell_frames: Option<&BuiltinHostOuterShellFrames>,
) -> (UiFrame, UiFrame) {
    let shell_frame = outer_shell_frames
        .and_then(|frames| frames.shell_frame)
        .unwrap_or_else(|| UiFrame::new(0.0, 0.0, shell_size.width, shell_size.height));
    let menu_bar_frame = outer_shell_frames
        .and_then(|frames| frames.menu_bar_frame)
        .unwrap_or(shell_frame);
    (shell_frame, menu_bar_frame)
}

fn resolved_popup_widths(intrinsic_widths: &[f32], available_width: f32) -> Vec<f32> {
    intrinsic_widths
        .iter()
        .map(|width| width.min(available_width.max(1.0)).max(1.0))
        .collect()
}

fn measured_root_popup_widths(
    menu_bar: &MenuBarModel,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
    resolved_preset_name: &str,
) -> Vec<f32> {
    menu_bar
        .menus
        .iter()
        .enumerate()
        .map(|(menu_index, menu)| {
            let rows = popup_measurement_rows(
                menu,
                preset_names,
                active_layout_preset,
                resolved_preset_name,
            );
            let fallback_width = super::constants::POPUP_WIDTHS
                .get(menu_index)
                .copied()
                .unwrap_or(224.0);
            content_measured_menu_popup_width(
                fallback_width,
                f32::MAX,
                rows.iter()
                    .map(|(label, shortcut)| (label.as_str(), shortcut.as_str())),
                menu_popup_text_width,
            )
        })
        .collect()
}

fn popup_measurement_rows(
    menu: &MenuModel,
    preset_names: &[String],
    active_layout_preset: Option<&str>,
    resolved_preset_name: &str,
) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    if menu.label.eq_ignore_ascii_case("Window") {
        rows.push((
            "Save Preset Asset".to_string(),
            resolved_preset_name.to_string(),
        ));
    }
    rows.extend(menu.items.iter().map(|item| {
        (
            item.label.clone(),
            if item.has_children() {
                ">".to_string()
            } else {
                item.shortcut.clone().unwrap_or_default()
            },
        )
    }));
    if menu.label.eq_ignore_ascii_case("Window") {
        rows.extend(preset_names.iter().map(|preset| {
            (
                preset.clone(),
                if Some(preset.as_str()) == active_layout_preset {
                    "active".to_string()
                } else {
                    String::new()
                },
            )
        }));
    }
    rows
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
            format!("workbench.layout.preset.save.{resolved_preset_name}"),
            true,
        )];
        items.extend(pointer_menu_item_tree(&menu.items));
        items.extend(
            preset_names
                .iter()
                .map(|preset| menu_action(format!("workbench.layout.preset.load.{preset}"), true)),
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
    match menu_item_binding(item).payload() {
        EditorUiBindingPayload::MenuAction { action_id } => Some(action_id.clone()),
        EditorUiBindingPayload::EditorCommand { command_id } => Some(command_id.clone()),
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
    metrics: MenuLabelSlotMetrics,
) -> Vec<UiFrame> {
    let Some(stencil_frames) = menu_pointer_chrome_stencil() else {
        return fallback_menu_button_frames(frame, menu_labels, menu_count, metrics);
    };
    let translated_frames = stencil_frames
        .map(|slot| UiFrame::new(frame.x + slot.x, frame.y + slot.y, slot.width, slot.height));
    menu_button_frames_from_stencil(&translated_frames, menu_labels, menu_count, metrics)
}

fn menu_pointer_chrome_stencil() -> Option<[UiFrame; MENU_BUTTON_COUNT]> {
    let generation = view_template_resource_generation(MENU_CHROME_ASSET, &[])?;
    let cached_frames = MENU_POINTER_CHROME_STENCIL_CACHE.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|cached| cached.generation == generation)
            .map(|cached| cached.frames)
    });
    if let Some(frames) = cached_frames {
        return Some(frames);
    }

    zircon_runtime::profile_counter!(
        "editor",
        "ui.menu_pointer.stencil_projection_build_count",
        1,
    );
    let nodes = build_view_template_node_projection(
        "host.menu.pointer.chrome",
        MENU_CHROME_ASSET,
        &[],
        UiSize::new(MENU_STENCIL_REFERENCE_WIDTH, MENU_STENCIL_REFERENCE_HEIGHT),
        &BTreeMap::new(),
    )
    .ok()?;

    let mut stencil_frames = [UiFrame::default(); MENU_BUTTON_COUNT];
    for node in nodes.iter() {
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
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
            );
        }
    }
    if !stencil_frames.iter().all(|slot| slot.width > 0.0) {
        return None;
    }

    MENU_POINTER_CHROME_STENCIL_CACHE.with(|cache| {
        *cache.borrow_mut() = Some(MenuPointerChromeStencilCache {
            generation,
            frames: stencil_frames,
        });
    });
    Some(stencil_frames)
}

fn menu_button_frames_from_stencil(
    stencil_frames: &[UiFrame; MENU_BUTTON_COUNT],
    menu_labels: &[&str],
    menu_count: usize,
    metrics: MenuLabelSlotMetrics,
) -> Vec<UiFrame> {
    let first = stencil_frames[0];
    let gap = menu_button_gap(stencil_frames).unwrap_or(2.0);
    let mut x = first.x;
    (0..menu_count)
        .map(|index| {
            let label = menu_labels.get(index).copied().unwrap_or_default();
            let width = menu_label_slot_width(label, metrics);
            let slot = UiFrame::new(x, first.y, width, first.height);
            x += width + gap;
            slot
        })
        .collect()
}

fn fallback_menu_button_frames(
    frame: UiFrame,
    menu_labels: &[&str],
    menu_count: usize,
    metrics: MenuLabelSlotMetrics,
) -> Vec<UiFrame> {
    let mut x = frame.x + 8.0;
    (0..menu_count)
        .map(|index| {
            let label = menu_labels.get(index).copied().unwrap_or_default();
            let width = menu_label_slot_width(label, metrics);
            let slot = UiFrame::new(x, frame.y + 2.0, width, 22.0);
            x += width + 4.0;
            slot
        })
        .collect()
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

#[cfg(test)]
#[path = "tests/build_host_menu_pointer_layout_slot_metrics_tests.rs"]
mod slot_metrics_tests;
