use zircon_runtime::core::framework::render::CapturedFrame;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiPaintPayload, UiRenderCommand, UiResolvedTextBox, UiTextPaintDecoration,
        UiTextPaintDecorationKind,
    },
};

pub(in super::super) fn assert_bbcode_table_pixels(
    samples: &[UiRenderCommand],
    capture: &CapturedFrame,
    background: &CapturedFrame,
) {
    assert_table_cell_pixels(
        &samples[super::BBCODE_TABLE_SAMPLE_INDEX],
        capture,
        background,
        "horizontal",
    );
}

pub(in super::super) fn assert_vertical_bbcode_table_pixels(
    samples: &[UiRenderCommand],
    capture: &CapturedFrame,
    background: &CapturedFrame,
) {
    assert_table_cell_pixels(
        &samples[super::VERTICAL_BBCODE_TABLE_SAMPLE_INDEX],
        capture,
        background,
        "vertical",
    );
}

fn assert_table_cell_pixels(
    command: &UiRenderCommand,
    capture: &CapturedFrame,
    background: &CapturedFrame,
    orientation: &str,
) {
    let layout = command
        .text_layout
        .as_ref()
        .expect("table framebuffer proof must retain a resolved layout");
    let element = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = element.payload else {
        panic!("table framebuffer proof must project a text paint payload");
    };

    for (box_index, text_box) in layout.boxes.iter().enumerate() {
        for kind in [
            UiTextPaintDecorationKind::TableCellBackground,
            UiTextPaintDecorationKind::TableCellBorder,
        ] {
            let decoration = table_decoration_for_box(&text.decorations, text_box, kind);
            let linear_color = table_decoration_linear_color(decoration);
            let minimum = table_decoration_minimum(decoration);
            let label = format!("{orientation} table cell {box_index} {kind:?}");
            let evidence_frames = table_decoration_evidence_frames(decoration);
            let changed = evidence_frames
                .iter()
                .map(|frame| {
                    super::super::count_changed_pixels_in_frame(
                        &capture.rgba,
                        &background.rgba,
                        capture.width,
                        capture.height,
                        *frame,
                        6,
                    )
                })
                .sum::<usize>();
            assert!(
                changed >= minimum,
                "{label} must change its rendered evidence bands against the background capture: frame={:?}, changed={changed}, minimum={minimum}",
                decoration.frame,
            );
            let matching = evidence_frames
                .iter()
                .map(|frame| super::count_near_color_coverage(capture, *frame, linear_color))
                .sum::<usize>();
            assert!(
                matching >= minimum,
                "{label} must expose its color in rendered evidence bands: frame={:?}, matching={matching}, minimum={minimum}",
                decoration.frame,
            );
        }
    }
}

fn table_decoration_for_box<'a>(
    decorations: &'a [UiTextPaintDecoration],
    text_box: &UiResolvedTextBox,
    kind: UiTextPaintDecorationKind,
) -> &'a UiTextPaintDecoration {
    let matches = decorations
        .iter()
        .filter(|decoration| {
            decoration.kind == kind
                && decoration.range == text_box.range
                && decoration.frame == text_box.frame
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "each resolved table cell must project one {kind:?} decoration: box={text_box:?}"
    );
    matches[0]
}

fn table_decoration_linear_color(decoration: &UiTextPaintDecoration) -> [u8; 3] {
    let color = decoration.color.as_str();
    assert_eq!(
        color.len(),
        9,
        "table decoration colors must carry explicit opaque #RRGGBBAA paint data: {color}"
    );
    assert_eq!(
        color.as_bytes().first(),
        Some(&b'#'),
        "table decoration colors must be hexadecimal: {color}"
    );
    assert_eq!(
        &color[7..9],
        "FF",
        "product table fixture requires opaque cell decorations: {color}"
    );
    [
        parse_hex_channel(color, 1),
        parse_hex_channel(color, 3),
        parse_hex_channel(color, 5),
    ]
}

fn parse_hex_channel(color: &str, offset: usize) -> u8 {
    u8::from_str_radix(&color[offset..offset + 2], 16)
        .unwrap_or_else(|_| panic!("table decoration color must be hexadecimal: {color}"))
}

fn table_decoration_minimum(decoration: &UiTextPaintDecoration) -> usize {
    let width = decoration.frame.width.ceil().max(1.0) as usize;
    let height = decoration.frame.height.ceil().max(1.0) as usize;
    match decoration.kind {
        UiTextPaintDecorationKind::TableCellBackground => (width * height / 20).clamp(16, 128),
        UiTextPaintDecorationKind::TableCellBorder => {
            let perimeter = width.saturating_add(height).saturating_mul(2);
            (perimeter * decoration.thickness.ceil().max(1.0) as usize / 8).clamp(8, 96)
        }
        _ => unreachable!("table pixel assertion only accepts table decorations"),
    }
}

fn table_decoration_evidence_frames(decoration: &UiTextPaintDecoration) -> Vec<UiFrame> {
    match decoration.kind {
        UiTextPaintDecorationKind::TableCellBackground => vec![decoration.frame],
        UiTextPaintDecorationKind::TableCellBorder => border_evidence_frames(
            decoration.frame,
            decoration
                .thickness
                .min(decoration.frame.width * 0.5)
                .min(decoration.frame.height * 0.5)
                .max(1.0),
        ),
        _ => unreachable!("table pixel assertion only accepts table decorations"),
    }
}

fn border_evidence_frames(frame: UiFrame, width: f32) -> Vec<UiFrame> {
    let vertical_height = (frame.height - width * 2.0).max(0.0);
    let mut frames = vec![
        UiFrame::new(frame.x, frame.y, frame.width, width),
        UiFrame::new(frame.x, frame.bottom() - width, frame.width, width),
    ];
    if vertical_height > 0.0 {
        frames.push(UiFrame::new(
            frame.x,
            frame.y + width,
            width,
            vertical_height,
        ));
        frames.push(UiFrame::new(
            frame.right() - width,
            frame.y + width,
            width,
            vertical_height,
        ));
    }
    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_color_contract_retains_opaque_linear_channels() {
        let decoration = UiTextPaintDecoration::table_cell_background(
            Default::default(),
            UiFrame::new(0.0, 0.0, 10.0, 10.0),
            "#102638FF",
        );

        assert_eq!(
            table_decoration_linear_color(&decoration),
            [0x10, 0x26, 0x38]
        );
        assert_eq!(table_decoration_minimum(&decoration), 16);
    }

    #[test]
    fn table_border_minimum_scales_with_the_resolved_perimeter() {
        let decoration = UiTextPaintDecoration::table_cell_border(
            Default::default(),
            UiFrame::new(0.0, 0.0, 200.0, 40.0),
            "#73D7FFFF",
            1.0,
        );

        assert_eq!(table_decoration_minimum(&decoration), 60);
    }

    #[test]
    fn table_border_evidence_excludes_the_cell_center() {
        assert_eq!(
            border_evidence_frames(UiFrame::new(4.0, 8.0, 200.0, 40.0), 1.0),
            vec![
                UiFrame::new(4.0, 8.0, 200.0, 1.0),
                UiFrame::new(4.0, 47.0, 200.0, 1.0),
                UiFrame::new(4.0, 9.0, 1.0, 38.0),
                UiFrame::new(203.0, 9.0, 1.0, 38.0),
            ]
        );
    }
}
