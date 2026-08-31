use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_icon_assets::push_icon_asset_pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.icon_name.trim().is_empty() {
        return;
    }
    push_icon_asset_pixels(
        commands,
        node.icon_name.as_str(),
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
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
            1.0,
        );

        assert_eq!(commands.len(), 1);
        assert!(commands[0].image_pixels.is_some());
        assert!(commands[0].image_key.is_none());
    }

    #[test]
    fn missing_svg_icon_button_fails_closed_without_manual_pixel_fallback() {
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
            1.0,
        );

        assert!(commands.is_empty());
    }
}
