use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};

use super::{
    metadata, payload::default_texture_payload, TextureArrayLayout, TextureAssetDescriptor,
    TexturePayload,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureAsset {
    pub uri: AssetUri,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    #[serde(default = "default_texture_payload")]
    pub payload: TexturePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<TextureAssetDescriptor>,
}

impl TextureAsset {
    pub fn new_rgba8(uri: AssetUri, width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self {
            uri,
            width,
            height,
            rgba,
            payload: TexturePayload::Rgba8,
            descriptor: Some(TextureAssetDescriptor::rgba8_srgb()),
        }
    }

    pub fn new_container(
        uri: AssetUri,
        width: u32,
        height: u32,
        format: impl Into<String>,
        bytes: Vec<u8>,
        mip_count: u32,
        array_layer_count: u32,
    ) -> Self {
        let format = format.into();
        Self {
            uri,
            width,
            height,
            rgba: Vec::new(),
            payload: TexturePayload::Container {
                format: format.clone(),
                bytes,
                mip_count,
                array_layers: array_layer_count,
            },
            descriptor: Some(TextureAssetDescriptor::container(
                format,
                mip_count,
                array_layer_count,
            )),
        }
    }

    pub fn texture_descriptor(&self) -> TextureAssetDescriptor {
        metadata::texture_asset_descriptor(self)
    }

    pub fn with_descriptor(mut self, descriptor: TextureAssetDescriptor) -> Self {
        self.descriptor = Some(descriptor.normalized());
        self
    }

    pub fn with_import_settings(mut self, settings: &toml::Table) -> Result<Self, String> {
        let mut descriptor = self.texture_descriptor().with_import_settings(settings)?;
        if let Some(array_layout) = TextureArrayLayout::from_import_settings(settings)? {
            self.apply_array_layout(array_layout, &mut descriptor)?;
        }
        self.descriptor = Some(descriptor);
        Ok(self)
    }

    pub fn render_image_descriptor(&self) -> RenderImageDescriptor {
        metadata::render_image_descriptor(self)
    }

    fn apply_array_layout(
        &mut self,
        array_layout: TextureArrayLayout,
        descriptor: &mut TextureAssetDescriptor,
    ) -> Result<(), String> {
        if self.payload != TexturePayload::Rgba8 {
            return Err(
                "texture import setting `array_layout` requires a decoded rgba8 image".to_string(),
            );
        }
        if descriptor.dimension != RenderImageDimension::D2 {
            return Err("texture import setting `array_layout` requires a 2d image".to_string());
        }
        if descriptor.depth_or_array_layers != 1 || descriptor.array_layer_count != 1 {
            return Err(
                "texture import setting `array_layout` requires a single-layer image".to_string(),
            );
        }
        let layers = match array_layout {
            TextureArrayLayout::RowCount { rows } => {
                nonzero_array_layout_value("array_layout.row_count", rows)?
            }
            TextureArrayLayout::RowHeight { pixels } => {
                let pixels = nonzero_array_layout_value("array_layout.row_height", pixels)?;
                if self.height % pixels != 0 {
                    return Err(format!(
                        "texture import setting `array_layout` can not evenly divide height = {} by row_height = {}",
                        self.height, pixels
                    ));
                }
                nonzero_array_layout_value("array_layout.row_count", self.height / pixels)?
            }
        };
        if self.height % layers != 0 {
            return Err(format!(
                "texture import setting `array_layout` can not evenly divide height = {} by layers = {}",
                self.height, layers
            ));
        }
        let expected_len = rgba8_len(self.width, self.height)?;
        if self.rgba.len() != expected_len {
            return Err(format!(
                "texture import setting `array_layout` expected rgba byte length {} but found {}",
                expected_len,
                self.rgba.len()
            ));
        }

        self.height /= layers;
        descriptor.depth_or_array_layers = layers;
        descriptor.array_layer_count = layers;
        Ok(())
    }
}

fn nonzero_array_layout_value(name: &str, value: u32) -> Result<u32, String> {
    if value == 0 {
        return Err(format!(
            "texture import setting `{name}` must be greater than zero"
        ));
    }
    Ok(value)
}

fn rgba8_len(width: u32, height: u32) -> Result<usize, String> {
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| format!("texture rgba8 extent {width}x{height} is too large to validate"))
}
