use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;

type ChatComposerColors = [[u8; 4]; 2];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chat_composer(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(rect.height * 0.5);
    let [surface_color, send_color] = chat_composer_colors_from_host(node, current_host_palette());
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        surface_color,
        1.0,
        radius,
        opacity,
    );
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width - rect.height + 4.0,
            y: rect.y + 4.0,
            width: (rect.height - 8.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        clip,
        order + 1,
        send_color,
        0.0,
        rect.height,
        opacity,
    );
}

fn chat_composer_colors_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> ChatComposerColors {
    [
        super::super::node_background(node).unwrap_or(palette.surface_inset),
        palette.accent,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_chat_composer_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_inset = [10, 11, 12, 255];
        palette.accent = [20, 21, 22, 255];

        assert_eq!(
            chat_composer_colors_from_host(&TemplatePaneNodeData::default(), palette),
            [[10, 11, 12, 255], [20, 21, 22, 255]]
        );
    }
}
