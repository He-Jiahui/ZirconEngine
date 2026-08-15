use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::{workbench_icon_button_palette, WorkbenchIconButtonPalette};
use super::super::state::{icon_button_node_uses_active_glyph, is_unavailable_icon_button_state};
use super::declared::declared_icon_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::typed_button_tone_color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_color(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> [u8; 4] {
    let palette = workbench_icon_button_palette();
    if is_unavailable_icon_button_state(state) {
        palette.text_disabled
    } else if danger {
        palette.error
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => palette.focus_ring,
            UiPainterResolvedState::Focused => {
                if icon_button_node_uses_active_glyph(node) {
                    palette.focus_ring
                } else if node.hovered {
                    palette.text
                } else {
                    normal_icon_glyph_color(node, context, palette)
                }
            }
            UiPainterResolvedState::Hovered => palette.text,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.text_disabled
            }
            UiPainterResolvedState::Normal => normal_icon_glyph_color(node, context, palette),
        }
    }
}

fn normal_icon_glyph_color(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    palette: WorkbenchIconButtonPalette,
) -> [u8; 4] {
    declared_icon_color(node)
        .or_else(|| typed_button_tone_color(node))
        .unwrap_or_else(|| {
            if context == WorkbenchIconButtonContext::Rail {
                palette.muted
            } else {
                palette.normal
            }
        })
}
