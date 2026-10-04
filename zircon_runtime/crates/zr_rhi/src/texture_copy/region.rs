use serde::{Deserialize, Serialize};

use super::TextureCopyAspect;

/// Identifies the mip level, layer or slice, and rectangle for a texture copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 缺省层数按一层读取，序列化时省略该默认值，以保持已有复制区域的文本格式。
pub struct TextureCopyRegion {
    pub mip_level: u32,
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
    pub width: u32,
    pub height: u32,
    #[serde(
        default = "default_depth_or_array_layers",
        skip_serializing_if = "depth_or_array_layers_is_one"
    )]
    pub depth_or_array_layers: u32,
    #[serde(default)]
    pub aspect: TextureCopyAspect,
}

impl TextureCopyRegion {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width,
            height,
            depth_or_array_layers: 1,
            aspect: TextureCopyAspect::All,
        }
    }

    pub const fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = mip_level;
        self
    }

    pub const fn with_origin(mut self, x: u32, y: u32, z: u32) -> Self {
        self.origin_x = x;
        self.origin_y = y;
        self.origin_z = z;
        self
    }

    pub const fn with_depth_or_array_layers(mut self, depth_or_array_layers: u32) -> Self {
        self.depth_or_array_layers = depth_or_array_layers;
        self
    }

    pub const fn with_aspect(mut self, aspect: TextureCopyAspect) -> Self {
        self.aspect = aspect;
        self
    }
}

const fn default_depth_or_array_layers() -> u32 {
    1
}

const fn depth_or_array_layers_is_one(value: &u32) -> bool {
    *value == 1
}

#[cfg(test)]
#[path = "tests/region.rs"]
mod tests;
