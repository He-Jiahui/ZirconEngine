use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::push_inspector_segments;

const INSPECTOR_CHECK_ICON: &str = "zircon_editor_shell/controls/check.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_check_tick(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_CHECK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }
    let Some(parts) = check_tick_fallback_segments(rect) else {
        return;
    };
    push_inspector_segments(commands, clip, order, color, opacity, &parts, 1.0);
}

fn check_tick_fallback_segments(rect: &FrameRect) -> Option<[FrameRect; 3]> {
    if !is_finite_rect(rect) || rect.width < 11.0 || rect.height < 12.0 {
        return None;
    }
    Some([
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 7.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 9.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 8.0,
            y: rect.y + 4.0,
            width: 3.0,
            height: 8.0,
        },
    ])
}

fn is_finite_rect(rect: &FrameRect) -> bool {
    rect.x.is_finite() && rect.y.is_finite() && rect.width.is_finite() && rect.height.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_tick_fallback_skips_a_slot_that_cannot_contain_its_grid() {
        let tiny = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
        let minimum = FrameRect {
            x: 2.0,
            y: 3.0,
            width: 11.0,
            height: 12.0,
        };

        assert!(check_tick_fallback_segments(&tiny).is_none());
        for segment in check_tick_fallback_segments(&minimum).expect("minimum grid should fit") {
            assert!(segment.x >= minimum.x);
            assert!(segment.y >= minimum.y);
            assert!(segment.x + segment.width <= minimum.x + minimum.width);
            assert!(segment.y + segment.height <= minimum.y + minimum.height);
        }
    }
}
