use super::RenderGraphTextureAspect;

/// Mip, array-layer, and plane scope addressed by a texture access.
///
/// A `None` count extends through the remaining descriptor range and is
/// resolved during graph compilation, never by an executor-local heuristic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphTextureSubresourceRange {
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
    pub aspect: RenderGraphTextureAspect,
}

impl RenderGraphTextureSubresourceRange {
    pub const fn full() -> Self {
        Self {
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            aspect: RenderGraphTextureAspect::All,
        }
    }

    pub const fn single_mip(mip_level: u32) -> Self {
        Self {
            base_mip_level: mip_level,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: None,
            aspect: RenderGraphTextureAspect::All,
        }
    }

    pub const fn with_array_layers(
        mut self,
        base_array_layer: u32,
        array_layer_count: u32,
    ) -> Self {
        self.base_array_layer = base_array_layer;
        self.array_layer_count = Some(array_layer_count);
        self
    }

    pub const fn with_aspect(mut self, aspect: RenderGraphTextureAspect) -> Self {
        self.aspect = aspect;
        self
    }
}
