use serde::{Deserialize, Serialize};

use super::TextureCopyAspect;

/// Identifies the mip level, layer or slice, and rectangle for a texture copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
mod tests {
    use super::TextureCopyRegion;

    #[test]
    fn texture_copy_region_defaults_to_one_layer_and_can_select_a_contiguous_range() {
        let default_region = TextureCopyRegion::new(4, 2);
        let layered_region = default_region.with_depth_or_array_layers(6);

        assert_eq!(default_region.depth_or_array_layers, 1);
        assert_eq!(layered_region.depth_or_array_layers, 6);
        assert_eq!(layered_region.width, 4);
        assert_eq!(layered_region.height, 2);
    }

    #[test]
    fn legacy_serialized_region_defaults_to_one_layer_without_expanding_new_output() {
        let legacy = r#"{"mip_level":0,"origin_x":0,"origin_y":0,"origin_z":0,"width":4,"height":2,"aspect":"All"}"#;
        let region: TextureCopyRegion = serde_json::from_str(legacy).unwrap();

        assert_eq!(region.depth_or_array_layers, 1);
        assert!(!serde_json::to_string(&region)
            .unwrap()
            .contains("depth_or_array_layers"));
    }
}
