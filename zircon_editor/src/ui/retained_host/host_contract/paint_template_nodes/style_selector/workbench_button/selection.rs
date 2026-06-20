use super::super::super::template_style_color::resolved_style_color;
use super::super::resolved_state_for_node;
use super::brightness::apply_visual_brightness;
use super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::palette::{ADD_COMPONENT_GLYPH, ADD_COMPONENT_TEXT};
use super::states::{base_button_style, is_unavailable_button_interaction};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
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
    if let Some(text) =
        declared_button_style_color(node.button_style.element.foreground_color.as_ref())
    {
        style.text = text;
        style.glyph = text;
    }
    if is_add_component_button {
        style.text = ADD_COMPONENT_TEXT;
        style.glyph = ADD_COMPONENT_GLYPH;
    }
    apply_visual_brightness(style, node.label_brightness)
}

fn declared_button_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}
