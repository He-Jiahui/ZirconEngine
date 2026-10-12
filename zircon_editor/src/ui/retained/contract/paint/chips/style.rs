//! chip 视觉状态与宿主 palette 的投影；禁用、按压/弹出、选择、悬停和焦点边框有明确优先级。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::style_selector::focus_visible_for_node;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 从宿主主题抽取 chip 自有状态色；共享 palette 变动需同时检查文本与边框对比。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchChipPalette {
    pub surface: [u8; 4],
    pub hover_surface: [u8; 4],
    pub pressed_surface: [u8; 4],
    pub selected_surface: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub border: [u8; 4],
    pub selected_border: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
    pub border_disabled: [u8; 4],
    pub focus_ring: [u8; 4],
    pub accent: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_palette(
) -> WorkbenchChipPalette {
    workbench_chip_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchChipPalette {
    WorkbenchChipPalette {
        surface: palette.surface,
        hover_surface: palette.surface_hover,
        pressed_surface: palette.surface_pressed,
        selected_surface: palette.surface_selected,
        surface_disabled: palette.surface_disabled,
        border: palette.border,
        selected_border: palette.accent_soft,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        border_disabled: palette.border_disabled,
        focus_ring: palette.focus_ring,
        accent: palette.accent,
    }
}

/// 已接管 chip 的交互表面颜色；禁用优先于弹出和选中。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_surface(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.surface_disabled
    } else if node.pressed || node.popup_open {
        palette.pressed_surface
    } else if node.selected || node.checked {
        palette.selected_surface
    } else if node.hovered {
        palette.hover_surface
    } else {
        palette.surface
    }
}

/// 焦点可见边框优先于按压和选中，用于键盘可感知性。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.border_disabled
    } else if focus_visible_for_node(node) {
        palette.focus_ring
    } else if node.pressed || node.popup_open {
        palette.border
    } else if node.selected || node.checked {
        palette.selected_border
    } else if node.hovered {
        palette.border
    } else {
        // Slate's compact toolbar controls remain visibly bounded at rest.
        // The standard border keeps adjacent filter and viewport chips from
        // collapsing into panel text while the hover, selection, and focus
        // states above retain their stronger visual signals.
        palette.border
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.text_disabled
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        palette.text_muted
    } else {
        palette.text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.text_disabled
    } else if node.pressed || node.popup_open {
        palette.accent
    } else {
        palette.text_muted
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
