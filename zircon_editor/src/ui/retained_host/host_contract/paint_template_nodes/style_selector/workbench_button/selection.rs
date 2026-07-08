use super::super::super::template_style_color::resolved_style_color;
use super::super::resolved_state_for_node;
use super::brightness::apply_visual_brightness;
use super::command::{is_prominent_workbench_command_button, prominent_workbench_command_style};
use super::metrics::workbench_button_border_width;
use super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::palette::{
    add_component_glyph_color, add_component_text_color, workbench_button_selection_palette,
};
use super::states::{base_button_style, is_unavailable_button_interaction};
use super::tab_like::{
    is_asset_browser_tab_like_button, is_asset_browser_toolbar_chip_button,
    is_asset_browser_utility_tab_button, is_tab_like_workbench_button,
    is_workbench_module_tab_button,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::template_component_family::uses_workbench_visual_language;
use zircon_runtime_interface::ui::style::UiStyleColor;

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
        style.text = add_component_text_color();
        style.glyph = add_component_glyph_color();
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

    let active = node.selected || node.checked;
    let selection_palette = workbench_button_selection_palette();
    style.surface = if active {
        selection_palette.tab_hot_surface
    } else if node.hovered || node.popup_open {
        selection_palette.tab_hot_surface
    } else {
        selection_palette.tab_rest_surface
    };
    style.border = selection_palette.border;
    style.border_width = 0.0;
    let text = if active {
        selection_palette.text
    } else {
        selection_palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn asset_browser_toolbar_chip_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked;
    let selection_palette = workbench_button_selection_palette();
    style.surface = if active {
        selection_palette.toolbar_chip_active_surface
    } else if node.hovered || node.popup_open {
        selection_palette.tab_hot_surface
    } else {
        selection_palette.transparent_surface
    };
    style.border = selection_palette.border;
    style.border_width = if active {
        workbench_button_border_width()
    } else {
        0.0
    };
    let text = if active || node.hovered || node.popup_open {
        selection_palette.text
    } else {
        selection_palette.text_muted
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

    let active = node.selected || node.checked;
    let selection_palette = workbench_button_selection_palette();
    style.surface = if active {
        selection_palette.asset_tab_active_surface
    } else if node.hovered || node.popup_open {
        selection_palette.tab_hot_surface
    } else {
        selection_palette.transparent_surface
    };
    style.border = selection_palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        selection_palette.text
    } else {
        selection_palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn asset_browser_utility_tab_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked;
    let selection_palette = workbench_button_selection_palette();
    style.surface = if !active && (node.hovered || node.popup_open) {
        selection_palette.tab_hot_surface
    } else {
        selection_palette.transparent_surface
    };
    style.border = selection_palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        selection_palette.text
    } else {
        selection_palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}

fn quiet_inactive_tab_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchButtonStyle,
) -> WorkbenchButtonStyle {
    let active = node.selected || node.checked;
    let selection_palette = workbench_button_selection_palette();
    style.surface = if active || node.hovered || node.popup_open {
        selection_palette.tab_hot_surface
    } else {
        selection_palette.transparent_surface
    };
    style.border = selection_palette.border;
    style.border_width = 0.0;
    let text = if active || node.hovered || node.popup_open {
        selection_palette.text
    } else {
        selection_palette.text_muted
    };
    style.text = text;
    style.glyph = text;
    style
}
