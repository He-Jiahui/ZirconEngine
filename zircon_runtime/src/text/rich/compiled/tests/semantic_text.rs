use super::*;
use crate::core::math::Vec2;
use crate::text::RichInlineWidgetSlotId;

#[test]
fn semantic_text_rejects_an_empty_inline_artifact_range() {
    let parsed = RichParseResult {
        text: "x".into(),
        runs: vec![StyledRun {
            byte_range: (0, 0),
            inline: Some(InlineObjectRef::Widget {
                slot: RichInlineWidgetSlotId::new(1),
                size: Vec2::new(1.0, 1.0),
            }),
            ..StyledRun::default()
        }],
        ..RichParseResult::default()
    };

    assert!(matches!(
        semantic_text_for_inline_runs(&parsed, &[0], 16),
        Err(RichTextParseError::ArtifactSourceRangeInvalid {
            range_kind: "inline semantic placeholder",
            start: 0,
            end: 0,
            source_bytes: 1,
        })
    ));
}
