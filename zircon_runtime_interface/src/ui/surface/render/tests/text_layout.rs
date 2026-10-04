use std::sync::Arc;

use super::{
    UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle, UiTextDirection,
    UiTextRange,
};
use crate::ui::layout::UiFrame;

#[test]
fn artifact_equality_uses_the_runtime_owner_identity() {
    let first = UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(1_u32),
        ("compiled-rich", 7_u64),
    );
    let same_identity = UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(2_u32),
        ("compiled-rich", 7_u64),
    );
    let different_identity = UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(3_u32),
        ("compiled-rich", 8_u64),
    );
    let different_kind = UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(1_u64),
        ("compiled-rich", 7_u64),
    );

    assert_eq!(first, same_identity);
    assert_ne!(first, different_identity);
    assert_ne!(first, different_kind);
}

#[test]
fn resolved_layout_equality_observes_runtime_artifact_identity() {
    let first = UiResolvedTextLayout {
        rich_text_artifact: Some(
            UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
                Arc::new(1_u32),
                ("compiled-rich", 7_u64),
            ),
        ),
        ..UiResolvedTextLayout::default()
    };
    let changed_artifact = UiResolvedTextLayout {
        rich_text_artifact: Some(
            UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
                Arc::new(2_u32),
                ("compiled-rich", 8_u64),
            ),
        ),
        ..first.clone()
    };

    assert_ne!(first, changed_artifact);
}

#[test]
fn resolved_line_serde_preserves_content_and_placement_geometry() {
    let line = UiResolvedTextLine {
        text: "right".to_string(),
        frame: UiFrame::new(70.0, 8.0, 30.0, 12.0),
        placement_frame: UiFrame::new(0.0, 8.0, 100.0, 12.0),
        source_range: UiTextRange { start: 0, end: 5 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 30.0,
        glyph_advances: vec![6.0; 5],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: Vec::new(),
        ellipsized: false,
    };

    let encoded = serde_json::to_value(&line).expect("serialize resolved line geometry");
    let decoded: UiResolvedTextLine =
        serde_json::from_value(encoded.clone()).expect("deserialize resolved line geometry");
    assert_eq!(decoded, line);

    let mut translated = decoded;
    translated.translate(5.0, -3.0);
    assert_eq!(translated.frame, UiFrame::new(75.0, 5.0, 30.0, 12.0));
    assert_eq!(
        translated.placement_frame,
        UiFrame::new(5.0, 5.0, 100.0, 12.0)
    );

    let mut missing_placement = encoded;
    missing_placement
        .as_object_mut()
        .expect("resolved line object")
        .remove("placement_frame");
    assert!(serde_json::from_value::<UiResolvedTextLine>(missing_placement).is_err());
}
