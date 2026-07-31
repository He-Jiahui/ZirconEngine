use std::path::{Path, PathBuf};

use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetVariationCoord};
use crate::text::{FontFaceDescriptor, FontStyle, FontWeight, VariationCoords};

use super::database::{canonical_source_key, normalized_family_key};
use super::descriptors::{descriptor_from_font_metadata, stretch_from_ttf_width_class};
use super::face_metadata::FontFaceMetadata;

pub(super) struct FontAssetFaceRegistration {
    pub(super) descriptor: FontFaceDescriptor,
    pub(super) metadata: FontFaceMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontAssetSourceKey {
    path: PathBuf,
    source_identity: [u8; 16],
    face_index: u32,
    family: String,
    weight: u16,
    style: FontStyleKey,
    stretch: u16,
    variations: Vec<(u32, u32)>,
}

impl FontAssetSourceKey {
    pub(super) fn from_descriptor(
        source_path: &Path,
        descriptor: &FontFaceDescriptor,
        source_identity: [u8; 16],
    ) -> Self {
        let mut variations: Vec<(u32, u32)> = descriptor
            .variations
            .0
            .iter()
            .map(|(tag, value)| (*tag, value.to_bits()))
            .collect();
        variations.sort_unstable();
        Self {
            path: canonical_source_key(source_path),
            source_identity,
            face_index: descriptor.face_index,
            family: normalized_family_key(descriptor.family.as_str()),
            weight: descriptor.weight.0,
            style: FontStyleKey::from(descriptor.style),
            stretch: descriptor.stretch.0,
            variations,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FontStyleKey {
    Normal,
    Italic,
    Oblique(u32),
}

impl From<FontStyle> for FontStyleKey {
    fn from(style: FontStyle) -> Self {
        match style {
            FontStyle::Normal => Self::Normal,
            FontStyle::Italic => Self::Italic,
            FontStyle::Oblique(angle) => Self::Oblique(angle.to_bits()),
        }
    }
}

pub(super) fn font_asset_faces(
    asset: &FontAsset,
    bytes: &[u8],
    source_path: &Path,
) -> Vec<FontAssetFaceRegistration> {
    let primary_index = primary_family_member_index(asset);
    let mut descriptors = Vec::new();

    if let Some(index) = primary_index {
        descriptors.push(descriptor_from_font_asset_member(
            bytes,
            source_path,
            &asset.family_members[index],
        ));
    } else {
        let metadata = FontFaceMetadata::from_sfnt_bytes(bytes, asset.face_index);
        descriptors.push(FontAssetFaceRegistration {
            descriptor: descriptor_from_font_metadata(
                &metadata,
                asset.family.as_deref(),
                source_path,
                asset.face_index,
            ),
            metadata,
        });
    }

    for (index, member) in asset.family_members.iter().enumerate() {
        if Some(index) == primary_index {
            continue;
        }
        descriptors.push(descriptor_from_font_asset_member(
            bytes,
            source_path,
            member,
        ));
    }

    descriptors
}

fn primary_family_member_index(asset: &FontAsset) -> Option<usize> {
    asset
        .family_members
        .iter()
        .position(|member| {
            if member.face_index != asset.face_index {
                return false;
            }
            match asset.family.as_deref() {
                Some(family) => {
                    normalized_family_key(member.family.as_str()) == normalized_family_key(family)
                }
                None => true,
            }
        })
        .or_else(|| {
            asset
                .family_members
                .iter()
                .position(|member| member.face_index == asset.face_index)
        })
        .or_else(|| (!asset.family_members.is_empty()).then_some(0))
}

fn descriptor_from_font_asset_member(
    bytes: &[u8],
    source_path: &Path,
    member: &FontAssetFamilyMember,
) -> FontAssetFaceRegistration {
    let metadata = FontFaceMetadata::from_sfnt_bytes(bytes, member.face_index);
    let mut descriptor = descriptor_from_font_metadata(
        &metadata,
        Some(member.family.as_str()),
        source_path,
        member.face_index,
    );
    if let Some(weight) = member.weight {
        descriptor.weight = FontWeight::clamped(weight);
    }
    if let Some(width_class) = member.width_class {
        descriptor.stretch = stretch_from_ttf_width_class(width_class);
    }
    if let Some(style) = member.style {
        descriptor.style = style_from_font_asset(style);
    }
    descriptor.variations = variation_coords_from_font_asset(&member.variations);
    FontAssetFaceRegistration {
        descriptor,
        metadata,
    }
}

fn style_from_font_asset(style: FontAssetFaceStyle) -> FontStyle {
    match style {
        FontAssetFaceStyle::Normal => FontStyle::Normal,
        FontAssetFaceStyle::Italic => FontStyle::Italic,
        FontAssetFaceStyle::Oblique => FontStyle::Oblique(0.0),
    }
}

fn variation_coords_from_font_asset(coords: &[FontAssetVariationCoord]) -> VariationCoords {
    VariationCoords(
        coords
            .iter()
            .filter_map(|coord| variation_tag(coord.tag.as_str()).map(|tag| (tag, coord.value)))
            .collect(),
    )
}

fn variation_tag(tag: &str) -> Option<u32> {
    let bytes: [u8; 4] = tag.as_bytes().try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}
