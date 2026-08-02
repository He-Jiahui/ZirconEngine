use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::{MUI_X_CHAT_INSET, MUI_X_CHAT_STREAMING_HEIGHT};

pub(super) fn push_agent_streaming_indicator(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !(node.component_variant.as_str().contains("streaming") || node.popup_open) {
        return;
    }

    let color = streaming_indicator_color_from_host(current_host_palette());
    super::super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + rect.height - MUI_X_CHAT_INSET,
            width: (rect.width * 0.42).max(1.0),
            height: MUI_X_CHAT_STREAMING_HEIGHT,
        },
        clip,
        order,
        color,
        0.0,
        MUI_X_CHAT_STREAMING_HEIGHT * 0.5,
        opacity,
    );
}

fn streaming_indicator_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.accent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_streaming_indicator_color_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [10, 11, 12, 255];

        assert_eq!(
            streaming_indicator_color_from_host(palette),
            [10, 11, 12, 255]
        );
    }
}
