use crate::core::framework::text::TextDirection;
use crate::text::{SharedTextLayoutSession, TextRange, text_style};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, test_style};

#[test]
fn plain_physical_lines_publish_their_actual_shaped_baselines() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let text = "Hg\n\u{4e16}\u{754c}";
    let layout = layout_text(text, &style, UiFrame::new(0.0, 0.0, 200.0, 40.0), None);

    let mut provider = SharedTextLayoutSession::new();
    let shaped = provider
        .shape_horizontal_range(
            text,
            &text_style(&style),
            TextDirection::Auto,
            TextRange {
                start: 0,
                end: text.len(),
            },
        )
        .into_result()
        .expect("the packaged Fira Mono and Noto Sans CJK faces must shape");

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(shaped.lines.len(), 2);
    assert!(
        shaped
            .lines
            .iter()
            .all(|line| line.baseline.is_finite() && line.baseline >= 0.0)
    );
    assert!(
        (layout.lines[0].baseline - shaped.lines[0].baseline).abs() < 0.01,
        "first physical line must preserve its shaped baseline"
    );
    assert!(
        (layout.lines[1].baseline - shaped.lines[1].baseline).abs() < 0.01,
        "second physical line must not reuse the first line's metrics sample"
    );
}

#[test]
fn wrapped_physical_lines_keep_their_selected_face_metrics_and_do_not_overlap() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    let text = "Hg\u{4e16}\u{754c}";
    let mut width_provider = SharedTextLayoutSession::new();
    let latin_width = width_provider
        .shape_horizontal_range(
            "Hg",
            &text_style(&style),
            TextDirection::Auto,
            TextRange { start: 0, end: 2 },
        )
        .into_result()
        .expect("the packaged Latin face must shape")
        .lines[0]
        .measured_width;
    let layout = layout_text(
        text,
        &style,
        UiFrame::new(0.0, 0.0, latin_width + 0.1, 200.0),
        None,
    );

    assert!(layout.lines.len() >= 2, "fixture must create a soft wrap");
    let mut line_provider = SharedTextLayoutSession::new();
    for line in &layout.lines {
        let shaped = line_provider
            .shape_horizontal_range(
                &line.text,
                &text_style(&style),
                TextDirection::Auto,
                TextRange {
                    start: 0,
                    end: line.text.len(),
                },
            )
            .into_result()
            .expect("each resolved physical line must have an actual direct shape");
        let expected = &shaped.lines[0];
        assert!(
            (line.baseline - expected.baseline).abs() < 0.01,
            "line={:?}",
            line.text
        );
        assert!(
            (line.frame.height - expected.line_height).abs() < 0.01,
            "line={:?}",
            line.text
        );
    }
    assert!(
        layout.lines.windows(2).all(|lines| {
            (lines[0].frame.y + lines[0].frame.height - lines[1].frame.y).abs() < 0.01
        }),
        "physical line frames must advance by the prior line's actual height: {:#?}",
        layout.lines
    );
}
