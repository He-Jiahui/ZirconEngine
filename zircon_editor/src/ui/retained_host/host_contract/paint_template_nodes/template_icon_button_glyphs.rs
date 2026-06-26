use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_icon_assets::push_icon_asset_pixels;
use super::template_icon_button_glyph_kind::icon_button_glyph_kind;
use super::template_icon_button_glyph_segments::push_icon_button_glyph_segments;
use super::template_icon_button_glyph_shapes::push_icon_button_glyph_shape;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    state: UiPainterResolvedState,
    opacity: f32,
) {
    let rendered_asset = push_icon_button_asset_glyph(
        commands,
        node.icon_name.as_str(),
        rect,
        clip,
        order,
        color,
        opacity,
    );
    if !rendered_asset {
        push_icon_button_glyph_shape(
            commands,
            icon_button_glyph_kind(node),
            rect,
            clip,
            order,
            color,
            opacity,
        );
    }

    if state == UiPainterResolvedState::Pressed {
        push_icon_button_glyph_segments(
            commands,
            rect,
            clip,
            order + 3,
            color,
            opacity * 0.28,
            &[(2.0, 13.0, 12.0, 1.0)],
        );
    }
}

fn push_icon_button_asset_glyph(
    commands: &mut Vec<HostPaintCommand>,
    icon_name: &str,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) -> bool {
    push_icon_asset_pixels(commands, icon_name, rect, clip, order, Some(color), opacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::{
        TemplateNodeFrameData, TemplatePaneNodeData,
    };

    #[test]
    fn real_svg_icon_button_prefers_asset_pixels_over_fallback_glyph() {
        let node = TemplatePaneNodeData {
            control_id: "WorkbenchToolbarOpen".into(),
            role: "IconButton".into(),
            icon_name: "editor_pages/workbench/menu/open-project.svg".into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        };
        let mut commands = Vec::new();

        push_icon_button_glyph(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            [203, 210, 220, 255],
            UiPainterResolvedState::Normal,
            1.0,
        );

        assert_eq!(commands.len(), 1);
        assert!(commands[0].image_pixels.is_some());
        assert!(commands[0].image_key.is_none());
    }

    #[test]
    fn missing_svg_icon_button_keeps_manual_glyph_fallback() {
        let node = TemplatePaneNodeData {
            control_id: "WorkbenchToolbarMenu".into(),
            role: "IconButton".into(),
            icon_name: "missing-zircon-toolbar-icon.svg".into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        };
        let mut commands = Vec::new();

        push_icon_button_glyph(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            [203, 210, 220, 255],
            UiPainterResolvedState::Normal,
            1.0,
        );

        assert!(commands.len() > 1);
        assert!(commands
            .iter()
            .all(|command| command.image_pixels.is_none()));
    }
}
