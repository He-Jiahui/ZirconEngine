use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiPaintPayload, UiRenderCommand, UiTextPaintDecorationKind, UiTextWritingMode},
};

const BBCODE_TABLE_SAMPLE_INDEX: usize = 18;
const VERTICAL_BBCODE_TABLE_SAMPLE_INDEX: usize = 19;
const FRAME_EPSILON: f32 = 0.01;

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

pub(super) fn assert_msdf_sharp_corner_pixels(
    samples: &[UiRenderCommand],
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    background: &zircon_runtime::core::framework::render::CapturedFrame,
) {
    let sdf = sample_by_node(samples, 107);
    let msdf = sample_by_node(samples, 123);
    assert_eq!(
        sdf.style.text_render_mode,
        zircon_runtime_interface::ui::surface::UiTextRenderMode::Sdf
    );
    assert_eq!(
        msdf.style.text_render_mode,
        zircon_runtime_interface::ui::surface::UiTextRenderMode::Msdf
    );
    let sdf_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        sdf.frame,
        10,
    )
    .expect("SDF sharp-corner comparison must contain real framebuffer pixels");
    let msdf_bounds = super::changed_pixel_bounds_in_frame(
        &capture.rgba,
        &background.rgba,
        capture.width,
        capture.height,
        msdf.frame,
        10,
    )
    .expect("MSDF sharp-corner comparison must contain real framebuffer pixels");
    assert!(sdf_bounds.4 > 64 && msdf_bounds.4 > 64);
    let decode_delta = super::count_relative_pixel_differences(
        &capture.rgba,
        capture.width,
        capture.height,
        sdf.frame,
        msdf.frame,
        6,
    );
    assert!(
        decode_delta > 32,
        "SDF and MSDF must reach distinct real framebuffer decode paths; delta={decode_delta}"
    );
    let sdf_apex = sharp_a_apex_occupancy(sdf.frame, capture, background);
    let msdf_apex = sharp_a_apex_occupancy(msdf.frame, capture, background);
    assert!(
        msdf_apex > sdf_apex,
        "MSDF must retain more high-contrast apex samples than SDF for the side-by-side A glyph; sdf={sdf_apex}, msdf={msdf_apex}"
    );
}

fn sharp_a_apex_occupancy(
    frame: zircon_runtime_interface::ui::layout::UiFrame,
    capture: &zircon_runtime::core::framework::render::CapturedFrame,
    background: &zircon_runtime::core::framework::render::CapturedFrame,
) -> usize {
    const APEX_WIDTH: usize = 22;
    const APEX_ROWS: usize = 4;
    const CONTRAST_THRESHOLD: u8 = 30;
    let width = capture.width as usize;
    let height = capture.height as usize;
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let mut changed = Vec::new();
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..left.saturating_add(APEX_WIDTH).min(width) {
            let index = (y * width + x) * 4;
            let delta = capture.rgba[index..index + 4]
                .iter()
                .zip(&background.rgba[index..index + 4])
                .map(|(sample, baseline)| sample.abs_diff(*baseline))
                .max()
                .unwrap_or(0);
            if delta >= CONTRAST_THRESHOLD {
                changed.push((x, y));
            }
        }
    }
    let apex_top = changed
        .iter()
        .map(|(_, y)| *y)
        .min()
        .expect("sharp-corner A sample must have high-contrast framebuffer pixels");
    changed
        .iter()
        .filter(|(_, y)| *y < apex_top.saturating_add(APEX_ROWS))
        .count()
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
