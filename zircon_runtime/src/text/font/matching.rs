use std::collections::HashSet;

use crate::text::{FontFamilyName, FontStretch, FontStyle, FontWeight};

const FONT_FAMILY_IDENTITY_HASH_DOMAIN: &[u8] = b"zircon-font-family-identity-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontFamilyIdentity([u8; 16]);

impl FontFamilyIdentity {
    pub(super) const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

pub(super) fn font_family_identity(family: &str) -> FontFamilyIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FONT_FAMILY_IDENTITY_HASH_DOMAIN);
    let family = family.trim();
    for byte in family.bytes() {
        hasher.update(&[byte.to_ascii_lowercase()]);
    }
    let mut identity = [0_u8; 16];
    identity.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    FontFamilyIdentity(identity)
}

pub(super) fn dedupe_families(
    families: impl IntoIterator<Item = FontFamilyName>,
) -> Vec<FontFamilyName> {
    let mut identities = HashSet::new();
    let mut result = Vec::new();
    for family in families {
        if family.is_empty() {
            continue;
        }
        if !identities.insert(font_family_identity(family.as_str())) {
            continue;
        }
        result.push(family);
    }
    result
}

pub(super) fn weight_distance(candidate: FontWeight, requested: FontWeight) -> u16 {
    candidate.0.abs_diff(requested.0)
}

pub(super) fn stretch_distance(candidate: FontStretch, requested: FontStretch) -> u16 {
    candidate.0.abs_diff(requested.0)
}

pub(super) fn style_distance(candidate: FontStyle, requested: FontStyle) -> u8 {
    match (candidate, requested) {
        (FontStyle::Normal, FontStyle::Normal) => 0,
        (FontStyle::Italic, FontStyle::Italic) => 0,
        (FontStyle::Oblique(_), FontStyle::Oblique(_)) => 0,
        (FontStyle::Italic, FontStyle::Oblique(_)) | (FontStyle::Oblique(_), FontStyle::Italic) => {
            1
        }
        _ => 2,
    }
}
