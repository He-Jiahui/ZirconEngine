use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rounded_border_clipped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_border_width(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
) {
    draw_rounded_border_clipped(frame, rect.clone(), clip, color, border_width, 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_square_border_resolves_a_partial_inner_edge() {
        let mut frame = HostRgbaFrame::filled(8, 8, [0, 0, 0, 255]);
        let rect = FrameRect {
            x: 1.0,
            y: 1.0,
            width: 6.0,
            height: 6.0,
        };

        draw_border_width(&mut frame, &rect, None, [255, 255, 255, 255], 1.5);

        let inner_edge = pixel(&frame, 2, 3);
        assert!(
            (1..255).contains(&inner_edge[0]),
            "1.5px square border must retain fractional inner coverage: {inner_edge:?}"
        );
        assert_eq!(inner_edge[0], inner_edge[1]);
        assert_eq!(inner_edge[1], inner_edge[2]);
        assert_eq!(pixel(&frame, 3, 3), [0, 0, 0, 255]);
    }

    fn pixel(frame: &HostRgbaFrame, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
        frame.as_bytes()[offset..offset + 4]
            .try_into()
            .expect("RGBA pixel")
    }
}
