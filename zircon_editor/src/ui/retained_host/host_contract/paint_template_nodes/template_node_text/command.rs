use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::template_node_text_line_height;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_command(
    commands: &mut Vec<HostPaintCommand>,
    text_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    color: [u8; 4],
    font_size: f32,
    text_style: UiTextRunPaintStyle,
    opacity: f32,
) {
    if label.trim().is_empty()
        || !is_paintable_text_rect(text_rect)
        || !is_paintable_font(font_size)
    {
        return;
    }
    commands.push(HostPaintCommand::wrapped_text(
        FrameRect {
            x: text_rect.x,
            y: text_rect.y,
            width: text_rect.width,
            height: text_rect.height,
        },
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        template_node_text_line_height(font_size),
        text_style,
        opacity,
    ));
}

fn is_paintable_text_rect(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn is_paintable_font(font_size: f32) -> bool {
    font_size.is_finite() && font_size > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

    #[test]
    fn template_node_text_command_uses_shared_runtime_line_height() {
        let mut commands = Vec::new();
        let frame = FrameRect {
            x: 4.0,
            y: 6.0,
            width: 80.0,
            height: 20.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 32.0,
        };

        push_text_command(
            &mut commands,
            &frame,
            &clip,
            7,
            "Caption".to_string(),
            [226, 230, 232, 255],
            10.666_667,
            UiTextRunPaintStyle {
                code: true,
                ..UiTextRunPaintStyle::default()
            },
            1.0,
        );

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].frame, frame);
        assert_eq!(commands[0].clip_frame.as_ref(), Some(&clip));
        assert_eq!(commands[0].z_index, 7);
        assert_eq!(commands[0].text.as_deref(), Some("Caption"));
        assert_eq!(
            commands[0].text_layout_policy,
            HostTextLayoutPolicy::WordWrap
        );
        assert!(commands[0].text_style.code);
        assert_eq!(
            commands[0].line_height,
            template_node_text_line_height(10.666_667)
        );
    }

    #[test]
    fn template_node_text_command_skips_an_empty_or_unpaintable_slot() {
        let mut commands = Vec::new();
        let rect = FrameRect {
            x: 4.0,
            y: 6.0,
            width: 0.0,
            height: 20.0,
        };

        push_text_command(
            &mut commands,
            &rect,
            &rect,
            7,
            "Caption".to_string(),
            [226, 230, 232, 255],
            10.666_667,
            UiTextRunPaintStyle::default(),
            1.0,
        );
        let populated_rect = FrameRect {
            width: 80.0,
            ..rect.clone()
        };
        push_text_command(
            &mut commands,
            &populated_rect,
            &populated_rect,
            8,
            String::new(),
            [226, 230, 232, 255],
            10.666_667,
            UiTextRunPaintStyle::default(),
            1.0,
        );

        assert!(commands.is_empty());
    }
}
