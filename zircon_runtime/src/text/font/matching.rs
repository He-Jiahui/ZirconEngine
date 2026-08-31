use std::collections::{HashMap, HashSet};

use crate::text::{FontFamilyName, FontStretch, FontStyle, FontWeight};

const FONT_FAMILY_IDENTITY_HASH_DOMAIN: &[u8] = b"zircon-font-family-identity-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontFamilyIdentity([u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FontFamilyCandidateScope {
    OwnerLocalOnly,
    OwnerThenGlobal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScopedFontFamilyCandidate {
    pub(super) family: FontFamilyName,
    pub(super) scope: FontFamilyCandidateScope,
}

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
    let families = families.into_iter();
    let (capacity, _) = families.size_hint();
    let mut identities = HashSet::with_capacity(capacity);
    let mut result = Vec::with_capacity(capacity);
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

pub(super) fn dedupe_scoped_families(
    families: impl IntoIterator<Item = (FontFamilyName, FontFamilyCandidateScope)>,
) -> Vec<ScopedFontFamilyCandidate> {
    let families = families.into_iter();
    let (capacity, _) = families.size_hint();
    let mut index_by_identity = HashMap::<FontFamilyIdentity, usize>::with_capacity(capacity);
    let mut result = Vec::<ScopedFontFamilyCandidate>::with_capacity(capacity);
    for (family, scope) in families {
        if family.is_empty() {
            continue;
        }
        let identity = font_family_identity(family.as_str());
        if let Some(index) = index_by_identity.get(&identity).copied() {
            if scope == FontFamilyCandidateScope::OwnerThenGlobal {
                result[index].scope = scope;
            }
            continue;
        }
        index_by_identity.insert(identity, result.len());
        result.push(ScopedFontFamilyCandidate { family, scope });
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

#[cfg(test)]
#[path = "matching/capacity_tests.rs"]
mod capacity_tests;
