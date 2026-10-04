use std::mem::size_of;

use super::*;

fn version_from_u64_tuple(version: (u64, u64, u64)) -> TextDataVersion {
    TextDataVersion::new(version.0 as u16, version.1 as u16, version.2 as u16)
}

fn version_from_u8_tuple(version: (u8, u8, u8)) -> TextDataVersion {
    TextDataVersion::new(version.0.into(), version.1.into(), version.2.into())
}

#[test]
fn compiled_snapshot_matches_exported_unicode_provider_versions() {
    let snapshot = compiled_unicode_data_snapshot();

    assert_eq!(
        snapshot.normalization().unicode_data,
        Some(version_from_u8_tuple(
            unicode_normalization::UNICODE_VERSION
        ))
    );
    assert_eq!(
        snapshot.bidi().unicode_data,
        Some(version_from_u64_tuple(unicode_bidi::UNICODE_VERSION))
    );
    assert_eq!(
        snapshot.bidi_mirroring().unicode_data,
        Some(version_from_u8_tuple(
            unicode_bidi_mirroring::UNICODE_VERSION
        ))
    );
    assert_eq!(
        snapshot.script().unicode_data,
        Some(version_from_u64_tuple(unicode_script::UNICODE_VERSION))
    );
    assert_eq!(
        snapshot.grapheme().unicode_data,
        Some(version_from_u64_tuple(
            unicode_segmentation::UNICODE_VERSION
        ))
    );
    assert_eq!(
        snapshot.word().unicode_data,
        Some(version_from_u64_tuple(
            unicode_segmentation::UNICODE_VERSION
        ))
    );
    assert_eq!(
        snapshot.line_break().unicode_data,
        Some(version_from_u8_tuple(unicode_linebreak::UNICODE_VERSION))
    );
    assert_eq!(
        snapshot.emoji().unicode_data,
        Some(version_from_u64_tuple(unicode_properties::UNICODE_VERSION))
    );
    assert_eq!(
        snapshot.general_category().unicode_data,
        Some(version_from_u64_tuple(unicode_properties::UNICODE_VERSION))
    );
}

#[test]
fn compiled_snapshot_keeps_mixed_provider_versions_visible() {
    let snapshot = compiled_unicode_data_snapshot();

    assert_eq!(snapshot.locale().unicode_data, None);
    assert_eq!(
        snapshot.line_break().unicode_data,
        Some(TextDataVersion::new(15, 0, 0))
    );
    assert_eq!(
        snapshot.bidi().unicode_data,
        Some(TextDataVersion::new(16, 0, 0))
    );
    assert_eq!(
        snapshot.vertical_orientation().unicode_data,
        Some(TextDataVersion::new(17, 0, 0))
    );
    assert_ne!(snapshot.id().fingerprint(), 0);
}

#[test]
fn snapshot_schema_tracks_shared_crates_as_separate_capability_roles() {
    let snapshot = compiled_unicode_data_snapshot();

    assert_eq!(snapshot.schema_version(), 4);
    assert_eq!(snapshot.id().generation(), 4);
    assert_eq!(PROVIDERS.len(), 12);
    assert_eq!(snapshot.word(), snapshot.grapheme());
    assert_eq!(snapshot.general_category(), snapshot.emoji());
    assert_eq!(
        snapshot.joining_type().implementation,
        TextDataVersion::new(2, 2, 0)
    );
}

#[test]
fn snapshot_generation_is_part_of_artifact_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);

    assert_ne!(current, next);
    assert_eq!(current.fingerprint(), next.fingerprint());
    assert_eq!(size_of::<UnicodeDataSnapshotId>(), 16);
}
