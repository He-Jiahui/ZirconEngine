use crate::ui::text::{resolve_text_layout, UiPreeditSpan, UiTextLayoutRequest};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextRange, UiTextWrap},
};

#[test]
fn text_layout_request_injects_preedit_without_mutating_source() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let preedit = UiPreeditSpan {
        range: UiTextRange { start: 6, end: 6 },
        text: "中文".to_string(),
    };
    let request =
        UiTextLayoutRequest::new("hello ", &style, UiFrame::new(0.0, 0.0, 80.0, 20.0), None)
            .with_preedit(&preedit);

    let resolution = resolve_text_layout(&request);

    assert_eq!(request.text, "hello ");
    assert_eq!(resolution.layout.source_range.end, "hello 中文".len());
    assert_eq!(resolution.layout.lines[0].text, "hello 中文");
    assert!(resolution.first_baseline > 0.0);
    assert!(resolution.first_baseline <= style.line_height);
}
