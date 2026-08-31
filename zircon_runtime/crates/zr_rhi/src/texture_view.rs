use serde::{Deserialize, Serialize};

use crate::{TextureFormat, TextureHandle, TextureViewAspect, TextureViewDimension};

/// Describes one persistent shader-visible subresource view. Counts of
/// `None` select the remaining mip levels or array layers from their base.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureViewDesc {
    pub label: Option<String>,
    pub texture: TextureHandle,
    /// `None` selects the parent texture format. An alternate format must be
    /// declared by `TextureDesc::view_formats` when the texture is created.
    #[serde(default)]
    pub format: Option<TextureFormat>,
    #[serde(default)]
    pub aspect: TextureViewAspect,
    pub dimension: TextureViewDimension,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

impl TextureViewDesc {
    pub fn new(
        label: impl Into<String>,
        texture: TextureHandle,
        dimension: TextureViewDimension,
    ) -> Self {
        Self {
            label: Some(label.into()),
            texture,
            format: None,
            aspect: TextureViewAspect::All,
            dimension,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }
    }

    pub const fn with_mip_range(mut self, base_mip_level: u32, mip_level_count: u32) -> Self {
        self.base_mip_level = base_mip_level;
        self.mip_level_count = Some(mip_level_count);
        self
    }

    pub const fn with_format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub const fn with_aspect(mut self, aspect: TextureViewAspect) -> Self {
        self.aspect = aspect;
        self
    }

    pub const fn resolved_format(&self, parent_format: TextureFormat) -> TextureFormat {
        match self.format {
            Some(format) => format,
            None => parent_format,
        }
    }

    pub const fn with_array_layer_range(
        mut self,
        base_array_layer: u32,
        array_layer_count: u32,
    ) -> Self {
        self.base_array_layer = base_array_layer;
        self.array_layer_count = Some(array_layer_count);
        self
    }
}
