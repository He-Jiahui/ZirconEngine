use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};

use super::{TextureAsset, TextureAssetDescriptor, TexturePayload};

pub const CUBEMAP_FACE_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CubemapSourceLayout {
    SixFiles,
    HorizontalCross,
    VerticalCross,
    Equirectangular,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CubemapAsset {
    pub uri: AssetUri,
    pub descriptor: TextureAssetDescriptor,
    pub source_layout: CubemapSourceLayout,
    pub sources: Vec<AssetReference>,
}

impl CubemapAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.sources.clone()
    }

    pub fn validate_source_count(&self) -> Result<(), CubemapAssetError> {
        let expected = match self.source_layout {
            CubemapSourceLayout::SixFiles => CUBEMAP_FACE_COUNT,
            CubemapSourceLayout::HorizontalCross
            | CubemapSourceLayout::VerticalCross
            | CubemapSourceLayout::Equirectangular => 1,
        };
        if self.sources.len() != expected {
            return Err(CubemapAssetError::SourceCount {
                layout: self.source_layout,
                expected,
                actual: self.sources.len(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CubemapAssetError {
    #[error("cubemap layout {layout:?} requires {expected} source reference(s), found {actual}")]
    SourceCount {
        layout: CubemapSourceLayout,
        expected: usize,
        actual: usize,
    },
    #[error("cubemap assembly requires six decoded faces, found {actual}")]
    FaceCount { actual: usize },
    #[error("cubemap face {face} must be an rgba8 single-layer 2d texture")]
    FacePayload { face: usize },
    #[error(
        "cubemap face {face} dimensions {width}x{height} do not match square face size {expected}"
    )]
    FaceDimensionMismatch {
        face: usize,
        expected: u32,
        width: u32,
        height: u32,
    },
    #[error("cubemap face {face} format `{actual}` does not match `{expected}`")]
    FaceFormatMismatch {
        face: usize,
        expected: String,
        actual: String,
    },
    #[error("cubemap face {face} color space does not match the first face")]
    FaceColorSpaceMismatch { face: usize },
    #[error("cubemap face {face} mip count {actual} does not match {expected}")]
    FaceMipCountMismatch {
        face: usize,
        expected: u32,
        actual: u32,
    },
    #[error("cubemap face {face} rgba8 payload length {actual} does not match {expected}")]
    FacePayloadLength {
        face: usize,
        expected: usize,
        actual: usize,
    },
    #[error("cubemap rgba8 payload extent is too large")]
    PayloadExtentOverflow,
}

pub fn texture_asset_from_cubemap_faces(
    asset: CubemapAsset,
    faces: Vec<TextureAsset>,
) -> Result<TextureAsset, CubemapAssetError> {
    asset.validate_source_count()?;
    if faces.len() != CUBEMAP_FACE_COUNT {
        return Err(CubemapAssetError::FaceCount {
            actual: faces.len(),
        });
    }

    let first = &faces[0];
    let first_descriptor = first.render_image_descriptor();
    validate_face_shape(0, first, &first_descriptor, first.width)?;
    let face_size = first.width;
    let mip_count = first_descriptor.mip_count.max(1);
    let expected_face_len = rgba8_mip_chain_len(face_size, mip_count)
        .ok_or(CubemapAssetError::PayloadExtentOverflow)?;

    for (face, texture) in faces.iter().enumerate() {
        let descriptor = texture.render_image_descriptor();
        validate_face_shape(face, texture, &descriptor, face_size)?;
        if descriptor.format != first_descriptor.format {
            return Err(CubemapAssetError::FaceFormatMismatch {
                face,
                expected: first_descriptor.format.clone(),
                actual: descriptor.format,
            });
        }
        if descriptor.color_space != first_descriptor.color_space {
            return Err(CubemapAssetError::FaceColorSpaceMismatch { face });
        }
        if descriptor.mip_count.max(1) != mip_count {
            return Err(CubemapAssetError::FaceMipCountMismatch {
                face,
                expected: mip_count,
                actual: descriptor.mip_count.max(1),
            });
        }
        if texture.rgba.len() != expected_face_len {
            return Err(CubemapAssetError::FacePayloadLength {
                face,
                expected: expected_face_len,
                actual: texture.rgba.len(),
            });
        }
    }

    let rgba = interleave_face_mips(&faces, face_size, mip_count)?;
    let mut descriptor = asset.descriptor;
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = CUBEMAP_FACE_COUNT as u32;
    descriptor.array_layer_count = CUBEMAP_FACE_COUNT as u32;
    descriptor.mip_count = mip_count;
    descriptor.format = first_descriptor.format;
    descriptor.color_space = first_descriptor.color_space;

    Ok(TextureAsset::new_rgba8(asset.uri, face_size, face_size, rgba).with_descriptor(descriptor))
}

fn validate_face_shape(
    face: usize,
    texture: &TextureAsset,
    descriptor: &RenderImageDescriptor,
    expected_size: u32,
) -> Result<(), CubemapAssetError> {
    if texture.payload != TexturePayload::Rgba8
        || descriptor.dimension != RenderImageDimension::D2
        || descriptor.array_layer_count != 1
        || descriptor.depth_or_array_layers != 1
    {
        return Err(CubemapAssetError::FacePayload { face });
    }
    if texture.width != expected_size || texture.height != expected_size {
        return Err(CubemapAssetError::FaceDimensionMismatch {
            face,
            expected: expected_size,
            width: texture.width,
            height: texture.height,
        });
    }
    Ok(())
}

fn interleave_face_mips(
    faces: &[TextureAsset],
    face_size: u32,
    mip_count: u32,
) -> Result<Vec<u8>, CubemapAssetError> {
    let face_len = rgba8_mip_chain_len(face_size, mip_count)
        .ok_or(CubemapAssetError::PayloadExtentOverflow)?;
    let capacity = face_len
        .checked_mul(CUBEMAP_FACE_COUNT)
        .ok_or(CubemapAssetError::PayloadExtentOverflow)?;
    let mut rgba = Vec::with_capacity(capacity);
    let mut face_mip_offset = 0_usize;
    for level in 0..mip_count {
        let mip_size = (face_size >> level).max(1);
        let mip_len = rgba8_level_len(mip_size).ok_or(CubemapAssetError::PayloadExtentOverflow)?;
        for face in faces {
            rgba.extend_from_slice(&face.rgba[face_mip_offset..face_mip_offset + mip_len]);
        }
        face_mip_offset += mip_len;
    }
    Ok(rgba)
}

fn rgba8_mip_chain_len(face_size: u32, mip_count: u32) -> Option<usize> {
    let mut total = 0_usize;
    for level in 0..mip_count {
        total = total.checked_add(rgba8_level_len((face_size >> level).max(1))?)?;
    }
    Some(total)
}

fn rgba8_level_len(size: u32) -> Option<usize> {
    size.checked_mul(size)?
        .checked_mul(4)
        .and_then(|value| usize::try_from(value).ok())
}
