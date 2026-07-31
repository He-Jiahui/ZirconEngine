use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::push_inspector_segments;

const INSPECTOR_MESH_ICON: &str = "zircon_editor_shell/inspector/mesh-renderer.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_MESH_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }
    let Some(parts) = cube_fallback_segments(rect) else {
        return;
    };
    push_inspector_segments(commands, clip, order, color, opacity, &parts, 1.0);
}

fn cube_fallback_segments(rect: &FrameRect) -> Option<[FrameRect; 3]> {
    if !is_finite_rect(rect) || rect.width < 7.0 || rect.height < 8.0 {
        return None;
    }
    Some([
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 3.0,
            width: rect.width - 6.0,
            height: rect.height - 6.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 1.0,
            width: rect.width - 6.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + rect.width - 3.0,
            y: rect.y + 4.0,
            width: 2.0,
            height: rect.height - 7.0,
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
    fn cube_fallback_skips_a_slot_that_cannot_contain_its_grid() {
        let tiny = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
        let minimum = FrameRect {
            x: 2.0,
            y: 3.0,
            width: 7.0,
            height: 8.0,
        };

        assert!(cube_fallback_segments(&tiny).is_none());
        for segment in cube_fallback_segments(&minimum).expect("minimum grid should fit") {
            assert!(segment.x >= minimum.x);
            assert!(segment.y >= minimum.y);
            assert!(segment.x + segment.width <= minimum.x + minimum.width);
            assert!(segment.y + segment.height <= minimum.y + minimum.height);
        }
    }
}
