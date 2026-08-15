use unicode_segmentation::UnicodeSegmentation;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiPaintPayload, UiRenderCommand, UiTextPaintDecorationKind, UiTextWritingMode},
};

#[path = "proof_assertions/msdf_pixels.rs"]
mod msdf_pixels;
#[path = "proof_assertions/table_pixels.rs"]
mod table_pixels;

pub(super) use msdf_pixels::assert_msdf_sharp_corner_pixels;
pub(super) use table_pixels::{assert_bbcode_table_pixels, assert_vertical_bbcode_table_pixels};

const BBCODE_TABLE_SAMPLE_INDEX: usize = 18;
const VERTICAL_BBCODE_TABLE_SAMPLE_INDEX: usize = 19;
const FRAME_EPSILON: f32 = 0.01;

pub(super) fn assert_mixed_bidi_editable_geometry(samples: &[UiRenderCommand]) {
    let command = sample_by_node(samples, 128);
    let layout = command
        .text_layout
        .as_ref()
        .expect("mixed-BiDi editable proof must own a resolved layout");
    let editable = layout
        .editable
        .as_ref()
        .expect("mixed-BiDi editable proof state");
    assert_eq!(editable.caret.offset, "LTR abc ".len());

    let element = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = element.payload else {
        panic!("mixed-BiDi proof must project a text paint payload");
    };
    let selections = text
        .decorations
        .iter()
        .filter(|decoration| decoration.kind == UiTextPaintDecorationKind::Selection)
        .collect::<Vec<_>>();
    let compositions = text
        .decorations
        .iter()
        .filter(|decoration| decoration.kind == UiTextPaintDecorationKind::CompositionUnderline)
        .collect::<Vec<_>>();
    let highlights = text
        .decorations
        .iter()
        .filter(|decoration| decoration.kind == UiTextPaintDecorationKind::CompositionHighlight)
        .collect::<Vec<_>>();
    let caret = text
        .decorations
        .iter()
        .find(|decoration| decoration.kind == UiTextPaintDecorationKind::Caret)
        .expect("mixed-BiDi caret decoration");

    assert_eq!(
        selections.len(),
        2,
        "mixed-BiDi source selection must split into two visual spans"
    );
    assert_eq!(compositions.len(), 1);
    assert_eq!(highlights.len(), 1);
    assert!(selections[0].frame.right() < selections[1].frame.x);
    assert_eq!(highlights[0].range, compositions[0].range);
    assert_eq!(highlights[0].frame.x, compositions[0].frame.x);
    assert_eq!(highlights[0].frame.width, compositions[0].frame.width);
    assert!(highlights[0].frame.height > compositions[0].frame.height);
    assert!(caret.frame.x + FRAME_EPSILON >= compositions[0].frame.right());
}

pub(super) fn assert_mixed_bidi_editable_pixels(
    samples: &[UiRenderCommand],
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    background: &zircon_runtime::core::framework::render::CapturedFrame,
) {
    let command = sample_by_node(samples, 128);
    let element = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = element.payload else {
        panic!("mixed-BiDi proof must project a text paint payload");
    };
    for decoration in text.decorations.iter().filter(|decoration| {
        matches!(
            decoration.kind,
            UiTextPaintDecorationKind::Selection
                | UiTextPaintDecorationKind::CompositionHighlight
                | UiTextPaintDecorationKind::CompositionUnderline
                | UiTextPaintDecorationKind::Caret
        )
    }) {
        let changed = super::count_changed_pixels_in_frame(
            &capture.rgba,
            &background.rgba,
            capture.width,
            capture.height,
            decoration.frame,
            6,
        );
        match decoration.kind {
            UiTextPaintDecorationKind::Selection => {
                let frame_area = decoration.frame.width.ceil().max(1.0) as usize
                    * decoration.frame.height.ceil().max(1.0) as usize;
                assert!(
                    changed >= (frame_area / 3).max(12),
                    "selection fill must cover its real framebuffer frame: frame={:?}, changed={changed}, area={frame_area}",
                    decoration.frame,
                );
            }
            UiTextPaintDecorationKind::CompositionHighlight => {
                let frame_area = decoration.frame.width.ceil().max(1.0) as usize
                    * decoration.frame.height.ceil().max(1.0) as usize;
                assert!(
                    changed >= (frame_area / 4).max(12),
                    "composition highlight must cover its real framebuffer frame: frame={:?}, changed={changed}, area={frame_area}",
                    decoration.frame,
                );
            }
            UiTextPaintDecorationKind::CompositionUnderline => assert_near_color_coverage(
                capture,
                decoration.frame,
                [0x4d, 0x89, 0xff],
                decoration.frame.width.ceil().max(1.0) as usize / 2,
                "composition underline",
            ),
            UiTextPaintDecorationKind::Caret => assert_near_color_coverage(
                capture,
                decoration.frame,
                [0xe8, 0xee, 0xf7],
                decoration.frame.height.ceil().max(1.0) as usize / 2,
                "caret",
            ),
            _ => unreachable!("editable decoration filter only admits four kinds"),
        }
    }
}

pub(super) fn count_near_color_coverage(
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    frame: zircon_runtime_interface::ui::layout::UiFrame,
    expected_linear: [u8; 3],
) -> usize {
    let expected = expected_linear.map(linear_channel_to_srgb_byte);
    let width = capture.width as usize;
    let height = capture.height as usize;
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let mut matching = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = (y * width + x) * 4;
            if capture.rgba[index..index + 3]
                .iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= 18)
            {
                matching += 1;
            }
        }
    }
    matching
}

fn assert_near_color_coverage(
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    frame: zircon_runtime_interface::ui::layout::UiFrame,
    expected_linear: [u8; 3],
    minimum: usize,
    label: &str,
) {
    let matching = count_near_color_coverage(capture, frame, expected_linear);
    assert!(
        matching >= minimum.max(1),
        "{label} must expose its own framebuffer color: frame={frame:?}, matching={matching}, minimum={}",
        minimum.max(1),
    );
}

fn linear_channel_to_srgb_byte(channel: u8) -> u8 {
    let linear = channel as f32 / 255.0;
    let srgb = if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[test]
fn framebuffer_decoration_expectations_encode_linear_vertex_colors_as_srgb() {
    assert_eq!(
        [0x4d, 0x89, 0xff].map(linear_channel_to_srgb_byte),
        [149, 194, 255]
    );
    assert_eq!(
        [0xe8, 0xee, 0xf7].map(linear_channel_to_srgb_byte),
        [245, 247, 251]
    );
}

#[cfg(target_os = "windows")]
pub(super) fn assert_variable_font_instance_pixels(
    samples: &[UiRenderCommand],
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    background: &zircon_runtime::core::framework::render::CapturedFrame,
) {
    let narrow = sample_by_node(samples, 125);
    let wide = sample_by_node(samples, 127);
    assert_eq!(narrow.text, wide.text);
    assert_eq!(
        narrow.style.font.as_deref(),
        Some(super::product_project_fixture::VARIABLE_FONT_ASSET_URI)
    );
    assert_eq!(narrow.style.font, wide.style.font);
    assert_ne!(narrow.style.font_family, wide.style.font_family);
    assert_eq!(
        narrow.style.text_render_mode,
        zircon_runtime_interface::ui::surface::UiTextRenderMode::Sdf
    );
    assert_eq!(narrow.style.text_render_mode, wide.style.text_render_mode);

    let narrow_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        narrow.frame,
        10,
    )
    .expect("narrow variable-font instance must produce real SDF framebuffer pixels");
    let wide_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        wide.frame,
        10,
    )
    .expect("wide variable-font instance must produce real SDF framebuffer pixels");
    assert!(narrow_bounds.4 > 96 && wide_bounds.4 > 96);
    let narrow_width = narrow_bounds.2 - narrow_bounds.0 + 1;
    let wide_width = wide_bounds.2 - wide_bounds.0 + 1;
    assert!(
        wide_width >= narrow_width + 24,
        "one physical face must retain distinct wdth instance pixels through SDF/atlas rendering; narrow={narrow_bounds:?}, wide={wide_bounds:?}"
    );
    let instance_delta = super::count_relative_pixel_differences(
        &capture.rgba,
        capture.width,
        capture.height,
        narrow.frame,
        wide.frame,
        10,
    );
    assert!(
        instance_delta > 64,
        "narrow/wide variable instances must not collapse to one framebuffer result; delta={instance_delta}"
    );
}

pub(super) fn assert_native_sdf_parity_layout(samples: &[UiRenderCommand]) {
    let native = sample_by_node(samples, 121);
    let sdf = sample_by_node(samples, 122);
    let native_layout = native
        .text_layout
        .as_ref()
        .expect("Native parity command resolved layout");
    let sdf_layout = sdf
        .text_layout
        .as_ref()
        .expect("SDF parity command resolved layout");

    assert!(native_layout.lines.len() >= 2);
    assert_eq!(native_layout.lines.len(), sdf_layout.lines.len());
    for (native_line, sdf_line) in native_layout.lines.iter().zip(&sdf_layout.lines) {
        assert_eq!(native_line.text, sdf_line.text);
        assert_eq!(native_line.source_range, sdf_line.source_range);
        assert_eq!(native_line.glyph_advances, sdf_line.glyph_advances);
        assert_close(
            native_line.frame.x - native.frame.x,
            sdf_line.frame.x - sdf.frame.x,
            "Native/SDF relative line x",
        );
        assert_close(
            native_line.frame.y - native.frame.y,
            sdf_line.frame.y - sdf.frame.y,
            "Native/SDF relative line y",
        );
        assert_close(
            native_line.frame.width,
            sdf_line.frame.width,
            "Native/SDF line width",
        );
        assert_close(
            native_line.frame.height,
            sdf_line.frame.height,
            "Native/SDF line height",
        );
    }
}

pub(super) fn assert_native_sdf_parity_pixels(
    samples: &[UiRenderCommand],
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    background: &zircon_runtime::core::framework::render::CapturedFrame,
) {
    let native = sample_by_node(samples, 121);
    let sdf = sample_by_node(samples, 122);
    let native_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        native.frame,
        10,
    )
    .expect("Native parity region must contain framebuffer glyph pixels");
    let sdf_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        sdf.frame,
        10,
    )
    .expect("SDF parity region must contain framebuffer glyph pixels");
    let native_size = (
        native_bounds.2 - native_bounds.0 + 1,
        native_bounds.3 - native_bounds.1 + 1,
    );
    let sdf_size = (
        sdf_bounds.2 - sdf_bounds.0 + 1,
        sdf_bounds.3 - sdf_bounds.1 + 1,
    );
    assert!(
        native_size.0.abs_diff(sdf_size.0) <= 24,
        "Native/SDF framebuffer width parity: native={native_size:?}, sdf={sdf_size:?}"
    );
    assert!(
        native_size.1.abs_diff(sdf_size.1) <= 12,
        "Native/SDF framebuffer height parity: native={native_size:?}, sdf={sdf_size:?}"
    );
}

fn sample_by_node(samples: &[UiRenderCommand], node_id: u64) -> &UiRenderCommand {
    samples
        .iter()
        .find(|command| command.node_id == UiNodeId::new(node_id))
        .unwrap_or_else(|| panic!("missing product proof node {node_id}"))
}

pub(super) fn assert_vertical_bbcode_paragraph_layout(samples: &[UiRenderCommand]) {
    let command = samples
        .iter()
        .find(|command| command.node_id == UiNodeId::new(120))
        .expect("VerticalRl paragraph product proof command");
    let layout = command
        .text_layout
        .as_ref()
        .expect("VerticalRl paragraph product proof must consume a resolved layout");
    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert!(layout.lines.len() >= 2);

    let centered = layout
        .lines
        .iter()
        .find(|line| line.text.contains("首列缩进居中"))
        .expect("centered VerticalRl paragraph column");
    let ended = layout
        .lines
        .iter()
        .find(|line| line.text.contains("末端对齐验证"))
        .expect("end-aligned VerticalRl paragraph column");
    assert!(
        centered.frame.y >= command.frame.y + 28.0 - FRAME_EPSILON,
        "VerticalRl paragraph indent must map to the physical y inline axis"
    );
    assert!(
        centered.frame.bottom() < command.frame.bottom() - FRAME_EPSILON,
        "centered VerticalRl paragraph must retain space on the physical bottom edge"
    );
    let inline_index = centered
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("centered VerticalRl paragraph must retain the inline image placeholder");
    assert!(centered.runs.iter().any(|run| run.text == "\u{fffc}"));
    assert_close(
        centered.glyph_advances[inline_index],
        16.0,
        "VerticalRl paragraph inline image height must remain the main-axis advance",
    );
    assert_close(
        ended.frame.bottom(),
        command.frame.bottom(),
        "VerticalRl paragraph end alignment must target the physical bottom edge",
    );
    assert!(
        centered.frame.x > ended.frame.x,
        "VerticalRl paragraph columns must continue from right to left"
    );
}

pub(super) fn assert_bbcode_block_layouts(samples: &[UiRenderCommand]) {
    let list_layout = samples[15]
        .text_layout
        .as_ref()
        .expect("BBCode list product proof must consume resolved layout");
    assert!(list_layout.lines.iter().any(|line| line.text.contains('◆')));
    assert!(list_layout
        .lines
        .iter()
        .any(|line| line.text.contains('🚀')));
    assert!(list_layout
        .lines
        .iter()
        .any(|line| line.text.contains('\u{fffc}')));
    assert!(
        list_layout
            .lines
            .windows(2)
            .any(|lines| lines[1].frame.x > lines[0].frame.x),
        "nested or hanging BBCode list lines must show a measured inset"
    );

    let paragraph_layout = samples[16]
        .text_layout
        .as_ref()
        .expect("BBCode paragraph product proof must consume resolved layout");
    assert!(
        paragraph_layout
            .lines
            .iter()
            .any(|line| line.frame.x > samples[16].frame.x + 1.0),
        "BBCode paragraph/indent product proof must inset rendered lines"
    );
}

pub(super) fn assert_bbcode_table_layout(samples: &[UiRenderCommand]) {
    let command = &samples[BBCODE_TABLE_SAMPLE_INDEX];
    let layout = command
        .text_layout
        .as_ref()
        .expect("BBCode table product proof must consume a resolved layout");
    let heading = find_line(layout, "BBCode V2 CELL BOX");
    let span_owner = find_line(layout, "Span owner");
    let colspan = find_line(layout, "colspan shares");
    let wrapped = find_line(layout, "wrapped detail");
    let product = layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("the WGPU span cell must retain its inline icon placeholder");
    assert_eq!(
        layout.boxes.len(),
        5,
        "every authored cell must emit one box"
    );
    assert!(layout.boxes.iter().all(|text_box| {
        text_box.background_color.is_some()
            && text_box.border_color.is_some()
            && (text_box.border_width - 1.0).abs() <= FRAME_EPSILON
    }));
    let heading_box = layout
        .boxes
        .iter()
        .find(|text_box| {
            text_box.range.start <= heading.source_range.start
                && heading.source_range.end <= text_box.range.end
        })
        .expect("merged heading cell box");
    let span_owner_box = find_box_for_line(layout, span_owner);
    let colspan_box = find_box_for_line(layout, colspan);
    let wrapped_box = find_box_for_line(layout, wrapped);
    let product_box = find_box_for_line(layout, product);
    assert!(heading.frame.x >= heading_box.frame.x + 16.0 - FRAME_EPSILON);
    assert!(heading.frame.y >= heading_box.frame.y + 10.0 - FRAME_EPSILON);

    let paint = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = paint.payload else {
        panic!("BBCode table product proof must project a text paint payload");
    };
    assert_eq!(
        text.decorations
            .iter()
            .filter(|decoration| {
                decoration.kind == UiTextPaintDecorationKind::TableCellBackground
            })
            .count(),
        5
    );
    assert_eq!(
        text.decorations
            .iter()
            .filter(|decoration| decoration.kind == UiTextPaintDecorationKind::TableCellBorder)
            .count(),
        5
    );

    assert_close(
        span_owner_box.frame.y,
        colspan_box.frame.y,
        "first detail row box y",
    );
    assert_close(
        wrapped_box.frame.y,
        product_box.frame.y,
        "second detail row box y",
    );
    assert_close(
        colspan_box.frame.x,
        wrapped_box.frame.x,
        "second shared column box x",
    );
    assert!(
        heading_box.frame.y < span_owner_box.frame.y
            && wrapped_box.frame.y > span_owner_box.frame.y,
        "the merged heading and detail rows must advance from top to bottom"
    );
    assert!(
        span_owner_box.frame.x < colspan_box.frame.x && product_box.frame.x > wrapped_box.frame.x,
        "rowspan and ordinary detail cells must occupy distinct columns"
    );
    assert!(
        heading_box.frame.right() > product_box.frame.x,
        "the one-line merged heading must visibly cross the third-column origin"
    );
    assert!(
        colspan_box.frame.right() > product_box.frame.x,
        "the colspan detail must use width across the third-column origin"
    );
    assert!(
        layout.boxes.iter().all(|text_box| {
            (text_box.frame.y - wrapped_box.frame.y).abs() > FRAME_EPSILON
                || text_box.frame.x > span_owner_box.frame.x + FRAME_EPSILON
        }),
        "the rowspan owner must reserve its first-column slot on the following row"
    );
    let wrapped_line_count = layout
        .lines
        .iter()
        .filter(|line| {
            line.frame.y + FRAME_EPSILON >= wrapped.frame.y
                && (line.frame.x - wrapped.frame.x).abs() <= FRAME_EPSILON
        })
        .count();
    assert!(
        wrapped_line_count >= 2,
        "the ordinary detail cell must wrap inside its final track; lines={wrapped_line_count}"
    );
    assert!(
        layout
            .lines
            .iter()
            .any(|line| line.text.contains('\u{fffc}')),
        "the table product proof must retain the inline icon placeholder"
    );
}

pub(super) fn assert_vertical_bbcode_table_layout(samples: &[UiRenderCommand]) {
    let command = &samples[VERTICAL_BBCODE_TABLE_SAMPLE_INDEX];
    let layout = command
        .text_layout
        .as_ref()
        .expect("VerticalRl BBCode table proof must consume a resolved layout");
    assert_eq!(
        layout.writing_mode,
        zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl
    );
    assert_eq!(
        layout.boxes.len(),
        5,
        "every vertical cell must emit one box"
    );
    assert!(layout.boxes.iter().all(|text_box| {
        text_box.background_color.is_some()
            && text_box.border_color.is_some()
            && (text_box.border_width - 1.0).abs() <= FRAME_EPSILON
    }));

    let heading = find_line(layout, "VERTICAL");
    let span = find_line(layout, "SPAN");
    let first = find_line(layout, "A1");
    let second = find_line(layout, "B2");
    let tail = find_line(layout, "RTL AXES");
    let heading_box = find_box_for_line(layout, heading);
    let span_box = find_box_for_line(layout, span);
    let first_box = find_box_for_line(layout, first);
    let second_box = find_box_for_line(layout, second);
    let tail_box = find_box_for_line(layout, tail);

    assert!(
        span_box.frame.y < first_box.frame.y,
        "later logical columns must advance down the physical y axis"
    );
    assert_close(
        first_box.frame.y,
        second_box.frame.y,
        "the same logical column must retain its physical y origin",
    );
    assert!(
        heading_box.frame.x > first_box.frame.x
            && first_box.frame.x > second_box.frame.x
            && second_box.frame.x > tail_box.frame.x,
        "logical rows must advance from physical right to left"
    );
    assert!(
        heading_box.frame.height > first_box.frame.height,
        "vertical colspan must increase physical height"
    );
    assert!(
        span_box.frame.width > first_box.frame.width,
        "vertical rowspan must increase physical width"
    );
    assert_close(
        span_box.frame.x,
        second_box.frame.x,
        "rowspan must extend toward the physical left",
    );
    assert!(
        heading.frame.y >= heading_box.frame.y + 8.0 - FRAME_EPSILON,
        "authored physical top padding must survive VerticalRl arrangement"
    );
    assert!(
        heading_box.frame.right() - heading.frame.right() >= 10.0 - FRAME_EPSILON,
        "authored physical right padding must survive VerticalRl arrangement"
    );
    assert!(
        layout.lines.iter().any(|line| {
            line.text.contains("VERTICAL")
                || line.text.contains("SPAN")
                || line.text.contains("RTL AXES")
        }),
        "the vertical product proof must retain a Latin shaped run"
    );

    let paint = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = paint.payload else {
        panic!("VerticalRl BBCode table proof must project a text paint payload");
    };
    assert_eq!(
        text.decorations
            .iter()
            .filter(|decoration| {
                decoration.kind == UiTextPaintDecorationKind::TableCellBackground
            })
            .count(),
        5
    );
    assert_eq!(
        text.decorations
            .iter()
            .filter(|decoration| decoration.kind == UiTextPaintDecorationKind::TableCellBorder)
            .count(),
        5
    );
}

fn find_box_for_line<'a>(
    layout: &'a zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    line: &zircon_runtime_interface::ui::surface::UiResolvedTextLine,
) -> &'a zircon_runtime_interface::ui::surface::UiResolvedTextBox {
    layout
        .boxes
        .iter()
        .find(|text_box| {
            text_box.range.start <= line.source_range.start
                && line.source_range.end <= text_box.range.end
        })
        .unwrap_or_else(|| panic!("expected a cell box containing line {:?}", line.text))
}

fn find_line<'a>(
    layout: &'a zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    text: &str,
) -> &'a zircon_runtime_interface::ui::surface::UiResolvedTextLine {
    layout
        .lines
        .iter()
        .find(|line| line.text.contains(text))
        .unwrap_or_else(|| {
            let lines = layout
                .lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>();
            panic!("BBCode table product proof must contain {text:?}; lines={lines:?}")
        })
}

fn assert_close(lhs: f32, rhs: f32, label: &str) {
    assert!(
        (lhs - rhs).abs() <= FRAME_EPSILON,
        "{label} must match: lhs={lhs}, rhs={rhs}"
    );
}
