use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;
use super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, template_image_tint,
    template_vector_image_pixels,
};
use super::geometry::image_rect_for_node;
use super::identity::{is_icon_node, template_node_has_image_source};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_image_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !template_node_has_image_source(node) {
        return;
    }
    let preview_size = node.preview_image.size();
    let materialization_rect =
        image_materialization_rect(node, rect, preview_size.width, preview_size.height);
    if !is_visible_frame(&materialization_rect) {
        return;
    }
    let Some(damage_frame) = intersect(&materialization_rect, clip) else {
        return;
    };
    let Some((target_width, target_height)) =
        raster_size_from_frame(materialization_rect.width, materialization_rect.height)
    else {
        return;
    };
    let tint = template_node_image_tint(node);
    let image = {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_node_image_pixels");
        if node.role.as_str() == "SvgIcon" {
            template_vector_image_pixels(
                &node.preview_image,
                node.media_source.as_str(),
                node.icon_name.as_str(),
                target_width,
                target_height,
                tint,
                !is_icon_node(node),
                Some(damage_frame),
            )
        } else {
            template_image_pixels(
                &node.preview_image,
                node.media_source.as_str(),
                node.icon_name.as_str(),
                target_width,
                target_height,
                tint,
                !is_icon_node(node),
                Some(damage_frame),
            )
        }
    };
    let Some(image) = image else {
        return;
    };
    let image_rect = image_rect_for_node(node, rect, image.width, image.height);
    if !is_visible_frame(&image_rect) || intersect(&image_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
}

fn image_materialization_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    preview_width: u32,
    preview_height: u32,
) -> FrameRect {
    if !is_icon_node(node) && (preview_width == 0 || preview_height == 0) {
        return rect.clone();
    }
    image_rect_for_node(node, rect, preview_width, preview_height)
}

fn template_node_image_tint(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    template_image_tint(
        is_icon_node(node),
        node.selected || node.checked || node.pressed || node.popup_open,
        node.disabled,
        node.text_tone.as_str(),
        node.validation_level.as_str(),
        resolved_style_color(node.button_style.element.foreground_color.as_ref()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::{
        ICON_TINT, ICON_TINT_ACTIVE,
    };

    fn icon_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            icon_name: "zircon_editor_shell/toolbar/compile.svg".into(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn focused_icon_image_does_not_use_active_tint() {
        let node = TemplatePaneNodeData {
            focused: true,
            ..icon_node()
        };

        assert_eq!(template_node_image_tint(&node), Some(ICON_TINT));
    }

    #[test]
    fn checked_icon_image_uses_active_tint_without_focus() {
        let node = TemplatePaneNodeData {
            checked: true,
            ..icon_node()
        };

        assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
    }

    #[test]
    fn popup_open_icon_image_uses_active_tint_without_focus() {
        let node = TemplatePaneNodeData {
            popup_open: true,
            ..icon_node()
        };

        assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
    }

    #[test]
    fn unknown_image_aspect_uses_the_visible_container_for_materialization() {
        let node = TemplatePaneNodeData {
            role: "Image".into(),
            media_source: "ui/editor/showcase_checker.svg".into(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 80.0,
        };

        let materialization_rect = image_materialization_rect(&node, &rect, 0, 0);

        assert_eq!(materialization_rect.x, rect.x);
        assert_eq!(materialization_rect.y, rect.y);
        assert_eq!(materialization_rect.width, rect.width);
        assert_eq!(materialization_rect.height, rect.height);
    }

    #[test]
    fn vector_raster_bucket_does_not_change_the_command_frame() {
        let node = TemplatePaneNodeData {
            role: "SvgIcon".into(),
            icon_name: "folder-open-outline".into(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 54.0,
            height: 54.0,
        };
        let expected_frame = image_materialization_rect(&node, &rect, 0, 0);
        let mut commands = Vec::new();

        push_template_image_command(&mut commands, &node, &rect, &rect, 7, 1.0);

        assert_eq!(commands.len(), 1);
        let command = &commands[0];
        assert_eq!(command.frame.x, expected_frame.x);
        assert_eq!(command.frame.y, expected_frame.y);
        assert_eq!(command.frame.width, expected_frame.width);
        assert_eq!(command.frame.height, expected_frame.height);
        let pixels = command
            .image_pixels
            .as_ref()
            .expect("SVG icon command should carry cached raster pixels");
        assert_eq!((pixels.width, pixels.height), (48, 48));
        assert_ne!(pixels.width as f32, command.frame.width);
    }
}
