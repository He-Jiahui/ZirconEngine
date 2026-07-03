use super::super::super::template_style_color::resolved_style_color;
use super::super::resolved_state_for_node;
use super::brightness::apply_visual_brightness;
use super::command::{is_prominent_workbench_command_button, prominent_workbench_command_style};
use super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::palette::{ADD_COMPONENT_GLYPH, ADD_COMPONENT_TEXT};
use super::states::{base_button_style, is_unavailable_button_interaction};
use super::tab_like::{
    is_asset_browser_tab_like_button, is_asset_browser_toolbar_chip_button,
    is_asset_browser_utility_tab_button, is_tab_like_workbench_button,
    is_workbench_module_tab_button,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use crate::ui::retained_host::host_contract::template_component_family::uses_workbench_visual_language;
use zircon_runtime_interface::ui::style::UiStyleColor;

const TRANSPARENT_SURFACE: [u8; 4] = [0, 0, 0, 0];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_button_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
    is_add_component_button: bool,
) -> WorkbenchButtonStyle {
    let state = resolved_state_for_node(node);
    let interaction = state.button_interaction_state();
    let mut style = base_button_style(kind, interaction);
    style.interaction = interaction;

    if is_unavailable_button_interaction(interaction) {
        return style;
    }

    if should_apply_declared_button_surface(node, kind) {
        if let Some(surface) =
            declared_button_style_color(node.button_style.element.background_color.as_ref())
        {
            style.surface = surface;
        }
        if let Some(border) =
            declared_button_style_color(node.button_style.element.border_color.as_ref())
        {
            style.border = border;
        }
    }
    if should_apply_declared_button_foreground(node, kind) {
        if let Some(text) =
            declared_button_style_color(node.button_style.element.foreground_color.as_ref())
        {
            style.text = text;
            style.glyph = text;
        }
    }
    if is_add_component_button {
        style.text = ADD_COMPONENT_TEXT;
        style.glyph = ADD_COMPONENT_GLYPH;
    }
    if is_prominent_workbench_command_button(node) {
        style = prominent_workbench_command_style(node, style);
    }
    if is_tab_like_workbench_button(node) {
        style = tab_like_button_style(node, style);
    }
    apply_visual_brightness(style, node.label_brightness)
}

fn declared_button_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}

fn should_apply_declared_button_surface(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> bool {
    uses_workbench_visual_language(node)
        && !matches!(
            kind,
            WorkbenchButtonKind::Primary | WorkbenchButtonKind::Danger
        )
}

fn should_apply_declared_button_foreground(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> bool {
    uses_workbench_visual_language(node) && kind != WorkbenchButtonKind::Primary
}

fn tab_like_button_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    if is_asset_browser_toolbar_chip_button(node) {
        return asset_browser_toolbar_chip_style(node, style);
    }
    if is_asset_browser_tab_like_button(node) {
        return asset_browser_tab_like_style(node, style);
    }
    if is_workbench_module_tab_button(node) {
        return quiet_inactive_tab_style(node, style);
    }

    let active = node.selected || node.checked || node.focused;
    let palette = current_host_palette();
    style.surface = if active {
        palette.surface_hover
    } else if node.hovered || node.popup_open {
        palette.surface_hover
    } else {
        palette.surface_pressed
    };
    style.border = palette.border;
    style.border_width = 0.0;
    let text = if active {
        palette.text
    } else {
        palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn asset_browser_toolbar_chip_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused;
    let palette = current_host_palette();
    style.surface = if active {
        palette.surface
    } else if node.hovered || node.popup_open {
        palette.surface_hover
    } else {
        TRANSPARENT_SURFACE
    };
    style.border = palette.border;
    style.border_width = if active { 1.0 } else { 0.0 };
    let text = if active || node.hovered || node.popup_open {
        palette.text
    } else {
        palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn asset_browser_tab_like_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    if is_asset_browser_utility_tab_button(node) {
        return asset_browser_utility_tab_style(node, style);
    }

    let active = node.selected || node.checked || node.focused;
    let palette = current_host_palette();
    style.surface = if active {
        palette.surface_pressed
    } else if node.hovered || node.popup_open {
        palette.surface_hover
    } else {
        TRANSPARENT_SURFACE
    };
    style.border = palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        palette.text
    } else {
        palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn asset_browser_utility_tab_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused;
    let palette = current_host_palette();
    style.surface = if !active && (node.hovered || node.popup_open) {
        palette.surface_hover
    } else {
        TRANSPARENT_SURFACE
    };
    style.border = palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        palette.text
    } else {
        palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn quiet_inactive_tab_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked || node.focused;
    let palette = current_host_palette();
    style.surface = if active || node.hovered || node.popup_open {
        palette.surface_hover
    } else {
        TRANSPARENT_SURFACE
    };
    style.border = palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        palette.text
    } else {
        palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}
