use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::super::template_selection_control_geometry::frame_is_within;

const CHECKBOX_TICK_ICON: &str = "checkmark";

pub(super) fn push_checkbox_tick(
    commands: &mut Vec<HostPaintCommand>,
    mark: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if mark.width < 12.0 || mark.height < 12.0 {
        return;
    }
    let color = PALETTE.accent;
    if push_icon_asset_pixels(
        commands,
        CHECKBOX_TICK_ICON,
        mark,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }
    for tick in checkbox_tick_segments(mark) {
        if frame_is_within(&tick, mark) {
            commands.push(HostPaintCommand::quad(
                tick,
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                1.0,
                opacity,
            ));
        }
    }
}

fn checkbox_tick_segments(mark: &FrameRect) -> [FrameRect; 3] {
    [
        FrameRect {
            x: mark.x + 3.0,
            y: mark.y + 7.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 5.0,
            y: mark.y + 9.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 8.0,
            y: mark.y + 4.0,
            width: 3.0,
            height: 8.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_checkbox_tick_uses_16px_shell_checkmark_asset() {
        let mark = FrameRect {
            x: 10.0,
            y: 6.0,
            width: 16.0,
            height: 16.0,
        };
        let mut commands = Vec::new();

        push_checkbox_tick(&mut commands, &mark, &mark, 3, 1.0);

        let icon_commands = commands
            .iter()
            .filter(|command| command.image_pixels.is_some())
            .collect::<Vec<_>>();
        assert_eq!(icon_commands.len(), 1);
        let icon = icon_commands[0]
            .image_pixels
            .as_ref()
            .expect("checkbox tick should paint real SVG pixels");
        assert_eq!((icon.width, icon.height), (16, 16));
        assert_eq!(icon_commands[0].frame.width, 16.0);
        assert_eq!(icon_commands[0].frame.height, 16.0);
        assert!(
            !icon.resource_key.starts_with("missing-icon:"),
            "checkbox tick should resolve through the shell checkmark asset"
        );
    }
}
