use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};

use super::{TextureArrayLayout, TextureAsset, TextureAssetDescriptor, TexturePayload};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureArrayLayerSource {
    Reference(AssetReference),
    SlicedFromImage {
        reference: AssetReference,
        layout: TextureArrayLayout,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Texture2DArrayAsset {
    pub uri: AssetUri,
    pub descriptor: TextureAssetDescriptor,
    pub layers: Vec<TextureArrayLayerSource>,
}

impl Texture2DArrayAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.layers
            .iter()
            .map(|layer| match layer {
                TextureArrayLayerSource::Reference(reference)
                | TextureArrayLayerSource::SlicedFromImage { reference, .. } => reference.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Texture2DArrayAssetError {
    #[error("texture array requires at least one layer")]
    Empty,
    #[error("texture array manifest declares {expected} layers but importer resolved {actual}")]
    LayerCount { expected: usize, actual: usize },
    #[error("texture array layer {layer} must be an rgba8 single-layer 2d texture")]
    LayerPayload { layer: usize },
    #[error(
        "texture array layer {layer} dimensions {width}x{height} do not match {expected_width}x{expected_height}"
    )]
    LayerDimensionMismatch {
        layer: usize,
        expected_width: u32,
        expected_height: u32,
        width: u32,
        height: u32,
    },
    #[error("texture array layer {layer} format `{actual}` does not match `{expected}`")]
    LayerFormatMismatch {
        layer: usize,
        expected: String,
        actual: String,
    },
    #[error("texture array layer {layer} color space does not match the first layer")]
    LayerColorSpaceMismatch { layer: usize },
    #[error("texture array layer {layer} mip count {actual} does not match {expected}")]
    LayerMipCountMismatch {
        layer: usize,
        expected: u32,
        actual: u32,
    },
    #[error("texture array layer {layer} rgba8 payload length {actual} does not match {expected}")]
    LayerPayloadLength {
        layer: usize,
        expected: usize,
        actual: usize,
    },
    #[error("texture array rgba8 payload extent is too large")]
    PayloadExtentOverflow,
}

pub fn texture_asset_from_array_layers(
    asset: Texture2DArrayAsset,
    mut layers: Vec<TextureAsset>,
) -> Result<TextureAsset, Texture2DArrayAssetError> {
    if layers.is_empty() {
        return Err(Texture2DArrayAssetError::Empty);
    }
    let uses_sliced_source = matches!(
        asset.layers.as_slice(),
        [TextureArrayLayerSource::SlicedFromImage { .. }]
    );
    if !uses_sliced_source && asset.layers.len() != layers.len() {
        return Err(Texture2DArrayAssetError::LayerCount {
            expected: asset.layers.len(),
            actual: layers.len(),
        });
    }

    let first_descriptor = take_render_image_descriptor(&mut layers[0]);
    let width = layers[0].width;
    let height = layers[0].height;
    let mip_count = first_descriptor.mip_count.max(1);
    let expected_layer_len = rgba8_mip_chain_len(width, height, mip_count)
        .ok_or(Texture2DArrayAssetError::PayloadExtentOverflow)?;

    validate_array_layer(
        0,
        &layers[0],
        &first_descriptor,
        &first_descriptor,
        width,
        height,
        mip_count,
        expected_layer_len,
    )?;
    for (layer, texture) in layers.iter_mut().enumerate().skip(1) {
        let descriptor = take_render_image_descriptor(texture);
        validate_array_layer(
            layer,
            texture,
            &descriptor,
            &first_descriptor,
            width,
            height,
            mip_count,
            expected_layer_len,
        )?;
    }

    let rgba = interleave_layer_mips(&layers, width, height, mip_count)?;
    let layer_count = layers.len() as u32;
    let mut descriptor = asset.descriptor;
    descriptor.dimension = RenderImageDimension::D2;
    descriptor.depth_or_array_layers = layer_count;
    descriptor.array_layer_count = layer_count;
    descriptor.mip_count = mip_count;
    descriptor.format = first_descriptor.format;
    descriptor.color_space = first_descriptor.color_space;

    Ok(TextureAsset::new_rgba8(asset.uri, width, height, rgba).with_descriptor(descriptor))
}

fn take_render_image_descriptor(texture: &mut TextureAsset) -> RenderImageDescriptor {
    texture
        .descriptor
        .take()
        .unwrap_or_else(|| TextureAssetDescriptor::from_payload(&texture.payload))
        .into_render_image_descriptor(texture.width, texture.height)
}

fn validate_array_layer(
    layer: usize,
    texture: &TextureAsset,
    descriptor: &RenderImageDescriptor,
    expected_descriptor: &RenderImageDescriptor,
    expected_width: u32,
    expected_height: u32,
    expected_mip_count: u32,
    expected_layer_len: usize,
) -> Result<(), Texture2DArrayAssetError> {
    if texture.payload != TexturePayload::Rgba8
        || descriptor.dimension != RenderImageDimension::D2
        || descriptor.array_layer_count != 1
        || descriptor.depth_or_array_layers != 1
    {
        return Err(Texture2DArrayAssetError::LayerPayload { layer });
    }
    if texture.width != expected_width || texture.height != expected_height {
        return Err(Texture2DArrayAssetError::LayerDimensionMismatch {
            layer,
            expected_width,
            expected_height,
            width: texture.width,
            height: texture.height,
        });
    }
    if descriptor.format != expected_descriptor.format {
        return Err(Texture2DArrayAssetError::LayerFormatMismatch {
            layer,
            expected: expected_descriptor.format.clone(),
            actual: descriptor.format.clone(),
        });
    }
    if descriptor.color_space != expected_descriptor.color_space {
        return Err(Texture2DArrayAssetError::LayerColorSpaceMismatch { layer });
    }
    if descriptor.mip_count.max(1) != expected_mip_count {
        return Err(Texture2DArrayAssetError::LayerMipCountMismatch {
            layer,
            expected: expected_mip_count,
            actual: descriptor.mip_count.max(1),
        });
    }
    if texture.rgba.len() != expected_layer_len {
        return Err(Texture2DArrayAssetError::LayerPayloadLength {
            layer,
            expected: expected_layer_len,
            actual: texture.rgba.len(),
        });
    }
    Ok(())
}

fn interleave_layer_mips(
    layers: &[TextureAsset],
    width: u32,
    height: u32,
    mip_count: u32,
) -> Result<Vec<u8>, Texture2DArrayAssetError> {
    let layer_len = rgba8_mip_chain_len(width, height, mip_count)
        .ok_or(Texture2DArrayAssetError::PayloadExtentOverflow)?;
    let capacity = layer_len
        .checked_mul(layers.len())
        .ok_or(Texture2DArrayAssetError::PayloadExtentOverflow)?;
    let mut rgba = Vec::with_capacity(capacity);
    let mut layer_mip_offset = 0_usize;
    for level in 0..mip_count {
        let mip_len = rgba8_level_len((width >> level).max(1), (height >> level).max(1))
            .ok_or(Texture2DArrayAssetError::PayloadExtentOverflow)?;
        for layer in layers {
            rgba.extend_from_slice(&layer.rgba[layer_mip_offset..layer_mip_offset + mip_len]);
        }
        layer_mip_offset += mip_len;
    }
    Ok(rgba)
}

fn rgba8_mip_chain_len(width: u32, height: u32, mip_count: u32) -> Option<usize> {
    let mut total = 0_usize;
    for level in 0..mip_count {
        total = total.checked_add(rgba8_level_len(
            (width >> level).max(1),
            (height >> level).max(1),
        )?)?;
    }
    Some(total)
}

fn rgba8_level_len(width: u32, height: u32) -> Option<usize> {
    width
        .checked_mul(height)?
        .checked_mul(4)
        .and_then(|value| usize::try_from(value).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(path: &str) -> AssetUri {
        AssetUri::parse(path).expect("texture test URI must be valid")
    }

    #[test]
    fn runtime92_owned_descriptors_recovery_batch_array_preserves_payload_and_extent() {
        let asset = Texture2DArrayAsset {
            uri: uri("res://textures/runtime92-array.ztexture"),
            descriptor: TextureAssetDescriptor::default(),
            layers: vec![
                TextureArrayLayerSource::Reference(AssetReference::from_locator(uri(
                    "res://textures/runtime92-array-0.png",
                ))),
                TextureArrayLayerSource::Reference(AssetReference::from_locator(uri(
                    "res://textures/runtime92-array-1.png",
                ))),
            ],
        };
        let layers = vec![
            TextureAsset::new_rgba8(
                uri("res://textures/runtime92-array-0.png"),
                2,
                1,
                vec![1; 8],
            ),
            TextureAsset::new_rgba8(
                uri("res://textures/runtime92-array-1.png"),
                2,
                1,
                vec![2; 8],
            ),
        ];

        let output = texture_asset_from_array_layers(asset, layers).unwrap();

        assert_eq!(output.rgba, [vec![1; 8], vec![2; 8]].concat());
        let descriptor = output.descriptor.unwrap();
        assert_eq!(descriptor.dimension, RenderImageDimension::D2);
        assert_eq!(descriptor.depth_or_array_layers, 2);
        assert_eq!(descriptor.array_layer_count, 2);
    }
}
