use serde::{Deserialize, Serialize};

const UNICODE_DATA_SNAPSHOT_SCHEMA_VERSION: u16 = 4;
const COMPILED_UNICODE_DATA_GENERATION: u64 = 4;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextDataVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl TextDataVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnicodeProviderSnapshot {
    pub implementation: TextDataVersion,
    pub unicode_data: Option<TextDataVersion>,
}

impl UnicodeProviderSnapshot {
    const fn new(implementation: TextDataVersion, unicode_data: Option<TextDataVersion>) -> Self {
        Self {
            implementation,
            unicode_data,
        }
    }
}

/// Compact identity carried by analysis, shaping artifacts, and cache keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnicodeDataSnapshotId {
    generation: u64,
    fingerprint: u64,
}

impl UnicodeDataSnapshotId {
    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn fingerprint(self) -> u64 {
        self.fingerprint
    }

    #[cfg(test)]
    pub(crate) const fn with_generation_for_test(self, generation: u64) -> Self {
        Self {
            generation,
            fingerprint: self.fingerprint,
        }
    }
}

/// Immutable descriptor for every Unicode-aware provider used by Runtime Text.
///
/// Provider implementation versions are part of the fingerprint even when two releases use the
/// same Unicode data. `locale` intentionally has no Unicode/CLDR data version: `icu_locale_core`
/// currently supplies syntax and canonical casing here, not a likely-subtag or CLDR data provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnicodeDataSnapshot {
    id: UnicodeDataSnapshotId,
    schema_version: u16,
    locale: UnicodeProviderSnapshot,
    normalization: UnicodeProviderSnapshot,
    bidi: UnicodeProviderSnapshot,
    bidi_mirroring: UnicodeProviderSnapshot,
    script: UnicodeProviderSnapshot,
    grapheme: UnicodeProviderSnapshot,
    word: UnicodeProviderSnapshot,
    line_break: UnicodeProviderSnapshot,
    emoji: UnicodeProviderSnapshot,
    general_category: UnicodeProviderSnapshot,
    joining_type: UnicodeProviderSnapshot,
    vertical_orientation: UnicodeProviderSnapshot,
}

impl UnicodeDataSnapshot {
    pub const fn id(self) -> UnicodeDataSnapshotId {
        self.id
    }

    pub const fn schema_version(self) -> u16 {
        self.schema_version
    }

    pub const fn locale(self) -> UnicodeProviderSnapshot {
        self.locale
    }

    pub const fn normalization(self) -> UnicodeProviderSnapshot {
        self.normalization
    }

    pub const fn bidi(self) -> UnicodeProviderSnapshot {
        self.bidi
    }

    pub const fn bidi_mirroring(self) -> UnicodeProviderSnapshot {
        self.bidi_mirroring
    }

    pub const fn script(self) -> UnicodeProviderSnapshot {
        self.script
    }

    pub const fn grapheme(self) -> UnicodeProviderSnapshot {
        self.grapheme
    }

    pub const fn word(self) -> UnicodeProviderSnapshot {
        self.word
    }

    pub const fn line_break(self) -> UnicodeProviderSnapshot {
        self.line_break
    }

    pub const fn emoji(self) -> UnicodeProviderSnapshot {
        self.emoji
    }

    pub const fn general_category(self) -> UnicodeProviderSnapshot {
        self.general_category
    }

    pub const fn joining_type(self) -> UnicodeProviderSnapshot {
        self.joining_type
    }

    pub const fn vertical_orientation(self) -> UnicodeProviderSnapshot {
        self.vertical_orientation
    }
}

const LOCALE_PROVIDER: UnicodeProviderSnapshot =
    UnicodeProviderSnapshot::new(TextDataVersion::new(2, 2, 0), None);
const NORMALIZATION_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 1, 25),
    Some(TextDataVersion::new(17, 0, 0)),
);
const BIDI_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 3, 18),
    Some(TextDataVersion::new(16, 0, 0)),
);
const BIDI_MIRRORING_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 4, 0),
    Some(TextDataVersion::new(16, 0, 0)),
);
const SCRIPT_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 5, 8),
    Some(TextDataVersion::new(17, 0, 0)),
);
const GRAPHEME_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(1, 13, 3),
    Some(TextDataVersion::new(17, 0, 0)),
);
// Word and grapheme segmentation are distinct snapshot roles even though the current compiled
// provider implements both from the same unicode-segmentation release and Unicode tables.
const WORD_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(1, 13, 3),
    Some(TextDataVersion::new(17, 0, 0)),
);
const LINE_BREAK_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 1, 5),
    Some(TextDataVersion::new(15, 0, 0)),
);
const EMOJI_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 1, 4),
    Some(TextDataVersion::new(17, 0, 0)),
);
// Emoji and General_Category are distinct capabilities even though unicode-properties owns both.
const GENERAL_CATEGORY_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 1, 4),
    Some(TextDataVersion::new(17, 0, 0)),
);
// icu_properties_data 2.2.0 was generated from ICU 78, whose Unicode baseline is 17.0.
const JOINING_TYPE_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(2, 2, 0),
    Some(TextDataVersion::new(17, 0, 0)),
);
// unicode-vo 0.1.0 documents VerticalOrientation revision 17 in its generated table source.
const VERTICAL_ORIENTATION_PROVIDER: UnicodeProviderSnapshot = UnicodeProviderSnapshot::new(
    TextDataVersion::new(0, 1, 0),
    Some(TextDataVersion::new(17, 0, 0)),
);

const PROVIDERS: [UnicodeProviderSnapshot; 12] = [
    LOCALE_PROVIDER,
    NORMALIZATION_PROVIDER,
    BIDI_PROVIDER,
    BIDI_MIRRORING_PROVIDER,
    SCRIPT_PROVIDER,
    GRAPHEME_PROVIDER,
    WORD_PROVIDER,
    LINE_BREAK_PROVIDER,
    EMOJI_PROVIDER,
    GENERAL_CATEGORY_PROVIDER,
    JOINING_TYPE_PROVIDER,
    VERTICAL_ORIENTATION_PROVIDER,
];

const fn mix(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV_PRIME)
}

const fn mix_version(mut hash: u64, version: TextDataVersion) -> u64 {
    hash = mix(hash, version.major as u64);
    hash = mix(hash, version.minor as u64);
    mix(hash, version.patch as u64)
}

const fn compiled_fingerprint() -> u64 {
    let mut hash = mix(
        FNV_OFFSET_BASIS,
        UNICODE_DATA_SNAPSHOT_SCHEMA_VERSION as u64,
    );
    let mut index = 0;
    while index < PROVIDERS.len() {
        let provider = PROVIDERS[index];
        hash = mix(hash, index as u64);
        hash = mix_version(hash, provider.implementation);
        hash = match provider.unicode_data {
            Some(version) => mix_version(mix(hash, 1), version),
            None => mix(hash, 0),
        };
        index += 1;
    }
    hash
}

const COMPILED_UNICODE_DATA_SNAPSHOT: UnicodeDataSnapshot = UnicodeDataSnapshot {
    id: UnicodeDataSnapshotId {
        generation: COMPILED_UNICODE_DATA_GENERATION,
        fingerprint: compiled_fingerprint(),
    },
    schema_version: UNICODE_DATA_SNAPSHOT_SCHEMA_VERSION,
    locale: LOCALE_PROVIDER,
    normalization: NORMALIZATION_PROVIDER,
    bidi: BIDI_PROVIDER,
    bidi_mirroring: BIDI_MIRRORING_PROVIDER,
    script: SCRIPT_PROVIDER,
    grapheme: GRAPHEME_PROVIDER,
    word: WORD_PROVIDER,
    line_break: LINE_BREAK_PROVIDER,
    emoji: EMOJI_PROVIDER,
    general_category: GENERAL_CATEGORY_PROVIDER,
    joining_type: JOINING_TYPE_PROVIDER,
    vertical_orientation: VERTICAL_ORIENTATION_PROVIDER,
};

pub const fn compiled_unicode_data_snapshot() -> UnicodeDataSnapshot {
    COMPILED_UNICODE_DATA_SNAPSHOT
}

pub const fn compiled_unicode_data_snapshot_id() -> UnicodeDataSnapshotId {
    COMPILED_UNICODE_DATA_SNAPSHOT.id()
}

impl Default for UnicodeDataSnapshotId {
    fn default() -> Self {
        compiled_unicode_data_snapshot_id()
    }
}

#[cfg(test)]
mod tests {
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
}
