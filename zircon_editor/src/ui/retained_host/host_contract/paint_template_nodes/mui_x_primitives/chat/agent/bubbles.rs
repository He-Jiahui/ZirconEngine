use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::{MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION, MUI_X_CHAT_INSET};

type AgentBubbleColors = [[u8; 4]; 2];

pub(super) fn push_agent_bubbles(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    let [agent_color, user_color] = agent_bubble_colors_from_host(current_host_palette());
    super::super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: rect.width * 0.58,
            height: bubble_height,
        },
        clip,
        order,
        agent_color,
        0.0,
        5.0,
        opacity,
    );
    super::super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
        clip,
        order + 1,
        user_color,
        0.0,
        5.0,
        opacity,
    );
}

fn agent_bubble_colors_from_host(palette: HostMaterialPalette) -> AgentBubbleColors {
    [palette.surface, palette.surface_selected]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_agent_bubble_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface = [10, 11, 12, 255];
        palette.surface_selected = [20, 21, 22, 255];

        assert_eq!(
            agent_bubble_colors_from_host(palette),
            [[10, 11, 12, 255], [20, 21, 22, 255]]
        );
    }
}
