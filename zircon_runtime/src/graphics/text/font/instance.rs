use std::collections::{BTreeMap, HashMap};
use std::mem::size_of;

use crate::core::framework::render::{FontFaceId, InstancedFaceId, VariationCoords};

const FONT_INSTANCE_HASH_DOMAIN: &[u8] = b"zircon-font-instance-v1";
const OPEN_TYPE_NORMALIZED_COORDINATE_SCALE: f32 = 16_384.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FontInstance {
    pub(crate) face: FontFaceId,
    pub(crate) variations: VariationCoords,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FontInstanceRegistry {
    instances: HashMap<InstancedFaceId, FontInstance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum FontInstanceError {
    #[error("font variation coordinate {tag:#010x} is not finite")]
    NonFiniteCoordinate { tag: u32 },
    #[error("font instance identity collision for {id:?}")]
    IdentityCollision { id: InstancedFaceId },
}

impl FontInstanceRegistry {
    pub(crate) fn resolve_or_insert(
        &mut self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontInstanceError> {
        let variations = canonical_variation_coords(variations)?;
        let id = font_instance_identity(face, &variations)?;
        let instance = FontInstance { face, variations };
        match self.instances.get(&id) {
            Some(existing) if existing == &instance => Ok(id),
            Some(_) => Err(FontInstanceError::IdentityCollision { id }),
            None => {
                self.instances.insert(id, instance);
                Ok(id)
            }
        }
    }

    pub(crate) fn get(&self, id: InstancedFaceId) -> Option<&FontInstance> {
        self.instances.get(&id)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.instances.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

pub(super) fn font_instance_identity(
    face: FontFaceId,
    variations: &VariationCoords,
) -> Result<InstancedFaceId, FontInstanceError> {
    let variations = canonical_variation_coords(variations)?;
    Ok(font_instance_id(face, &variations))
}

fn canonical_variation_coords(
    variations: &VariationCoords,
) -> Result<VariationCoords, FontInstanceError> {
    let mut coordinates = BTreeMap::new();
    for &(tag, value) in &variations.0 {
        if !value.is_finite() {
            return Err(FontInstanceError::NonFiniteCoordinate { tag });
        }
        coordinates.insert(tag, if value == 0.0 { 0.0 } else { value });
    }
    Ok(VariationCoords(coordinates.into_iter().collect()))
}

pub(super) fn variations_with_font_weight(
    font_bytes: &[u8],
    face_index: u32,
    variations: &VariationCoords,
    font_weight: u16,
) -> VariationCoords {
    variations_for_face(font_bytes, face_index, variations, Some(font_weight))
}

pub(super) fn variations_for_face(
    font_bytes: &[u8],
    face_index: u32,
    variations: &VariationCoords,
    font_weight: Option<u16>,
) -> VariationCoords {
    let Ok(face) = ttf_parser::Face::parse(font_bytes, face_index) else {
        return variations.clone();
    };
    let axes = face.variation_axes().into_iter().collect::<Vec<_>>();
    if axes.is_empty() {
        return VariationCoords::default();
    }

    let mut weighted = variations.clone();
    if let Some(font_weight) = font_weight {
        if axes
            .iter()
            .any(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
        {
            weighted
                .0
                .push((u32::from_be_bytes(*b"wght"), f32::from(font_weight)));
        }
    }
    let Ok(weighted) = canonical_variation_coords(&weighted) else {
        return variations.clone();
    };
    VariationCoords(
        weighted
            .0
            .into_iter()
            .filter_map(|(tag, value)| {
                let axis = axes
                    .iter()
                    .find(|axis| u32::from_be_bytes(axis.tag.to_bytes()) == tag)?;
                let value =
                    quantized_axis_value(value, axis.min_value, axis.def_value, axis.max_value);
                (value != axis.def_value).then_some((tag, value))
            })
            .collect(),
    )
}

fn quantized_axis_value(value: f32, min_value: f32, default_value: f32, max_value: f32) -> f32 {
    let value = value.clamp(min_value, max_value);
    if value == default_value {
        return default_value;
    }
    let span = if value < default_value {
        default_value - min_value
    } else {
        max_value - default_value
    };
    if span <= f32::EPSILON {
        return default_value;
    }
    let normalized = (value - default_value) / span;
    let normalized = (normalized.clamp(-1.0, 1.0) * OPEN_TYPE_NORMALIZED_COORDINATE_SCALE).trunc()
        / OPEN_TYPE_NORMALIZED_COORDINATE_SCALE;
    if normalized < 0.0 {
        default_value + normalized * (default_value - min_value)
    } else {
        default_value + normalized * (max_value - default_value)
    }
}

fn font_instance_id(face: FontFaceId, variations: &VariationCoords) -> InstancedFaceId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FONT_INSTANCE_HASH_DOMAIN);
    hasher.update(&face.0.to_le_bytes());
    for (tag, value) in &variations.0 {
        hasher.update(&tag.to_be_bytes());
        hasher.update(&value.to_bits().to_le_bytes());
    }
    let mut id_bytes = [0_u8; size_of::<u64>()];
    id_bytes.copy_from_slice(&hasher.finalize().as_bytes()[..size_of::<u64>()]);
    InstancedFaceId(u64::from_le_bytes(id_bytes))
}

#[cfg(test)]
#[path = "instance/tests.rs"]
mod tests;
