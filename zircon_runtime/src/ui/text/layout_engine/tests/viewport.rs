use crate::{text::SharedTextLayoutSession, ui::text::UiTextViewport};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::super::layout_text_with_provider_and_viewport;
use super::test_style;

#[test]
fn huge_plain_text_shapes_and_resolves_only_the_visible_window() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.line_height = 10.0;
    let text = (0..100)
        .map(|index| format!("row-{index:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut provider = SharedTextLayoutSession::new();
    provider.begin_frame(1);

    let layout = layout_text_with_provider_and_viewport(
        &text,
        &style,
        UiFrame::new(0.0, 0.0, 160.0, 1_000.0),
        Some(UiFrame::new(0.0, 121.0, 160.0, 8.0)),
        UiTextViewport::new(121.0, 8.0, 2).expect("finite document viewport"),
        None,
        &mut provider,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "row-010");
    assert_eq!(layout.lines[0].frame.y, 120.0);
    assert_eq!(layout.measured_height, 1_200.0);
    assert!(layout.overflow_clipped);

    let report = provider.cache_report();
    assert!(
        report.insert_count < 20,
        "visible window should avoid shaping all rows"
    );
}
