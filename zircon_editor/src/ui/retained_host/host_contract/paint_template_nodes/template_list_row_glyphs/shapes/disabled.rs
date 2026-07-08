use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{push_segments, GlyphSegmentSpec};

const DISABLED_DIAMOND_SEGMENTS: [GlyphSegmentSpec; 4] = [
    GlyphSegmentSpec::new(7, 3, 2, 2),
    GlyphSegmentSpec::new(11, 7, 2, 2),
    GlyphSegmentSpec::new(7, 11, 2, 2),
    GlyphSegmentSpec::new(3, 7, 2, 2),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_disabled_diamond(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        &DISABLED_DIAMOND_SEGMENTS,
        clip,
        order,
        color,
        opacity,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_diamond_fallback_uses_supplied_palette_tint() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 16.0,
        };
        let tint = [17, 18, 19, 20];
        let mut commands = Vec::new();

        push_disabled_diamond(&mut commands, &rect, &rect, 0, tint, 1.0);

        assert!(!commands.is_empty());
        assert!(commands
            .iter()
            .all(|command| command.background_color == Some(tint)));
    }
}
