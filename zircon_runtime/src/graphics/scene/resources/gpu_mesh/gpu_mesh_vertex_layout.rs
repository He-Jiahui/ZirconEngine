use super::gpu_mesh_vertex::GpuMeshVertex;

impl GpuMeshVertex {
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint16x4,
                    offset: 32,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 40,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 56,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 72,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 88,
                    shader_location: 7,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GpuMeshVertex;

    #[test]
    fn gpu_mesh_vertex_layout_appends_tangent_color_and_uv1_after_skinning_channels() {
        let layout = GpuMeshVertex::layout();

        assert_eq!(layout.array_stride, 96);
        assert_eq!(layout.attributes.len(), 8);
        assert_eq!(layout.attributes[3].shader_location, 3);
        assert_eq!(layout.attributes[3].offset, 32);
        assert_eq!(layout.attributes[4].shader_location, 4);
        assert_eq!(layout.attributes[4].offset, 40);
        assert_eq!(layout.attributes[5].format, wgpu::VertexFormat::Float32x4);
        assert_eq!(layout.attributes[5].shader_location, 5);
        assert_eq!(layout.attributes[5].offset, 56);
        assert_eq!(layout.attributes[6].format, wgpu::VertexFormat::Float32x4);
        assert_eq!(layout.attributes[6].shader_location, 6);
        assert_eq!(layout.attributes[6].offset, 72);
        assert_eq!(layout.attributes[7].format, wgpu::VertexFormat::Float32x2);
        assert_eq!(layout.attributes[7].shader_location, 7);
        assert_eq!(layout.attributes[7].offset, 88);
    }
}
