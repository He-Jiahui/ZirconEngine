use crate::core::framework::text::TextWritingMode;
use crate::text::{RichTextFormat, TextAlign, TextFrame, TextRange, TextSize, TextWrap};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiRichTextFormat, UiTextAlign, UiTextRange, UiTextWrap, UiTextWritingMode},
};

#[test]
fn text_transport_root_stays_structural() {
    let root = include_str!("mod.rs");

    assert!(root.contains("mod conversion;"));
    assert!(root.contains("mod tests;"));
    for forbidden in ["impl From<", "#[test]"] {
        assert!(
            !root.contains(forbidden),
            "text transport root retained `{forbidden}`"
        );
    }
}

#[test]
fn rich_text_format_transport_round_trips_every_variant() {
    let cases = [
        (UiRichTextFormat::Plain, RichTextFormat::Plain),
        (
            UiRichTextFormat::MarkdownInlineV1,
            RichTextFormat::MarkdownInlineV1,
        ),
        (UiRichTextFormat::BbCodeV1, RichTextFormat::BbCodeV1),
        (UiRichTextFormat::HtmlSubsetV1, RichTextFormat::HtmlSubsetV1),
    ];

    for (transport, neutral) in cases {
        assert_eq!(RichTextFormat::from(transport), neutral);
        assert_eq!(UiRichTextFormat::from(neutral), transport);
    }
}

#[test]
fn text_writing_mode_transport_round_trips_every_variant() {
    let cases = [
        (
            UiTextWritingMode::HorizontalTb,
            TextWritingMode::HorizontalTopToBottom,
        ),
        (
            UiTextWritingMode::VerticalRl,
            TextWritingMode::VerticalRightToLeft,
        ),
    ];

    for (transport, neutral) in cases {
        assert_eq!(TextWritingMode::from(transport), neutral);
        assert_eq!(UiTextWritingMode::from(neutral), transport);
    }
}

#[test]
fn text_alignment_and_wrap_transport_round_trip_every_variant() {
    let align_cases = [
        (UiTextAlign::Left, TextAlign::Left),
        (UiTextAlign::Center, TextAlign::Center),
        (UiTextAlign::Right, TextAlign::Right),
        (UiTextAlign::Start, TextAlign::Start),
        (UiTextAlign::End, TextAlign::End),
        (UiTextAlign::Justify, TextAlign::Justify),
    ];
    for (transport, neutral) in align_cases {
        assert_eq!(TextAlign::from(transport), neutral);
        assert_eq!(UiTextAlign::from(neutral), transport);
    }

    let wrap_cases = [
        (UiTextWrap::None, TextWrap::None),
        (UiTextWrap::Word, TextWrap::Word),
        (UiTextWrap::WordSmart, TextWrap::WordSmart),
        (UiTextWrap::Glyph, TextWrap::Glyph),
    ];
    for (transport, neutral) in wrap_cases {
        assert_eq!(TextWrap::from(transport), neutral);
        assert_eq!(UiTextWrap::from(neutral), transport);
    }
}

#[test]
fn text_geometry_transport_preserves_components() {
    let transport = UiFrame::new(1.25, -2.5, 320.0, 48.5);
    let neutral = TextFrame::from(transport);

    assert_eq!(neutral, TextFrame::new(1.25, -2.5, 320.0, 48.5));
    assert_eq!(UiFrame::from(neutral), transport);

    let transport_range = UiTextRange { start: 3, end: 17 };
    let neutral_range = TextRange::from(transport_range);
    assert_eq!(neutral_range, TextRange { start: 3, end: 17 });
    assert_eq!(UiTextRange::from(neutral_range), transport_range);

    assert_eq!(
        UiSize::from(TextSize::new(640.0, 360.0)),
        UiSize::new(640.0, 360.0)
    );
}
