use crate::graphics::shader::template::ShaderVertexInputScalarKind;

use super::mesh_shader_vertex_contract::{
    MeshShaderVertexAttribute, MeshShaderVertexLayoutContract,
};

impl MeshShaderVertexLayoutContract {
    pub(super) fn from_wgpu_vertex_buffer_layouts<'a>(
        layouts: impl IntoIterator<Item = wgpu::VertexBufferLayout<'a>>,
    ) -> Result<Self, String> {
        Self::try_new(layouts.into_iter().flat_map(|layout| {
            layout.attributes.iter().map(|attribute| {
                MeshShaderVertexAttribute::new(
                    attribute.shader_location,
                    wgpu_vertex_scalar_kind(attribute.format),
                )
            })
        }))
    }
}

fn wgpu_vertex_scalar_kind(format: wgpu::VertexFormat) -> ShaderVertexInputScalarKind {
    use wgpu::VertexFormat as Format;

    match format {
        Format::Uint8
        | Format::Uint8x2
        | Format::Uint8x4
        | Format::Uint16
        | Format::Uint16x2
        | Format::Uint16x4
        | Format::Uint32
        | Format::Uint32x2
        | Format::Uint32x3
        | Format::Uint32x4 => ShaderVertexInputScalarKind::Uint,
        Format::Sint8
        | Format::Sint8x2
        | Format::Sint8x4
        | Format::Sint16
        | Format::Sint16x2
        | Format::Sint16x4
        | Format::Sint32
        | Format::Sint32x2
        | Format::Sint32x3
        | Format::Sint32x4 => ShaderVertexInputScalarKind::Sint,
        Format::Unorm8
        | Format::Unorm8x2
        | Format::Unorm8x4
        | Format::Snorm8
        | Format::Snorm8x2
        | Format::Snorm8x4
        | Format::Unorm16
        | Format::Unorm16x2
        | Format::Unorm16x4
        | Format::Snorm16
        | Format::Snorm16x2
        | Format::Snorm16x4
        | Format::Float16
        | Format::Float16x2
        | Format::Float16x4
        | Format::Float32
        | Format::Float32x2
        | Format::Float32x3
        | Format::Float32x4
        | Format::Float64
        | Format::Float64x2
        | Format::Float64x3
        | Format::Float64x4
        | Format::Unorm10_10_10_2
        | Format::Unorm8x4Bgra => ShaderVertexInputScalarKind::Float,
    }
}

#[cfg(test)]
#[path = "tests/mesh_shader_vertex_contract_wgpu.rs"]
mod tests;
