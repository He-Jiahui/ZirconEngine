use super::sanitized_advance;

#[test]
fn sanitized_advance_rejects_negative_and_non_finite_geometry() {
    assert_eq!(sanitized_advance(12.5), 12.5);
    assert_eq!(sanitized_advance(-4.0), 0.0);
    assert_eq!(sanitized_advance(f32::NAN), 0.0);
    assert_eq!(sanitized_advance(f32::INFINITY), 0.0);
    assert_eq!(sanitized_advance(f32::NEG_INFINITY), 0.0);
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum ShapeArtifactV1<'a> {
    Unavailable,
    Canonical(&'a super::UiShapedText),
}

fn canonical_wire_fixture() -> super::UiShapedText {
    use super::*;
    UiShapedText {
        source_text: "中x".into(),
        source_range: UiTextRange { start: 0, end: 4 },
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 9.0,
        measured_height: 14.0,
        writing_mode: UiTextWritingMode::HorizontalTb,
        render_mode: UiTextRenderMode::Native,
        font_key: None,
        atlas_resource: None,
        ellipsis_range: None,
        lines: vec![UiShapedTextLine {
            text: "中x".into(),
            frame: UiFrame::new(0.0, 0.0, 9.0, 14.0),
            source_range: UiTextRange { start: 0, end: 4 },
            visual_range: UiTextRange { start: 0, end: 4 },
            measured_width: 9.0,
            baseline: 11.0,
            direction: UiTextDirection::LeftToRight,
            ellipsized: false,
            glyphs: Vec::new(),
            clusters: vec![UiShapedTextCluster {
                kind: UiTextRunKind::Plain,
                text: "中x".into(),
                source_range: UiTextRange { start: 0, end: 4 },
                visual_range: UiTextRange { start: 0, end: 4 },
                direction: UiTextDirection::LeftToRight,
            }],
        }],
    }
}

#[test]
fn shape_artifact_v1_json_and_positional_bytes_preserve_canonical_payload() {
    use super::{UiTextPaint, UiTextShapeArtifact};
    use crate::serialization::PositionalEncoding;
    let shaped = canonical_wire_fixture();
    let artifact = UiTextPaint::from_shaped_text(shaped.clone(), None).shaped;
    let legacy = ShapeArtifactV1::Canonical(&shaped);
    let expected_json = serde_json::to_vec(&legacy).unwrap();
    assert_eq!(serde_json::to_vec(&artifact).unwrap(), expected_json);
    let decoded: UiTextShapeArtifact = serde_json::from_slice(&expected_json).unwrap();
    assert_eq!(decoded, artifact);
    assert_eq!(decoded.canonical(), Some(&shaped));
    assert_eq!(
        serde_json::to_string(&UiTextShapeArtifact::Unavailable).unwrap(),
        r#""unavailable""#
    );
    for encoding in [
        PositionalEncoding::FixedLittleEndian,
        PositionalEncoding::VariableLittleEndian,
    ] {
        let expected = encoding.encode(&legacy).unwrap();
        let actual = encoding.encode(&artifact).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual[0], 1);
        assert_eq!(
            encoding.decode::<UiTextShapeArtifact>(&expected).unwrap(),
            artifact
        );
        let absent = encoding.encode(&ShapeArtifactV1::Unavailable).unwrap();
        assert_eq!(
            encoding.encode(&UiTextShapeArtifact::Unavailable).unwrap(),
            absent
        );
        assert!(absent.iter().all(|byte| *byte == 0));
        assert_eq!(
            encoding.decode::<UiTextShapeArtifact>(&absent).unwrap(),
            UiTextShapeArtifact::Unavailable
        );
    }
}

#[test]
fn shape_artifact_v1_rejects_invalid_truncated_and_trailing_payloads() {
    use super::{UiTextPaint, UiTextShapeArtifact};
    use crate::serialization::{PositionalDecodeError, PositionalEncoding};
    for invalid in [
        r#"{"canonical":null}"#,
        r#"{"canonical":{"source_text":"missing required layout"}}"#,
        r#"{"other":{}}"#,
    ] {
        assert!(serde_json::from_str::<UiTextShapeArtifact>(invalid).is_err());
    }
    let artifact = UiTextPaint::from_shaped_text(canonical_wire_fixture(), None).shaped;
    for encoding in [
        PositionalEncoding::FixedLittleEndian,
        PositionalEncoding::VariableLittleEndian,
    ] {
        let mut bytes = encoding.encode(&artifact).unwrap();
        assert!(encoding
            .decode::<UiTextShapeArtifact>(&bytes[..bytes.len() - 1])
            .is_err());
        bytes.push(0);
        assert!(matches!(
            encoding.decode::<UiTextShapeArtifact>(&bytes),
            Err(PositionalDecodeError::TrailingBytes { remaining: 1 })
        ));
        let mut invalid_tag = encoding.encode(&UiTextShapeArtifact::Unavailable).unwrap();
        invalid_tag[0] = 2;
        assert!(encoding
            .decode::<UiTextShapeArtifact>(&invalid_tag)
            .is_err());
    }
}
