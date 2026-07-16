use std::path::Path;

use crate::core::math::UVec2;

use super::super::*;

const TEST_ASSET_GUID: &str = "12345678-90ab-4cde-8f01-234567890abc";

#[test]
fn text_sdf_offline_bake_roundtrip_is_byte_identical() {
    let artifact = fixture_artifact(SdfMode::Sdf);

    let encoded = artifact.encode().expect("fixture should encode");
    let decoded = SdfOfflineArtifact::decode(&encoded).expect("fixture should decode");

    assert_eq!(decoded, artifact);
    assert_eq!(
        decoded.encode().expect("decoded artifact should re-encode"),
        encoded
    );
    assert_eq!(
        decoded.glyph_pixels(7).expect("glyph 7 pixels"),
        [5, 6, 9, 10]
    );
}

#[test]
fn text_sdf_offline_artifact_sorts_glyphs_and_preserves_rgba_rows() {
    let artifact = fixture_artifact(SdfMode::Msdf);

    assert_eq!(
        artifact
            .glyphs()
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![7, 11]
    );
    assert_eq!(
        artifact.glyph_pixels(7).expect("glyph 7 RGBA rows"),
        [5, 5, 5, 5, 6, 6, 6, 6, 9, 9, 9, 9, 10, 10, 10, 10]
    );
    assert_eq!(artifact.identity().params.mode, SdfMode::Msdf);
}

#[test]
fn text_sdf_offline_artifact_rejects_stale_identity_and_corrupt_payload() {
    let artifact = fixture_artifact(SdfMode::Mtsdf);
    let mut stale = artifact.identity().clone();
    stale.face_index = 2;

    assert!(artifact.validate_identity(&stale).is_err());

    let mut corrupt = artifact.encode().expect("fixture should encode");
    *corrupt.last_mut().expect("encoded payload") ^= 0x5a;
    assert!(SdfOfflineArtifact::decode(&corrupt).is_err());
}

#[test]
fn text_sdf_offline_artifact_path_is_versioned_and_identity_stable() {
    let identity = fixture_identity(SdfMode::Msdf);
    let path = sdf_offline_artifact_path(Path::new("library"), &identity);
    let normalized = path.to_string_lossy().replace('\\', "/");

    assert!(normalized.starts_with("library/text/sdf/v1/12345678-90ab-4cde-8f01-234567890abc/"));
    assert!(normalized.contains("/face_0001/"));
    assert!(normalized.ends_with("/msdf_48_8000.zsdf"));
}

#[test]
fn text_sdf_offline_artifact_rejects_invalid_rect_and_trailing_bytes() {
    let invalid_rect = SdfOfflineArtifact::new(
        fixture_identity(SdfMode::Sdf),
        UVec2::new(4, 4),
        vec![SdfOfflinePage {
            page_index: 0,
            pixels: vec![0; 16],
        }],
        vec![fixture_glyph(7, 'A', SdfOfflineRect::new(3, 3, 2, 2))],
    );
    assert!(invalid_rect.is_err());

    let mut trailing = fixture_artifact(SdfMode::Sdf)
        .encode()
        .expect("fixture should encode");
    trailing.push(0);
    assert!(SdfOfflineArtifact::decode(&trailing).is_err());
}

fn fixture_artifact(mode: SdfMode) -> SdfOfflineArtifact {
    let channels = mode.channel_count() as usize;
    let mut page_pixels = Vec::new();
    for sample in 0_u8..16 {
        page_pixels.extend(std::iter::repeat(sample).take(channels));
    }
    SdfOfflineArtifact::new(
        fixture_identity(mode),
        UVec2::new(4, 4),
        vec![SdfOfflinePage {
            page_index: 0,
            pixels: page_pixels,
        }],
        vec![
            fixture_glyph(11, 'M', SdfOfflineRect::new(0, 0, 1, 1)),
            fixture_glyph(7, 'A', SdfOfflineRect::new(1, 1, 2, 2)),
        ],
    )
    .expect("valid fixture")
}

fn fixture_identity(mode: SdfMode) -> SdfOfflineArtifactIdentity {
    SdfOfflineArtifactIdentity {
        asset_guid: TEST_ASSET_GUID.to_string(),
        face_index: 1,
        variation_hash: [0x11; 32],
        source_hash: [0x22; 32],
        params: SdfBakeParams::for_mode(mode),
    }
}

fn fixture_glyph(glyph_id: u32, scalar: char, rect: SdfOfflineRect) -> SdfOfflineGlyph {
    SdfOfflineGlyph {
        glyph_id,
        codepoint: scalar as u32,
        page_index: 0,
        rect,
        metrics: SdfOfflineGlyphMetrics {
            bitmap_left: 1.25,
            bitmap_bottom: -2.5,
            advance: 28.0,
            ascent: 36.0,
        },
    }
}
