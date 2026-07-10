use super::*;

#[test]
fn text_rich_markdown_ui_adapter_uses_stripped_text_ranges() {
    let parsed = parse_source_text("before **bold** after", true);

    assert_eq!(parsed.text, "before bold after");
    assert_eq!(parsed.runs.len(), 3);
    assert_eq!(
        parsed.runs[0].source_range,
        UiTextRange { start: 0, end: 7 }
    );
    assert_eq!(parsed.runs[1].kind, UiTextRunKind::Strong);
    assert_eq!(parsed.runs[1].text, "bold");
    assert_eq!(
        parsed.runs[1].source_range,
        UiTextRange { start: 7, end: 11 }
    );
    assert_eq!(
        parsed.runs[2].source_range,
        UiTextRange { start: 11, end: 17 }
    );
}
