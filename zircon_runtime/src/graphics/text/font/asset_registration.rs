use std::path::{Path, PathBuf};

use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetVariationCoord};
use crate::core::framework::render::{FontFaceDescriptor, FontStyle, FontWeight, VariationCoords};

use super::database::{canonical_source_key, normalized_family_key};
use super::descriptors::{descriptor_from_font_bytes, stretch_from_ttf_width_class};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontAssetSourceKey {
    path: PathBuf,
    face_index: u32,
    family: String,
    weight: u16,
    style: FontStyleKey,
    stretch: u16,
    variations: Vec<(u32, u32)>,
}

impl FontAssetSourceKey {
    pub(super) fn from_descriptor(source_path: &Path, descriptor: &FontFaceDescriptor) -> Self {
        let mut variations: Vec<(u32, u32)> = descriptor
            .variations
            .0
            .iter()
            .map(|(tag, value)| (*tag, value.to_bits()))
            .collect();
        variations.sort_unstable();
        Self {
            path: canonical_source_key(source_path),
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

pub(super) fn font_asset_descriptors(
    asset: &FontAsset,
    bytes: &[u8],
    source_path: &Path,
) -> Vec<FontFaceDescriptor> {
    let primary_index = primary_family_member_index(asset);
    let mut descriptors = Vec::new();

    if let Some(index) = primary_index {
        descriptors.push(descriptor_from_font_asset_member(
            bytes,
            source_path,
            &asset.family_members[index],
        ));
    } else {
        descriptors.push(descriptor_from_font_bytes(
            bytes,
            asset.family.as_deref(),
            source_path,
            asset.face_index,
        ));
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
) -> FontFaceDescriptor {
    let mut descriptor = descriptor_from_font_bytes(
        bytes,
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
    descriptor
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
