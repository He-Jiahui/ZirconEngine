use thiserror::Error;

use super::{RenderImageDescriptor, RenderImageDimension};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureViewKind {
    D1,
    D2,
    D2Array,
    D3,
    Cube,
    CubeArray,
}

/// 校验后的图像形状把逻辑维度转换为纹理视图种类；D2 数组、D3 深度和立方体六面约束在此集中表达。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderImageShape {
    pub extent: TextureExtent3D,
    pub view_kind: TextureViewKind,
}

impl RenderImageShape {
    pub const fn array_layer_count(self) -> u32 {
        match self.view_kind {
            TextureViewKind::D2Array | TextureViewKind::Cube | TextureViewKind::CubeArray => {
                self.extent.depth_or_array_layers
            }
            TextureViewKind::D1 | TextureViewKind::D2 | TextureViewKind::D3 => 1,
        }
    }

    pub const fn depth(self) -> u32 {
        match self.view_kind {
            TextureViewKind::D3 => self.extent.depth_or_array_layers,
            TextureViewKind::D1
            | TextureViewKind::D2
            | TextureViewKind::D2Array
            | TextureViewKind::Cube
            | TextureViewKind::CubeArray => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderImageShapeError {
    #[error("texture {axis} extent must be greater than zero")]
    ZeroExtent { axis: &'static str },
    #[error("1d textures require height, depth, and array layers to equal one")]
    InvalidD1Extent,
    #[error("cube textures require square faces, found {width}x{height}")]
    CubeFacesNotSquare { width: u32, height: u32 },
    #[error("cube texture layer count must be a non-zero multiple of six, found {layers}")]
    CubeLayerCount { layers: u32 },
}

impl RenderImageDescriptor {
    pub fn validated_shape(&self) -> Result<RenderImageShape, RenderImageShapeError> {
        validate_non_zero("width", self.width)?;
        validate_non_zero("height", self.height)?;
        validate_non_zero("depth_or_array_layers", self.depth_or_array_layers)?;

        let view_kind = match self.dimension {
            RenderImageDimension::D1 => {
                if self.height != 1 || self.depth_or_array_layers != 1 {
                    return Err(RenderImageShapeError::InvalidD1Extent);
                }
                TextureViewKind::D1
            }
            RenderImageDimension::D2 => {
                if self.depth_or_array_layers == 1 {
                    TextureViewKind::D2
                } else {
                    TextureViewKind::D2Array
                }
            }
            RenderImageDimension::D3 => TextureViewKind::D3,
            RenderImageDimension::Cube => {
                if self.width != self.height {
                    return Err(RenderImageShapeError::CubeFacesNotSquare {
                        width: self.width,
                        height: self.height,
                    });
                }
                if self.depth_or_array_layers % 6 != 0 {
                    return Err(RenderImageShapeError::CubeLayerCount {
                        layers: self.depth_or_array_layers,
                    });
                }
                if self.depth_or_array_layers == 6 {
                    TextureViewKind::Cube
                } else {
                    TextureViewKind::CubeArray
                }
            }
        };

        Ok(RenderImageShape {
            extent: TextureExtent3D {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.depth_or_array_layers,
            },
            view_kind,
        })
    }
}

const fn validate_non_zero(axis: &'static str, value: u32) -> Result<(), RenderImageShapeError> {
    if value == 0 {
        Err(RenderImageShapeError::ZeroExtent { axis })
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/shape.rs"]
mod tests;
