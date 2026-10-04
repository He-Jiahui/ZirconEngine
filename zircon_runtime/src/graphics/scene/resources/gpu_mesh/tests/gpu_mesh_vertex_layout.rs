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

#[test]
fn gpu_mesh_previous_position_layout_reuses_position_at_velocity_location() {
    let layout = GpuMeshVertex::previous_position_layout();

    assert_eq!(layout.array_stride, 96);
    assert_eq!(layout.attributes.len(), 1);
    assert_eq!(layout.attributes[0].format, wgpu::VertexFormat::Float32x3);
    assert_eq!(layout.attributes[0].shader_location, 8);
    assert_eq!(layout.attributes[0].offset, 0);
}
